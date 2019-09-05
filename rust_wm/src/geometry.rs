use std::ops::{Add, Sub};
use wlroots_rs::*;

#[repr(C)]
#[derive(Debug, Default, PartialEq, Eq, Clone, Copy)]
pub struct Point {
  pub x: i32,
  pub y: i32,
}

impl Point {
  pub fn x(&self) -> i32 {
    self.x
  }

  pub fn y(&self) -> i32 {
    self.y
  }
}

#[repr(C)]
#[derive(Debug, Default, PartialEq, Eq, Clone, Copy)]
pub struct Size {
  pub width: i32,
  pub height: i32,
}

impl Size {
  pub fn width(&self) -> i32 {
    self.width
  }

  pub fn height(&self) -> i32 {
    self.height
  }

  pub fn with_width(&self, width: i32) -> Size {
    Size {
      width,
      height: self.height,
    }
  }

  pub fn with_height(&self, height: i32) -> Size {
    Size {
      width: self.width,
      height,
    }
  }
}

impl Add<Size> for Size {
  type Output = Size;

  fn add(self, other: Size) -> Self::Output {
    Size {
      width: self.width + other.width,
      height: self.height + other.height,
    }
  }
}

impl Sub<Size> for Size {
  type Output = Size;

  fn sub(self, other: Size) -> Self::Output {
    Size {
      width: self.width - other.width,
      height: self.height - other.height,
    }
  }
}

// #[repr(C)]
// #[derive(Debug, Default, PartialEq, Eq, Clone)]
// pub struct Rectangle {
//   pub top_left: Point,
//   pub size: Size,
// }

pub trait Rectangle {
  fn left(&self) -> i32;
  fn top(&self) -> i32;
  fn right(&self) -> i32;
  fn bottom(&self) -> i32;
  fn width(&self) -> i32;
  fn height(&self) -> i32;
  fn top_left(&self) -> Point;
  fn bottom_right(&self) -> Point;
  fn size(&self) -> Size;
  fn set_top_left(&mut self, top_left: Point);
  fn set_size(&mut self, size: Size);
  fn with_size(&mut self, size: Size) -> Self;
  fn contains(&self, point: &Point) -> bool;
}

impl Rectangle for wlr_box {
  fn left(&self) -> i32 {
    self.x
  }

  fn top(&self) -> i32 {
    self.y
  }

  fn right(&self) -> i32 {
    self.left() + self.width()
  }

  fn bottom(&self) -> i32 {
    self.top() + self.height()
  }

  fn width(&self) -> i32 {
    self.width
  }

  fn height(&self) -> i32 {
    self.height
  }

  fn top_left(&self) -> Point {
    Point {
      x: self.left(),
      y: self.top(),
    }
  }

  fn bottom_right(&self) -> Point {
    Point {
      x: self.right(),
      y: self.bottom(),
    }
  }

  fn set_top_left(&mut self, top_left: Point) {
    self.x = top_left.x;
    self.y = top_left.y;
  }

  fn size(&self) -> Size {
    Size {
      width: self.width,
      height: self.height,
    }
  }

  fn set_size(&mut self, size: Size) {
    self.width = size.width;
    self.height = size.height;
  }

  fn with_size(&mut self, size: Size) -> Self {
    let mut new_geo = self.clone();
    new_geo.set_size(size);
    new_geo
  }

  fn contains(&self, point: &Point) -> bool {
    self.left() <= point.x
      && self.right() > point.x
      && self.top() <= point.y
      && self.bottom() > point.y
  }
}

#[repr(C)]
#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub struct Displacement {
  pub dx: i32,
  pub dy: i32,
}

impl Sub<Point> for Point {
  type Output = Displacement;

  fn sub(self, other: Self) -> Self::Output {
    Displacement {
      dx: self.x - other.x,
      dy: self.y - other.y,
    }
  }
}

impl Add<Displacement> for Point {
  type Output = Point;

  fn add(self, other: Displacement) -> Self::Output {
    Point {
      x: self.x + other.dx,
      y: self.y + other.dy,
    }
  }
}

impl Sub<Displacement> for Point {
  type Output = Point;

  fn sub(self, other: Displacement) -> Self::Output {
    Point {
      x: self.x - other.dx,
      y: self.y - other.dy,
    }
  }
}
