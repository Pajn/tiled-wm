use crate::entities::Id;

#[allow(non_camel_case_types)]
pub struct wc_server {}

#[allow(non_camel_case_types)]
#[derive(Default, Debug, Clone)]
pub struct wlr_box {
  pub x: i32,
  pub y: i32,
  pub width: i32,
  pub height: i32,
}

#[derive(Debug)]
#[repr(C)]
pub struct IdArray {
  pub data: *mut Id,
  pub length: usize,
  pub capacity: usize,
}

impl IdArray {
  pub fn from_vec(mut vec: Vec<Id>) -> IdArray {
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
