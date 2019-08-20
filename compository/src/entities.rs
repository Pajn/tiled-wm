use crate::ffi::{wc_server, wlr_box};
use std::collections::BTreeMap;

pub type Id = u64;

#[derive(Default, Debug)]
pub struct Window {
  pub id: Id,
  pub app_id: String,
  pub fullscreen: bool,
  pub geo: wlr_box,
  pub workspace: Id,
}

impl Window {
  pub fn is_tiled(&self) -> bool {
    !self.fullscreen && self.app_id != "ulauncher"
  }
}

#[derive(Default, Debug)]
pub struct Workspace {
  pub id: Id,
  pub windows: Vec<Id>,
  pub active_monitor: Option<Id>,
}

#[derive(Debug)]
pub struct Monitor {
  pub id: Id,
  pub width: i32,
  pub height: i32,

  pub active_workspace: Id,
}

#[derive(Default, Debug)]
pub struct WindowManager {
  pub next_monitor_id: Id,
  pub next_workspace_id: Id,
  pub next_window_id: Id,

  pub focused_window: Option<Id>,
  pub active_workspace: Id,

  pub monitors: BTreeMap<Id, Monitor>,
  pub workspaces: BTreeMap<Id, Workspace>,
  pub windows: BTreeMap<Id, Window>,

  pub wc_server: Option<*mut wc_server>,
  pub focus_callback: Option<extern "C" fn(server: *mut wc_server, window_id: Id) -> ()>,
  pub dirty_window_callback: Option<extern "C" fn(server: *mut wc_server, window_id: Id) -> ()>,
}

impl WindowManager {
  pub fn create_workspace(&mut self) -> Id {
    let id = self.next_workspace_id;
    self.next_workspace_id += 1;

    let workspace = Workspace {
      id,
      ..Default::default()
    };
    self.workspaces.insert(id, workspace);

    id
  }

  pub fn get_workspace_by_window(&self, window_id: Id) -> Option<&Workspace> {
    self
      .windows
      .get(&window_id)
      .and_then(|window| self.workspaces.get(&window.workspace))
  }

  pub fn get_monitor_by_window(&self, window_id: Id) -> Option<&Monitor> {
    self
      .windows
      .get(&window_id)
      .and_then(|window| self.workspaces.get(&window.workspace))
      .and_then(|workspace| workspace.active_monitor)
      .and_then(|monitor| self.monitors.get(&monitor))
  }

  pub fn get_workspace_id_by_window(&self, window_id: Id) -> Option<Id> {
    self.windows.get(&window_id).map(|window| window.workspace)
  }

  pub fn get_unassigned_workspaces(&self) -> Vec<&Workspace> {
    self
      .workspaces
      .values()
      .filter(|w| w.active_monitor.is_none())
      .collect()
  }

  pub fn arrange_windows(&mut self, workspace_id: Id) -> () {
    let window_positions: Vec<_> = self
      .workspaces
      .get(&workspace_id)
      .unwrap()
      .windows
      .iter()
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
        if let (Some(server), Some(dirty_window_callback)) =
          (self.wc_server, self.dirty_window_callback)
        {
          dirty_window_callback(server, window.id);
        }
      }
    }
  }

  pub fn focus_window(&mut self, window_id: Id) -> () {
    self.focused_window = Some(window_id);
    if let (Some(server), Some(focus_callback)) = (self.wc_server, self.focus_callback) {
      focus_callback(server, window_id);
    }
  }
}
