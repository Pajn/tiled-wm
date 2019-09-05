use crate::actions::*;
use crate::entities::*;
use crate::geometry::*;
use std::process::Command;
use xkbcommon::xkb;

pub fn handle_key_press(
  wm: &mut WindowManager,
  xkb_state: &xkb::State,
  keycode: xkb::Keycode,
) -> bool {
  let keysyms = xkb_state.key_get_syms(keycode);

  // if wm.input_inhibitor.is_inhibited() {
  //   return false;
  // }

  let modifiers = |modifiers: &[&str]| {
    let all_modifiers = [
      xkb::MOD_NAME_ALT,
      xkb::MOD_NAME_CAPS,
      xkb::MOD_NAME_CTRL,
      xkb::MOD_NAME_LOGO,
      xkb::MOD_NAME_NUM,
      xkb::MOD_NAME_SHIFT,
    ];

    all_modifiers.iter().all(|modifier| {
      xkb_state.mod_name_is_active(modifier, xkb::STATE_MODS_DEPRESSED)
        == modifiers.contains(modifier)
    })
  };

  keysyms.iter().any(|keysym| match *keysym {
    xkb::KEY_Home if modifiers(&[xkb::MOD_NAME_LOGO]) => {
      naviate_first(wm);
      true
    }
    xkb::KEY_End if modifiers(&[xkb::MOD_NAME_LOGO]) => {
      naviate_last(wm);
      true
    }
    xkb::KEY_Left if modifiers(&[xkb::MOD_NAME_LOGO]) => {
      naviate(wm, Direction::Left);
      true
    }
    xkb::KEY_Right if modifiers(&[xkb::MOD_NAME_LOGO]) => {
      naviate(wm, Direction::Right);
      true
    }
    xkb::KEY_Left if modifiers(&[xkb::MOD_NAME_LOGO, xkb::MOD_NAME_CTRL]) => {
      move_window(wm, Direction::Left);
      true
    }
    xkb::KEY_Right if modifiers(&[xkb::MOD_NAME_LOGO, xkb::MOD_NAME_CTRL]) => {
      move_window(wm, Direction::Right);
      true
    }
    xkb::KEY_Left if modifiers(&[xkb::MOD_NAME_LOGO, xkb::MOD_NAME_SHIFT]) => {
      naviate_monitor(wm, Direction::Left, Activation::LastActive);
      true
    }
    xkb::KEY_Right if modifiers(&[xkb::MOD_NAME_LOGO, xkb::MOD_NAME_SHIFT]) => {
      naviate_monitor(wm, Direction::Right, Activation::LastActive);
      true
    }
    xkb::KEY_Left if modifiers(&[xkb::MOD_NAME_LOGO, xkb::MOD_NAME_CTRL, xkb::MOD_NAME_SHIFT]) => {
      move_window_monitor(wm, Direction::Left, Activation::LastActive);
      true
    }
    xkb::KEY_Right if modifiers(&[xkb::MOD_NAME_LOGO, xkb::MOD_NAME_CTRL, xkb::MOD_NAME_SHIFT]) => {
      move_window_monitor(wm, Direction::Right, Activation::LastActive);
      true
    }
    xkb::KEY_a if modifiers(&[xkb::MOD_NAME_LOGO]) => {
      Command::new("ulauncher-toggle")
        .spawn()
        .expect("failed to execute process");
      true
    }
    // xkb::KEY_r if modifiers(&[xkb::MOD_NAME_LOGO]) => {
    //   if let Some(active_window) = wm.active_window {
    //     if let Some(monitor) = wm.monitor_by_window(active_window) {
    //       let monitor_width = monitor.extents().width();
    //       let window_width = wm.get_window(active_window).size.width;

    //       let window = wm.windows.get_mut(&active_window).unwrap();
    //       if window_width < monitor_width / 3 {
    //         window.set_size(window.size.with_width(monitor_width / 3));
    //       } else if window_width < monitor_width / 2 {
    //         window.set_size(window.size.with_width(monitor_width / 2));
    //       } else if window_width < ((monitor_width / 3) * 2) {
    //         window.set_size(window.size.with_width((monitor_width / 3) * 2));
    //       } else {
    //         window.set_size(window.size.with_width(monitor_width / 3));
    //       }
    //       arrange_windows(wm);
    //     } else {
    //       println!("Active window not on a monitor?");
    //     }
    //   }
    //   true
    // }
    // xkb::KEY_f if modifiers(&[xkb::MOD_NAME_LOGO]) => {
    //   if let Some(active_window) = wm.active_window {
    //     if let Some(monitor) = wm.monitor_by_window(active_window) {
    //       let monitor_width = monitor.extents().width();

    //       let window = wm.windows.get_mut(&active_window).unwrap();
    //       window.set_size(window.size.with_width(monitor_width));
    //       arrange_windows(wm);
    //     } else {
    //       println!("Active window not on a monitor?");
    //     }
    //   }
    //   true
    // }
    // xkb::KEY_c if modifiers(&[xkb::MOD_NAME_LOGO]) => {
    //   if let Some(active_window) = wm.active_window {
    //     if let Some(monitor) = wm.monitor_by_window(active_window) {
    //       let monitor_left = monitor.extents().left();
    //       let monitor_width = monitor.extents().width();
    //       let window = wm.get_window(active_window);

    //       if window.is_tiled() {
    //         let worksapce_id = window.workspace;

    //         let scroll_left =
    //           window.workspace_geo.x - monitor_left - monitor_width / 2 + window.size.width / 2;

    //         let workspace = wm.workspaces.get_mut(&worksapce_id).unwrap();
    //         workspace.scroll_left = scroll_left;

    //         arrange_windows(wm);
    //       }
    //     } else {
    //       println!("Active window not on a monitor?");
    //     }
    //   }
    //   true
    // }
    xkb::KEY_BackSpace if modifiers(&[xkb::MOD_NAME_LOGO]) => {
      if let Some(active_window) = wm.active_window {
        let window = wm.get_window(active_window);
        window.ask_client_to_close(wm);
      }
      true
    }
    _ => false,
  })
}
