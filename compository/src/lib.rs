use std::collections::BTreeMap;
use std::mem::transmute;
use std::os::raw::c_char;
use std::ffi::CStr;

#[allow(non_camel_case_types)]
#[derive(Default, Debug, Clone)]
pub struct wlr_box {
	x: i32,
    y: i32,
    width: i32,
    height: i32,
}

#[derive(Debug)]
#[repr(C)]
pub struct IdArray {
    data: *mut u64,
    length: usize,
    capacity: usize,
}

impl IdArray {
    fn from_vec(mut vec: Vec<u64>) -> IdArray {
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
    id: u64,
    app_id: String,
    fullscreen: bool,
    geo: wlr_box,
    workspace: u64
}

impl Window {
    fn is_tiled(&self) -> bool {
        !self.fullscreen && self.app_id != "ulauncher"
    }
}

#[derive(Default, Debug)]
pub struct Workspace {
    id: u64,
    windows: Vec<u64>,
    active_monitor: Option<u64>,
}

#[derive(Debug)]
pub struct Monitor {
    id: u64,
    active_workspace: u64,
}

#[derive(Default, Debug)]
pub struct WindowManager {
    next_monitor_id: u64,
    next_workspace_id: u64,
    next_window_id: u64,

    active_workspace: u64,

    monitors: BTreeMap<u64, Monitor>,
    workspaces: BTreeMap<u64, Workspace>,
    windows: BTreeMap<u64, Window>,
}

impl WindowManager {
    fn create_workspace(&mut self) -> u64 {
        let id = self.next_workspace_id;
        self.next_workspace_id += 1;

        let workspace = Workspace {id, ..Default::default()};
        self.workspaces.insert(id, workspace);

        id
    }

    fn get_workspace_by_window(&self, window_id: u64) -> Option<&Workspace> {
        self.windows.get(&window_id).and_then(|window| self.workspaces.get(&window.workspace))
    }

    fn get_workspace_id_by_window(&self, window_id: u64) -> Option<u64> {
        self.windows.get(&window_id).map(|window| window.workspace)
    }

    fn get_unassigned_workspaces(&self) -> Vec<&Workspace> {
        self.workspaces.values().filter(|w| w.active_monitor.is_none()).collect()
    }

    fn arrange_windows(&mut self, workspace_id: u64) -> Vec<u64> {
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

        dirty_windows
    }
}

#[no_mangle]
pub extern fn create_window_manager() -> *mut WindowManager {
    let mut wm = WindowManager::default();
    let first_workspace_id = wm.create_workspace();
    wm.active_workspace = first_workspace_id;

    let _wm = unsafe { transmute(Box::new(wm)) };
    _wm
}

#[no_mangle]
pub extern fn create_monitor(_wm: *mut WindowManager) -> u64 {
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
pub extern fn create_window(_wm: *mut WindowManager) -> u64 {
    let mut wm = unsafe { &mut *_wm };
    let id = wm.next_window_id;
    wm.next_window_id += 1;

    let window = Window { id, workspace: wm.active_workspace, ..Default::default() };
    wm.windows.insert(id, window);
    wm.workspaces.get_mut(&wm.active_workspace).unwrap().windows.push(id);

    println!("WM: {:?}", &wm);

    id
}

#[no_mangle]
pub extern fn destroy_window(_wm: *mut WindowManager, window_id: u64) -> IdArray {
    let wm = unsafe { &mut *_wm };

    let workspace = wm.workspaces.get_mut(&wm.get_workspace_id_by_window(window_id).unwrap()).unwrap();
    let workspace_id = workspace.id;
    
    let index = workspace.windows.iter().position(|id| *id == window_id).unwrap();
    workspace.windows.remove(index);
    wm.windows.remove(&window_id);

    IdArray::from_vec(wm.arrange_windows(workspace_id))
}

#[no_mangle]
pub extern fn arrange_windows(_wm: *mut WindowManager, window_id: u64) -> IdArray {
    let wm = unsafe { &mut *_wm };

    let workspace_id = wm.get_workspace_id_by_window(window_id).unwrap();

    IdArray::from_vec(wm.arrange_windows(workspace_id))
}

#[no_mangle]
pub extern fn configure_window(_wm: *mut WindowManager, window_id: u64, _geo: *mut wlr_box, app_id: *const c_char, fullscreen: bool) -> IdArray {
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

    IdArray::from_vec(wm.arrange_windows(workspace_id))
}

#[no_mangle]
pub extern fn update_window(_wm: *mut WindowManager, window_id: u64, _geo: *mut wlr_box) -> IdArray {
    let wm = unsafe { &mut *_wm };
    let geo = unsafe { &mut *_geo };

    let window = wm.windows.get(&window_id).unwrap();
    let workspace = wm.get_workspace_by_window(window_id).unwrap();
    let workspace_id = workspace.id;
    if window.is_tiled() {
        let x = wm.get_workspace_by_window(window_id).unwrap().windows.iter().fold(0, |sum, w| sum + wm.windows.get(w).unwrap().geo.width);
        geo.x = x;
        geo.y = 0;
        wm.windows.get_mut(&window_id).unwrap().geo = geo.clone();
    }

    println!("WM: {:?}", &wm);

    IdArray::from_vec(wm.arrange_windows(workspace_id))
}

#[no_mangle]
pub extern fn get_window_geometry(_wm: *mut WindowManager, window_id: u64) -> wlr_box {
    let wm = unsafe { &mut *_wm };
    println!("get_window_geometry: {:?}", &window_id);

    wm.windows.get(&window_id).unwrap().geo.clone()
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