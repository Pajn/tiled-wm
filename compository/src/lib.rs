mod entities;
mod ffi;
mod keyboard;

use crate::entities::*;
use crate::ffi::{wc_server, wlr_box};
use std::ffi::CStr;
use std::mem::transmute;
use std::os::raw::c_char;

#[no_mangle]
pub extern "C" fn create_window_manager(server: *mut wc_server) -> *mut WindowManager {
  let mut wm = WindowManager::default();
  wm.wc_server = Some(server);
  let first_workspace_id = wm.create_workspace();
  wm.active_workspace = first_workspace_id;

  let _wm = unsafe { transmute(Box::new(wm)) };
  _wm
}

#[no_mangle]
pub extern "C" fn register_focus_callback(
  _wm: *mut WindowManager,
  callback: extern "C" fn(server: *mut wc_server, window_id: Id) -> (),
) -> () {
  let mut wm = unsafe { &mut *_wm };

  wm.focus_callback = Some(callback);
}

#[no_mangle]
pub extern "C" fn register_dirty_window_callback(
  _wm: *mut WindowManager,
  callback: extern "C" fn(server: *mut wc_server, window_id: Id) -> (),
) -> () {
  let mut wm = unsafe { &mut *_wm };

  wm.dirty_window_callback = Some(callback);
}

#[no_mangle]
pub extern "C" fn create_monitor(_wm: *mut WindowManager, width: i32, height: i32) -> Id {
  let mut wm = unsafe { &mut *_wm };
  let id = wm.next_monitor_id;
  wm.next_monitor_id += 1;

  let unassigned_workspaces = wm.get_unassigned_workspaces();

  let workspace_id = if unassigned_workspaces.len() == 0 {
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

  let monitor = Monitor {
    id,
    width,
    height,
    active_workspace: workspace_id,
  };
  wm.monitors.insert(id, monitor);
  wm.workspaces.get_mut(&workspace_id).unwrap().active_monitor = Some(id);

  println!("create_monitor WM: {:?}", &wm);

  id
}

#[no_mangle]
pub extern "C" fn create_window(_wm: *mut WindowManager) -> Id {
  let mut wm = unsafe { &mut *_wm };
  let id = wm.next_window_id;
  wm.next_window_id += 1;

  let window = Window {
    id,
    workspace: wm.active_workspace,
    ..Default::default()
  };
  wm.windows.insert(id, window);
  wm.workspaces
    .get_mut(&wm.active_workspace)
    .unwrap()
    .windows
    .push(id);
  wm.focused_window = Some(id);

  println!("create_window WM: {:?}", &wm);

  id
}

#[no_mangle]
pub extern "C" fn destroy_window(_wm: *mut WindowManager, window_id: Id) -> () {
  let wm = unsafe { &mut *_wm };

  let workspace = wm
    .workspaces
    .get_mut(&wm.get_workspace_id_by_window(window_id).unwrap())
    .unwrap();
  let workspace_id = workspace.id;

  let index = workspace
    .windows
    .iter()
    .position(|id| *id == window_id)
    .unwrap();

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
pub extern "C" fn arrange_windows(_wm: *mut WindowManager, window_id: Id) -> () {
  let wm = unsafe { &mut *_wm };

  let workspace_id = wm.get_workspace_id_by_window(window_id).unwrap();

  wm.arrange_windows(workspace_id);
}

#[no_mangle]
pub extern "C" fn configure_window(
  _wm: *mut WindowManager,
  window_id: Id,
  _geo: *mut wlr_box,
  app_id: *const c_char,
  fullscreen: bool,
) -> bool {
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

  let is_tiled = window.is_tiled();
  if is_tiled {
    let x = wm
      .get_workspace_by_window(window_id)
      .unwrap()
      .windows
      .iter()
      .fold(0, |sum, w| sum + wm.windows.get(w).unwrap().geo.width);
    let monitor = wm.get_monitor_by_window(window_id).unwrap();
    geo.x = x;
    geo.y = 0;
    geo.height = monitor.height;
    wm.windows.get_mut(&window_id).unwrap().geo = geo.clone();
  } else if window.app_id == "ulauncher" {
    let monitor = wm.get_monitor_by_window(window_id).unwrap();
    geo.x = (monitor.width - geo.width) / 2;
    geo.y = (monitor.height - geo.height) / 2;
  }

  println!("configure_window WM: {:?}", &wm);

  wm.arrange_windows(workspace_id);

  is_tiled
}

#[no_mangle]
pub extern "C" fn update_window(_wm: *mut WindowManager, window_id: Id, _geo: *mut wlr_box) -> () {
  let wm = unsafe { &mut *_wm };
  let geo = unsafe { &mut *_geo };

  println!("update_window geo: {:?}", &geo);

  let window = wm.windows.get(&window_id).unwrap();
  let workspace = wm.get_workspace_by_window(window_id).unwrap();
  let workspace_id = workspace.id;
  if window.is_tiled() {
    let x = wm
      .get_workspace_by_window(window_id)
      .unwrap()
      .windows
      .iter()
      .take_while(|w| **w != window_id)
      .fold(0, |sum, w| sum + wm.windows.get(w).unwrap().geo.width);
    let monitor = wm.get_monitor_by_window(window_id).unwrap();

    geo.x = x;
    geo.y = 0;
    geo.height = monitor.height;
    wm.windows.get_mut(&window_id).unwrap().geo = geo.clone();
  }

  println!("update_window WM: {:?}", &wm);

  wm.arrange_windows(workspace_id);
}

#[no_mangle]
pub extern "C" fn get_window_geometry(_wm: *mut WindowManager, window_id: Id) -> wlr_box {
  let wm = unsafe { &mut *_wm };
  println!("get_window_geometry: {:?}", &window_id);

  wm.windows.get(&window_id).unwrap().geo.clone()
}

#[no_mangle]
pub extern "C" fn focus_window(_wm: *mut WindowManager, window_id: Id) -> () {
  let wm = unsafe { &mut *_wm };
  println!("focus_window: {:?}", &window_id);

  wm.focused_window = Some(window_id);
}

#[cfg(test)]
mod tests {
  #[test]
  fn it_works() {
    assert_eq!(2 + 2, 4);
  }
}
