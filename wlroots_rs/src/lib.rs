#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use std::ptr;

// include!(concat!(env!("OUT_DIR"), "/wlroots.rs"));
include!("wlroots.rs");

impl Default for wl_list {
  fn default() -> Self {
    wl_list {
      prev: ptr::null_mut(),
      next: ptr::null_mut(),
    }
  }
}

impl Default for wl_listener {
  fn default() -> Self {
    wl_listener {
      link: Default::default(),
      notify: None,
    }
  }
}

impl Default for wlr_box {
  fn default() -> Self {
    wlr_box {
      x: 0, y: 0,
      width: 0, height: 0,
    }
  }
}