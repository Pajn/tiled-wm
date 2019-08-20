use std::collections::BTreeMap;
use std::mem::transmute;
use std::os::raw::c_char;
use std::ffi::{CString, CStr};
use xkb::Keysym;
use xkbcommon_sys::{xkb_state_mod_name_is_active, XKB_STATE_MODS_DEPRESSED, xkb_keysym_t, xkb_state};
use std::process::Command;
use std::cmp;

#[allow(non_camel_case_types)]
pub struct wc_server {}

#[allow(non_camel_case_types)]
#[derive(Default, Debug, Clone)]
pub struct wlr_box {
	x: i32,
    y: i32,
    width: i32,
    height: i32,
}

pub type Id = u64;

#[derive(Debug)]
#[repr(C)]
pub struct IdArray {
    data: *mut Id,
    length: usize,
    capacity: usize,
}

impl IdArray {
    fn from_vec(mut vec: Vec<Id>) -> IdArray {
        let result = IdArray {
            data: vec.as_mut_ptr(),
            length: vec.len(),
            capacity: vec.capacity(),
        };
        std::mem::forget(vec);
        println!("result: {:?}", &result);
        result
    }
}

#[derive(Default, Debug)]
pub struct Window {
    id: Id,
    app_id: String,
    fullscreen: bool,
    geo: wlr_box,
    workspace: Id
}

impl Window {
    fn is_tiled(&self) -> bool {
        !self.fullscreen && self.app_id != "ulauncher"
    }
}

#[derive(Default, Debug)]
pub struct Workspace {
    id: Id,
    windows: Vec<Id>,
    active_monitor: Option<Id>,
}

#[derive(Debug)]
pub struct Monitor {
    id: Id,
    active_workspace: Id,
}

#[derive(Default, Debug)]
pub struct WindowManager {
    next_monitor_id: Id,
    next_workspace_id: Id,
    next_window_id: Id,

    focused_window: Option<Id>,
    active_workspace: Id,

    monitors: BTreeMap<Id, Monitor>,
    workspaces: BTreeMap<Id, Workspace>,
    windows: BTreeMap<Id, Window>,

    wc_server: Option<*mut wc_server>,
    focus_callback: Option<extern fn(server: *mut wc_server, window_id: Id) -> ()>,
    dirty_windows_callback: Option<extern fn(server: *mut wc_server, window_ids: *mut IdArray) -> ()>,
}

impl WindowManager {
    fn create_workspace(&mut self) -> Id {
        let id = self.next_workspace_id;
        self.next_workspace_id += 1;

        let workspace = Workspace {id, ..Default::default()};
        self.workspaces.insert(id, workspace);

        id
    }

    fn get_workspace_by_window(&self, window_id: Id) -> Option<&Workspace> {
        self.windows.get(&window_id).and_then(|window| self.workspaces.get(&window.workspace))
    }

    fn get_workspace_id_by_window(&self, window_id: Id) -> Option<Id> {
        self.windows.get(&window_id).map(|window| window.workspace)
    }

    fn get_unassigned_workspaces(&self) -> Vec<&Workspace> {
        self.workspaces.values().filter(|w| w.active_monitor.is_none()).collect()
    }

    fn arrange_windows(&mut self, workspace_id: Id) -> () {
        let mut dirty_windows = vec![];

        let window_positions: Vec<_> = self.workspaces
            .get(&workspace_id).unwrap()
            .windows.iter()
            .scan(0, |next_x, window_id| {
                let window = self.windows.get(window_id).unwrap();

                if window.is_tiled() {
                    let x = *next_x;
                    *next_x += window.geo.width;
                    Some((*window_id, x))
                } else {
                    None
                }
            })
            .collect();

        for (window_id, x) in window_positions {
            let window = self.windows.get_mut(&window_id).unwrap();

            if window.geo.x != x {
                window.geo.x = x;
                dirty_windows.push(window_id);
            }
        }

        if let (Some(server), Some(dirty_windows_callback)) = (self.wc_server, self.dirty_windows_callback) {
            let mut dirty_windows = IdArray::from_vec(dirty_windows);
            dirty_windows_callback(server, &mut dirty_windows);
        }
    }

    fn focus_window(&mut self, window_id: Id) -> () {
        self.focused_window = Some(window_id);
        if let (Some(server), Some(focus_callback)) = (self.wc_server, self.focus_callback) {
            focus_callback(server, window_id);
        }
    }
}

#[no_mangle]
pub extern fn create_window_manager(server: *mut wc_server) -> *mut WindowManager {
    let mut wm = WindowManager::default();
    wm.wc_server = Some(server);
    let first_workspace_id = wm.create_workspace();
    wm.active_workspace = first_workspace_id;

    let _wm = unsafe { transmute(Box::new(wm)) };
    _wm
}

#[no_mangle]
pub extern fn register_focus_callback(_wm: *mut WindowManager, callback: extern fn(server: *mut wc_server, window_id: Id) -> ()) -> () {
    let mut wm = unsafe { &mut *_wm };

    wm.focus_callback = Some(callback);
}

#[no_mangle]
pub extern fn register_dirty_windows_callback(_wm: *mut WindowManager, callback: extern fn(server: *mut wc_server, window_ids: *mut IdArray) -> ()) -> () {
    let mut wm = unsafe { &mut *_wm };

    wm.dirty_windows_callback = Some(callback);
}

#[no_mangle]
pub extern fn create_monitor(_wm: *mut WindowManager) -> Id {
    let mut wm = unsafe { &mut *_wm };
    let id = wm.next_monitor_id;
    wm.next_monitor_id += 1;

    let unassigned_workspaces = wm.get_unassigned_workspaces();

    let workspace_id =
        if unassigned_workspaces.len() == 0 {
            let id = wm.create_workspace();
            wm.create_workspace();
            id
        } else if unassigned_workspaces.len() == 1 {
            let id = unassigned_workspaces[0].id;
            wm.create_workspace();
            id
        } else {
            unassigned_workspaces[0].id
        };
    
    let monitor = Monitor { id, active_workspace: workspace_id };
    wm.monitors.insert(id, monitor);
    wm.workspaces.get_mut(&workspace_id).unwrap().active_monitor = Some(id);

    println!("WM: {:?}", &wm);

    id
}

#[no_mangle]
pub extern fn create_window(_wm: *mut WindowManager) -> Id {
    let mut wm = unsafe { &mut *_wm };
    let id = wm.next_window_id;
    wm.next_window_id += 1;

    let window = Window { id, workspace: wm.active_workspace, ..Default::default() };
    wm.windows.insert(id, window);
    wm.workspaces.get_mut(&wm.active_workspace).unwrap().windows.push(id);
    wm.focused_window = Some(id);

    println!("WM: {:?}", &wm);

    id
}

#[no_mangle]
pub extern fn destroy_window(_wm: *mut WindowManager, window_id: Id) -> () {
    let wm = unsafe { &mut *_wm };

    let workspace = wm.workspaces.get_mut(&wm.get_workspace_id_by_window(window_id).unwrap()).unwrap();
    let workspace_id = workspace.id;
    
    let index = workspace.windows.iter().position(|id| *id == window_id).unwrap();

    if wm.focused_window == Some(window_id) {
        if index > 0 {
            wm.focused_window = workspace.windows.get(index - 1).copied();
        } else {
            wm.focused_window = workspace.windows.get(index + 1).copied();
        }
    }

    workspace.windows.remove(index);
    wm.windows.remove(&window_id);

    wm.arrange_windows(workspace_id);
}

#[no_mangle]
pub extern fn arrange_windows(_wm: *mut WindowManager, window_id: Id) -> () {
    let wm = unsafe { &mut *_wm };

    let workspace_id = wm.get_workspace_id_by_window(window_id).unwrap();

    wm.arrange_windows(workspace_id);
}

#[no_mangle]
pub extern fn configure_window(_wm: *mut WindowManager, window_id: Id, _geo: *mut wlr_box, app_id: *const c_char, fullscreen: bool) -> () {
    let wm = unsafe { &mut *_wm };
    let geo = unsafe { &mut *_geo };
    let app_id = unsafe {
        assert!(!app_id.is_null());

        CStr::from_ptr(app_id)
    };

    let window = wm.windows.get_mut(&window_id).unwrap();
    window.app_id = app_id.to_string_lossy().to_string();
    window.fullscreen = fullscreen;
    let workspace = wm.workspaces.get(&window.workspace).unwrap();
    let workspace_id = workspace.id;
    if window.is_tiled() {
        let x = wm.get_workspace_by_window(window_id).unwrap().windows.iter().fold(0, |sum, w| sum + wm.windows.get(w).unwrap().geo.width);
        geo.x = x;
        geo.y = 0;
        wm.windows.get_mut(&window_id).unwrap().geo = geo.clone();
    }

    println!("WM: {:?}", &wm);

    wm.arrange_windows(workspace_id);
}

#[no_mangle]
pub extern fn update_window(_wm: *mut WindowManager, window_id: Id, _geo: *mut wlr_box) -> () {
    let wm = unsafe { &mut *_wm };
    let geo = unsafe { &mut *_geo };

    let window = wm.windows.get(&window_id).unwrap();
    let workspace = wm.get_workspace_by_window(window_id).unwrap();
    let workspace_id = workspace.id;
    if window.is_tiled() {
        // FIX: X should not be last
        let x = wm.get_workspace_by_window(window_id).unwrap().windows.iter().fold(0, |sum, w| sum + wm.windows.get(w).unwrap().geo.width);
        geo.x = x;
        geo.y = 0;
        wm.windows.get_mut(&window_id).unwrap().geo = geo.clone();
    }

    println!("WM: {:?}", &wm);

    wm.arrange_windows(workspace_id);
}

#[no_mangle]
pub extern fn get_window_geometry(_wm: *mut WindowManager, window_id: Id) -> wlr_box {
    let wm = unsafe { &mut *_wm };
    println!("get_window_geometry: {:?}", &window_id);

    wm.windows.get(&window_id).unwrap().geo.clone()
}

#[no_mangle]
pub extern fn focus_window(_wm: *mut WindowManager, window_id: Id) -> () {
    let wm = unsafe { &mut *_wm };
    println!("focus_window: {:?}", &window_id);

    wm.focused_window = Some(window_id);
}

#[repr(u8)]
#[derive(Debug, PartialEq, Eq)]
pub enum KeyState {
    Released,
    Pressed,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Modifier {
    Control,
    Alt,
    Shift,
}

fn mods_are_active(state: *mut xkb_state, modifiers: Vec<Modifier>) -> bool {
    let mut expected_mod_state = BTreeMap::new();
    expected_mod_state.insert(Modifier::Control, modifiers.contains(&Modifier::Control));
    expected_mod_state.insert(Modifier::Alt, modifiers.contains(&Modifier::Alt));
    expected_mod_state.insert(Modifier::Shift, modifiers.contains(&Modifier::Shift));

    expected_mod_state.into_iter().all(|(modifier, expected_state)| {
        let mod_name = CString::new(match modifier {
            Modifier::Control => "Control",
            Modifier::Alt => "Alt",
            Modifier::Shift => "Shift",
        }).unwrap();

        unsafe { (xkb_state_mod_name_is_active(state, mod_name.as_ptr(), XKB_STATE_MODS_DEPRESSED) != 0) == expected_state }
    })
}

#[no_mangle]
pub extern fn keyboard_on_key(_wm: *mut WindowManager, keysym: xkb_keysym_t, key_state: KeyState, xkb_state: *mut xkb_state) -> bool {
    if key_state != KeyState::Pressed { return false; }

    match Keysym(keysym) {
        ::xkb::key::a if mods_are_active(xkb_state, vec![Modifier::Control]) => {
            Command::new("/bin/sh")
                .arg("-c")
                .arg("ulauncher-toggle")
                .spawn()
                .expect("failed to execute process");
            true
        }
        ::xkb::key::Left if mods_are_active(xkb_state, vec![Modifier::Shift]) => {
            let wm = unsafe { &mut *_wm };

            if let Some(focused_window) = wm.focused_window {
                let workspace = wm.get_workspace_by_window(focused_window).unwrap();
                let index = cmp::max(1, workspace.windows.iter().position(|id| *id == focused_window).unwrap());
                let window_id = *workspace.windows.get(index - 1).unwrap();

                wm.focus_window(window_id);
            } else {
                let workspace = wm.workspaces.get(&wm.active_workspace).unwrap();
                let window_id = *workspace.windows.get(0).unwrap();

                wm.focus_window(window_id);
            }
            println!("WM: {:?}", &wm);

            true
        }
        ::xkb::key::Right if mods_are_active(xkb_state, vec![Modifier::Shift]) => {
            let wm = unsafe { &mut *_wm };

            if let Some(focused_window) = wm.focused_window {
                let workspace = wm.get_workspace_by_window(focused_window).unwrap();
                let index = cmp::min(workspace.windows.len() - 2, workspace.windows.iter().position(|id| *id == focused_window).unwrap());
                let window_id = *workspace.windows.get(index + 1).unwrap();

                wm.focus_window(window_id);
            } else {
                let workspace = wm.workspaces.get(&wm.active_workspace).unwrap();
                let window_id = *workspace.windows.get(0).unwrap();

                wm.focus_window(window_id);
            }
            println!("WM: {:?}", &wm);

            true
        }
        ::xkb::key::Left if mods_are_active(xkb_state, vec![Modifier::Shift, Modifier::Control]) => {
            let wm = unsafe { &mut *_wm };

            if let Some(focused_window) = wm.focused_window {
                let workspace = wm.workspaces.get_mut(&wm.get_workspace_id_by_window(focused_window).unwrap()).unwrap();
                let index = workspace.windows.iter().position(|id| *id == focused_window).unwrap();

                if index > 0 {
                    let other_index = index - 1;

                    if index != other_index {
                        workspace.windows.swap(index, other_index);
                    }

                    let workspace_id = workspace.id;
                    wm.arrange_windows(workspace_id);
                }
            }
            println!("WM: {:?}", &wm);

            true
        }
        ::xkb::key::Right if mods_are_active(xkb_state, vec![Modifier::Shift, Modifier::Control]) => {
            let wm = unsafe { &mut *_wm };

            if let Some(focused_window) = wm.focused_window {
                let workspace = wm.workspaces.get_mut(&wm.get_workspace_id_by_window(focused_window).unwrap()).unwrap();
                let index = workspace.windows.iter().position(|id| *id == focused_window).unwrap();

                if index < workspace.windows.len() - 1 {
                    let other_index = index + 1;

                    if index != other_index {
                        workspace.windows.swap(index, other_index);
                    }

                    let workspace_id = workspace.id;
                    wm.arrange_windows(workspace_id);
                }
            }
            println!("WM: {:?}", &wm);

            true
        }
        _ => false
    }
}

#[no_mangle]
pub extern fn rust_free(array: IdArray) {
    if !array.data.is_null() {
        unsafe { Vec::from_raw_parts(array.data, array.length, array.capacity); }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}