use crate::geometry::*;
// use crate::input_inhibitor::{focus_exclusive_client, InputInhibitor};
use std::cmp;
use std::collections::BTreeMap;
use std::ffi::CStr;
use std::ptr;
use wlroots_rs::*;

pub type Id = u64;

#[derive(Debug)]
pub struct IdGenerator {
  next_id: Id,
}

impl IdGenerator {
  pub fn new() -> IdGenerator {
    IdGenerator { next_id: 1 }
  }

  pub fn next_id(&mut self) -> Id {
    let id = self.next_id;
    self.next_id = id + 1;
    id
  }
}

#[repr(C)]
#[derive(Debug)]
pub enum SurfaceType {
  Xdg {
    xdg_surface: *mut wlr_xdg_surface,
  },
  Xwayland {
    xwayland_surface: *mut wlr_xwayland_surface,
  },
}

impl Default for SurfaceType {
  fn default() -> Self {
    SurfaceType::Xdg {
      xdg_surface: ptr::null_mut(),
    }
  }
}

#[allow(non_camel_case_types)]
pub struct wc_server;
#[repr(C)]
#[derive(Debug)]
pub struct ServerPtr {
  pub server: *mut wc_server,
}

impl Default for ServerPtr {
  fn default() -> Self {
    ServerPtr {
      server: ptr::null_mut(),
    }
  }
}

#[repr(C)]
#[derive(Debug, Default)]
pub struct View {
  pub window_id: Id,
  pub link: wl_list,
  pub server: ServerPtr,
  pub surface_type: SurfaceType,

  pub mapped: bool,

  /* Current coordinates of the view.
   *
   * NOTE The width and height may not reflect what the client currently
   * thinks, but this is only temporary - when you change these you _must_
   * notify the client of its new size.
   */
  pub geo: wlr_box,

  // Serial for a pending move / resize.
  pub pending_serial: u32,
  pub is_pending_serial: bool,

  pub map: wl_listener,
  pub unmap: wl_listener,
  pub commit: wl_listener,
  pub destroy: wl_listener,
  pub request_move: wl_listener,
  pub request_resize: wl_listener,
  pub configure: wl_listener,
  pub state_change: wl_listener,
}

const MAX_SIZE: u32 = 999999;
const MIN_SIZE: u32 = 0;

impl View {
  pub fn app_id(&self) -> String {
    match self.surface_type {
      SurfaceType::Xdg { xdg_surface } => unsafe {
        match (*xdg_surface).role {
          wlr_xdg_surface_role::WLR_XDG_SURFACE_ROLE_TOPLEVEL => {
            CStr::from_ptr((*(*xdg_surface).__bindgen_anon_1.toplevel).app_id)
              .to_string_lossy()
              .to_string()
          }
          _ => "".to_owned(),
        }
      },
      _ => "".to_owned(),
    }
  }
  pub fn title(&self) -> String {
    match self.surface_type {
      SurfaceType::Xdg { xdg_surface } => unsafe {
        match (*xdg_surface).role {
          wlr_xdg_surface_role::WLR_XDG_SURFACE_ROLE_TOPLEVEL => {
            CStr::from_ptr((*(*xdg_surface).__bindgen_anon_1.toplevel).title)
              .to_string_lossy()
              .to_string()
          }
          _ => "".to_owned(),
        }
      },
      _ => "".to_owned(),
    }
  }

  pub fn is_toplevel(&self) -> bool {
    match self.surface_type {
      SurfaceType::Xdg { xdg_surface } => unsafe {
        match (*xdg_surface).role {
          wlr_xdg_surface_role::WLR_XDG_SURFACE_ROLE_TOPLEVEL => true,
          _ => false,
        }
      },
      _ => false,
    }
  }

  pub fn max_height(&self) -> u32 {
    match self.surface_type {
      SurfaceType::Xdg { xdg_surface } => unsafe {
        match (*xdg_surface).role {
          wlr_xdg_surface_role::WLR_XDG_SURFACE_ROLE_TOPLEVEL => {
            (*(*xdg_surface).__bindgen_anon_1.toplevel)
              .current
              .max_height
          }
          _ => MAX_SIZE,
        }
      },
      _ => MAX_SIZE,
    }
  }
  pub fn max_width(&self) -> u32 {
    match self.surface_type {
      SurfaceType::Xdg { xdg_surface } => unsafe {
        match (*xdg_surface).role {
          wlr_xdg_surface_role::WLR_XDG_SURFACE_ROLE_TOPLEVEL => {
            (*(*xdg_surface).__bindgen_anon_1.toplevel)
              .current
              .max_width
          }
          _ => MAX_SIZE,
        }
      },
      _ => MAX_SIZE,
    }
  }
  pub fn min_height(&self) -> u32 {
    match self.surface_type {
      SurfaceType::Xdg { xdg_surface } => unsafe {
        match (*xdg_surface).role {
          wlr_xdg_surface_role::WLR_XDG_SURFACE_ROLE_TOPLEVEL => {
            (*(*xdg_surface).__bindgen_anon_1.toplevel)
              .current
              .min_height
          }
          _ => MIN_SIZE,
        }
      },
      _ => MIN_SIZE,
    }
  }
  pub fn min_width(&self) -> u32 {
    match self.surface_type {
      SurfaceType::Xdg { xdg_surface } => unsafe {
        match (*xdg_surface).role {
          wlr_xdg_surface_role::WLR_XDG_SURFACE_ROLE_TOPLEVEL => {
            (*(*xdg_surface).__bindgen_anon_1.toplevel)
              .current
              .min_width
          }
          _ => MIN_SIZE,
        }
      },
      _ => MIN_SIZE,
    }
  }

  pub fn height(&self) -> u32 {
    // match self.surface_type {
    //   SurfaceType::Xdg { xdg_surface } => unsafe {
    //     (*(*xdg_surface).surface).current.height as u32
    //   },
    //   _ => MIN_SIZE,
    // }

    println!("Height {}:", self.app_id());
    println!("self.geo.height {}:", self.geo.height);
    match self.surface_type {
      SurfaceType::Xdg { xdg_surface } => unsafe {
        unsafe { println!("geometry {}:", (*xdg_surface).geometry.height) };
        unsafe {
          println!(
            "surface current {}:",
            (*(*xdg_surface).surface).current.height
          )
        };
        match (*xdg_surface).role {
          wlr_xdg_surface_role::WLR_XDG_SURFACE_ROLE_TOPLEVEL => {
            unsafe {
              println!(
                "toplevel current {}:",
                (*(*xdg_surface).__bindgen_anon_1.toplevel).current.height
              )
            };
          }
          _ => {}
        }
      },
      _ => {}
    }
    (self.geo.height - self.shadow_size().height) as u32
  }
  pub fn width(&self) -> u32 {
    (self.geo.width - self.shadow_size().width) as u32
  }

  pub fn shadow_size(&self) -> Size {
    match self.surface_type {
      SurfaceType::Xdg { xdg_surface } => unsafe {
        Size {
          width: ((*(*xdg_surface).surface).current.width - (*xdg_surface).geometry.width),
          height: ((*(*xdg_surface).surface).current.height - (*xdg_surface).geometry.height),
        }
      },
      _ => Size::default(),
    }
  }

  pub fn shadow_displacement(&self) -> Displacement {
    match self.surface_type {
      SurfaceType::Xdg { xdg_surface } => unsafe {
        Displacement {
          dx: ((*(*xdg_surface).surface).current.width - (*xdg_surface).geometry.width) / 2,
          dy: ((*(*xdg_surface).surface).current.height - (*xdg_surface).geometry.height) / 2,
        }
      },
      _ => Displacement::default(),
    }
  }

  pub fn is_fullscreen(&self) -> bool {
    match self.surface_type {
      SurfaceType::Xdg { xdg_surface } => unsafe {
        match (*xdg_surface).role {
          wlr_xdg_surface_role::WLR_XDG_SURFACE_ROLE_TOPLEVEL => {
            (*(*xdg_surface).__bindgen_anon_1.toplevel)
              .current
              .fullscreen
          }
          _ => false,
        }
      },
      _ => false,
    }
  }
}

#[derive(Debug)]
pub struct Window {
  pub id: Id,
  pub workspace: Id,
  pub workspace_geo: wlr_box,
  pub window_info: Box<View>,
  pub window_info_ptr: *mut View,
  pub is_dragged: bool,
}

impl Window {
  pub fn new(
    id_generator: &mut IdGenerator,
    workspace: Id,
    view: Box<View>,
    window_info_ptr: *mut View,
  ) -> Window {
    Window {
      id: id_generator.next_id(),
      workspace,
      workspace_geo: view.geo,
      window_info: view,
      window_info_ptr,
      is_dragged: false,
    }
  }

  pub fn x(&self) -> i32 {
    self.window_info.geo.x - self.window_info.shadow_displacement().dx
  }

  pub fn y(&self) -> i32 {
    self.window_info.geo.y - self.window_info.shadow_displacement().dy
  }

  pub fn height(&self) -> i32 {
    self.window_info.height() as i32
  }

  pub fn width(&self) -> i32 {
    self.window_info.width() as i32
  }

  pub fn rendered_top_left(&self) -> Point {
    Point {
      x: self.x(),
      y: self.y(),
    }
  }

  pub fn rendered_size(&self) -> Size {
    Size {
      width: self.width(),
      height: self.height(),
    }
  }

  pub fn max_height(&self) -> i32 {
    cmp::max(self.window_info.max_height() as i32, self.height() as i32)
  }

  pub fn min_height(&self) -> i32 {
    self.window_info.min_height() as i32
  }

  pub fn max_width(&self) -> i32 {
    cmp::max(self.window_info.max_width() as i32, self.width() as i32)
  }

  pub fn min_width(&self) -> i32 {
    self.window_info.min_width() as i32
  }

  pub fn resize(&mut self, size: Size) {
    println!("resize {}, {:?}", self.window_info.app_id(), size);
    // size.width = cmp::max(cmp::min(size.width, self.max_width()), self.min_width());
    // size.height = cmp::max(cmp::min(size.height, self.max_height()), self.min_height());
    let new_geo = self
      .window_info
      .geo
      .with_size(size + self.window_info.shadow_size());

    match self.window_info.surface_type {
      SurfaceType::Xdg { xdg_surface } => unsafe {
        self.window_info.pending_serial =
          wlr_xdg_toplevel_set_size(xdg_surface, new_geo.width as u32, new_geo.height as u32);
        self.window_info.is_pending_serial = true;
      },
      SurfaceType::Xwayland { xwayland_surface } => unsafe {
        self.window_info.pending_serial = 1;
        wlr_xwayland_surface_configure(
          xwayland_surface,
          new_geo.x as i16,
          new_geo.y as i16,
          new_geo.width as u16,
          new_geo.height as u16,
        );
      },
    }
  }

  pub fn move_to(&mut self, top_left: Point) {
    println!("move_to {}, {:?}", self.window_info.app_id(), top_left);
    println!("geo {:?}", self.window_info.geo);
    let top_left = top_left - self.window_info.shadow_displacement();
    println!("move_to displaced {:?}", top_left);
    unsafe { wc_view_damage_whole(self.window_info_ptr) }
    self.window_info.geo.set_top_left(top_left);
    unsafe { wc_view_damage_whole(self.window_info_ptr) }
  }

  pub fn is_tiled(&self) -> bool {
    println!(
      "is_tiled {}, {}, {}",
      self.window_info.app_id(),
      self.window_info.is_toplevel(),
      self.window_info.is_fullscreen()
    );
    self.window_info.app_id() != "ulauncher"
      && self.window_info.is_toplevel()
      && !self.window_info.is_fullscreen()
  }

  pub fn ask_client_to_close(&self, wm: &WindowManager) -> () {
    // unsafe { (*wm.tools).ask_client_to_close((*self.window_info).window()) };
    // TODO: Close
  }
}

#[derive(Debug)]
pub struct Workspace {
  pub id: Id,
  pub on_monitor: Option<Id>,
  pub scroll_left: i32,
  pub windows: Vec<Id>,
  pub active_window: Option<Id>,
}

impl Workspace {
  pub fn new(id_generator: &mut IdGenerator) -> Workspace {
    Workspace {
      id: id_generator.next_id(),
      on_monitor: None,
      scroll_left: 0,
      windows: vec![],
      active_window: None,
    }
  }

  pub fn get_tiled_windows(&self, wm: &WindowManager) -> Vec<Id> {
    self
      .windows
      .iter()
      .filter(|w| wm.get_window(**w).is_tiled())
      .copied()
      .collect()
  }

  pub fn get_window_index(&self, window: Id) -> Option<usize> {
    self
      .windows
      .iter()
      .enumerate()
      .find(|(_, w)| **w == window)
      .map(|(index, _)| index)
  }

  pub fn get_tiled_window_index(&self, wm: &WindowManager, window: Id) -> Option<usize> {
    self
      .get_tiled_windows(wm)
      .iter()
      .enumerate()
      .find(|(_, w)| **w == window)
      .map(|(index, _)| index)
  }

  pub fn swap_windows(&mut self, a: Id, b: Id) {
    let a_raw_index = self.get_window_index(a).unwrap();
    let b_raw_index = self.get_window_index(b).unwrap();
    self.windows.swap(a_raw_index, b_raw_index);
  }
}
#[repr(C)]
pub struct Ptr<T> {
  pub value: *mut T,
}

impl<T> Ptr<T> {
  pub fn new(value: *mut T) -> Self {
    Ptr { value }
  }
}

impl<T> Default for Ptr<T> {
  fn default() -> Self {
    Ptr {
      value: ptr::null_mut(),
    }
  }
}
impl<T> std::fmt::Debug for Ptr<T> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "Ptr {{ value: {:?} }}", self.value)
  }
}

#[repr(C)]
#[derive(Debug, Default)]
pub struct Output {
  pub link: wl_list,
  pub server: ServerPtr,

  pub wlr_output: Ptr<wlr_output>,
  pub damage: Ptr<wlr_output_damage>,

  pub layers: [wl_list; 4],

  pub destroy: wl_listener,
  pub frame: wl_listener,
}

#[derive(Debug)]
pub struct Monitor {
  pub id: Id,
  pub workspace: Id,
  pub output: Box<Output>,
}

impl Monitor {
  pub fn new(id_generator: &mut IdGenerator, workspace: Id, output: Box<Output>) -> Monitor {
    Monitor {
      id: id_generator.next_id(),
      // TODO: Remove
      workspace,
      output,
    }
  }

  pub fn extents(&self) -> wlr_box {
    wlr_box {
      x: 0,
      y: 0,
      width: unsafe { (*self.output.wlr_output.value).width },
      height: unsafe { (*self.output.wlr_output.value).height },
    }
  }
}

#[derive(Debug)]
pub struct ResizeGesture {
  pub window: Id,
  // pub buttons: raw::MirPointerButtons,
  // pub modifiers: input_event_modifier::Type,
  pub top_left: Point,
  pub size: Size,
  // pub edge: raw::MirResizeEdge::Type,
}

#[derive(Debug)]
pub struct MoveGesture {
  pub window: Id,
  // pub buttons: raw::MirPointerButtons,
  // pub modifiers: input_event_modifier::Type,
  pub top_left: Point,
}

#[derive(Debug)]
pub enum Gesture {
  Resize(ResizeGesture),
  Move(MoveGesture),
  None,
}

#[derive(Debug)]
pub struct WindowManager {
  pub server: *mut wc_server,
  // pub input_inhibitor: Box<InputInhibitor>,
  pub monitor_id_generator: IdGenerator,
  pub window_id_generator: IdGenerator,
  pub workspace_id_generator: IdGenerator,

  pub monitors: BTreeMap<Id, Monitor>,
  pub windows: BTreeMap<Id, Window>,
  pub workspaces: BTreeMap<Id, Workspace>,

  pub old_cursor: Point,
  pub gesture: Gesture,
  pub active_window: Option<Id>,
  pub active_workspace: Id,
  pub new_window_workspace: Id,
}

impl WindowManager {
  pub fn get_window(&self, window_id: Id) -> &Window {
    self
      .windows
      .get(&window_id)
      // .expect(format!("Window with id {} not found", window_id))
      .expect("Window with id {} not found")
  }

  pub fn get_workspace(&self, workspace_id: Id) -> &Workspace {
    self
      .workspaces
      .get(&workspace_id)
      // .expect(format!("Workspace with id {} not found", workspace_id))
      .expect("Workspace with id {} not found")
  }

  pub fn monitor_by_workspace(&self, workspace_id: Id) -> Option<&Monitor> {
    self
      .get_workspace(workspace_id)
      .on_monitor
      .and_then(|monitor_id| self.monitors.get(&monitor_id))
  }

  pub fn monitor_by_window(&self, window_id: Id) -> Option<&Monitor> {
    let workspace_id = self.get_window(window_id).workspace;
    self.monitor_by_workspace(workspace_id)
  }

  pub fn active_window(&self) -> Option<&Window> {
    self.active_window.and_then(|id| self.windows.get(&id))
  }

  pub fn active_workspace(&self) -> &Workspace {
    self
      .workspaces
      .get(&self.active_workspace)
      .expect("Active workspace not found")
  }

  pub fn new_window_workspace(&self) -> &Workspace {
    self
      .workspaces
      .get(&self.new_window_workspace)
      .expect("New window workspace not found")
  }

  pub fn get_or_create_unused_workspace(&mut self) -> Id {
    let unused_workspaces = self
      .workspaces
      .values()
      .filter(|w| w.on_monitor == None)
      .collect::<Vec<_>>();

    match unused_workspaces.first() {
      Option::None => {
        let first_workspace = Workspace::new(&mut self.workspace_id_generator);
        let first_workspace_id = first_workspace.id;
        self.workspaces.insert(first_workspace.id, first_workspace);
        let second_workspace = Workspace::new(&mut self.workspace_id_generator);
        self
          .workspaces
          .insert(second_workspace.id, second_workspace);

        first_workspace_id
      }
      Some(first_workspace) => {
        let first_workspace_id = first_workspace.id;

        // We want there to always be an additional workspace avalible
        if unused_workspaces.len() == 1 {
          let aditional_workspace = Workspace::new(&mut self.workspace_id_generator);
          self
            .workspaces
            .insert(aditional_workspace.id, aditional_workspace);
        }

        first_workspace_id
      }
    }
  }

  pub fn add_window(&mut self, window: Window) -> () {
    println!("WM: {:?}, adding: {:?}", &self, &window);
    let workspace = self.workspaces.get_mut(&window.workspace).unwrap();

    if let Some(index) = self
      .active_window
      .and_then(|active_window| workspace.get_window_index(active_window))
    {
      workspace.windows.insert(index + 1, window.id);
    } else {
      workspace.windows.push(window.id);
    }

    let window_id = window.id;
    self.windows.insert(window.id, window);

    let window = self.get_window(window_id);
    if window.window_info.is_toplevel() {
      // TODO: fix
      // if self.input_inhibitor.is_allowed(&window) {
      self.activate_window(window_id);
      // } else {
      //   focus_exclusive_client(self);
      // }
    }
  }

  pub fn delete_window(&mut self, window_id: Id) -> () {
    // TODO: fix
    // self.input_inhibitor.clear_if_dead();

    self
      .remove_window_from_workspace(window_id)
      .expect("nowindow in workspace advise_delete_window");
    self.windows.remove(&window_id);

    if self.active_window == Some(window_id) {
      // Mir will focus a new window for us so we can just unset
      // active_window and wait for the focus event
      self.active_window = None;
    }
  }

  pub fn activate_window(&mut self, window_id: Id) -> () {
    let workspace_id = self.get_window(window_id).workspace;
    let workspace = self.workspaces.get_mut(&workspace_id).unwrap();

    workspace.active_window = Some(window_id);
    self.active_window = Some(window_id);
    self.active_workspace = workspace_id;
  }

  pub fn remove_window_from_workspace(&mut self, window: Id) -> Result<(), ()> {
    let workspace = self.get_workspace(self.get_window(window).workspace);
    let workspace_id = workspace.id;
    if workspace.active_window == Some(window) {
      let active_window = self.get_window(workspace.active_window.unwrap());
      if active_window.is_tiled() {
        let tiled_index = workspace.get_tiled_window_index(self, window).ok_or(())?;
        let tiled_index = if tiled_index > 0 {
          tiled_index - 1
        } else {
          tiled_index + 1
        };
        let next_active_window = workspace.get_tiled_windows(self).get(tiled_index).copied();
        let workspace = self.workspaces.get_mut(&workspace_id).unwrap();
        workspace.active_window = next_active_window;
      } else {
        let next_active_window = workspace.get_tiled_windows(self).last().copied();
        let workspace = self.workspaces.get_mut(&workspace_id).unwrap();
        workspace.active_window = next_active_window;
      }
    }
    let workspace = self.workspaces.get_mut(&workspace_id).unwrap();
    let raw_index = workspace.get_window_index(window).ok_or(())?;
    workspace.windows.remove(raw_index);
    Ok(())
  }

  pub fn focus_window(&mut self, window_id: Id) -> () {
    self.active_window = Some(window_id);
    let window = self.get_window(window_id);

    // TODO: fix
    // if self.input_inhibitor.is_allowed(window) {
    println!("wc_focus_view {:?}", window.window_info_ptr);
    unsafe { wc_focus_view(window.window_info_ptr) };
    // } else {
    //   focus_exclusive_client(self);
    // }
  }
}

extern "C" {
  pub fn wc_focus_view(view: *mut View) -> ();
  pub fn wc_view_damage_whole(view: *mut View) -> ();
}
