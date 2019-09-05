// use crate::entities::*;
// use std::mem::transmute;
// use wayland_server::sys::server::wl_client;

// #[derive(Debug)]
// pub struct InputInhibitor {
//   exclusive_client: Option<*const wl_client>,
// }

// #[no_mangle]
// pub extern "C" fn input_inhibitor_new() -> *mut InputInhibitor {
//   let inhibitor = InputInhibitor {
//     exclusive_client: None,
//   };

//   unsafe { transmute(Box::new(inhibitor)) }
// }

// #[no_mangle]
// pub extern "C" fn input_inhibitor_is_inhibited(inhibitor: *const InputInhibitor) -> bool {
//   let inhibitor = unsafe { &*inhibitor };
//   inhibitor.is_inhibited()
// }

// #[no_mangle]
// pub extern "C" fn input_inhibitor_set(
//   inhibitor: *mut InputInhibitor,
//   exclusive_client: *const wl_client,
// ) -> () {
//   let inhibitor = unsafe { &mut *inhibitor };
//   inhibitor.exclusive_client = Some(exclusive_client);
// }

// #[no_mangle]
// pub extern "C" fn input_inhibitor_clear(inhibitor: *mut InputInhibitor) -> () {
//   let inhibitor = unsafe { &mut *inhibitor };
//   inhibitor.exclusive_client = None;
// }

// impl InputInhibitor {
//   pub fn is_inhibited(&self) -> bool {
//     self.exclusive_client.is_some()
//   }
//   pub fn is_allowed(&self, window: &Window) -> bool {
//     match self.exclusive_client {
//       Some(exclusive_client) => {
//         let is_owned = unsafe { client_owns_window(exclusive_client, window.window_info) };
//         is_owned
//       }
//       None => true,
//     }
//   }
//   pub fn clear_if_dead(&mut self) {
//     if let Some(exclusive_client) = self.exclusive_client {
//       let is_alive = unsafe { client_is_alive(exclusive_client) };
//       if !is_alive {
//         self.exclusive_client = None;
//       }
//     }
//   }
// }

// pub fn focus_exclusive_client(wm: &mut WindowManager) -> () {
//   wm.focus_window(
//     wm.windows
//       .values()
//       .find(|window| wm.input_inhibitor.is_allowed(window))
//       .map(|window| window.id),
//   );
// }
