use crate::entities::WindowManager;
use std::cmp;
use std::collections::BTreeMap;
use std::ffi::CString;
use std::process::Command;
use xkb::Keysym;
use xkbcommon_sys::{
  xkb_keysym_t, xkb_state, xkb_state_mod_name_is_active, XKB_STATE_MODS_DEPRESSED,
};

#[repr(u8)]
#[derive(Debug, PartialEq, Eq)]
pub enum KeyState {
  #[allow(dead_code)]
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

  expected_mod_state
    .into_iter()
    .all(|(modifier, expected_state)| {
      let mod_name = CString::new(match modifier {
        Modifier::Control => "Control",
        Modifier::Alt => "Alt",
        Modifier::Shift => "Shift",
      })
      .unwrap();

      unsafe {
        (xkb_state_mod_name_is_active(state, mod_name.as_ptr(), XKB_STATE_MODS_DEPRESSED) != 0)
          == expected_state
      }
    })
}

#[no_mangle]
pub extern "C" fn keyboard_on_key(
  _wm: *mut WindowManager,
  keysym: xkb_keysym_t,
  key_state: KeyState,
  xkb_state: *mut xkb_state,
) -> bool {
  if key_state != KeyState::Pressed {
    return false;
  }

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
        let index = cmp::max(
          1,
          workspace
            .windows
            .iter()
            .position(|id| *id == focused_window)
            .unwrap(),
        );
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
        let index = cmp::min(
          workspace.windows.len() - 2,
          workspace
            .windows
            .iter()
            .position(|id| *id == focused_window)
            .unwrap(),
        );
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
        let workspace = wm
          .workspaces
          .get_mut(&wm.get_workspace_id_by_window(focused_window).unwrap())
          .unwrap();
        let index = workspace
          .windows
          .iter()
          .position(|id| *id == focused_window)
          .unwrap();

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
        let workspace = wm
          .workspaces
          .get_mut(&wm.get_workspace_id_by_window(focused_window).unwrap())
          .unwrap();
        let index = workspace
          .windows
          .iter()
          .position(|id| *id == focused_window)
          .unwrap();

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
    _ => false,
  }
}
