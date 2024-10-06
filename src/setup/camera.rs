use bevy::prelude::*;

use bevy::input::mouse::{MouseMotion, MouseScrollUnit, MouseWheel, MouseButton};
use std::f32::consts::{FRAC_PI_2, PI, TAU};

pub struct CameraComponent;

impl Plugin for CameraComponent {
  fn build(&self, app: &mut App) {
    app.add_systems(Startup, setup_camera);
    app.add_systems(Update, (
      orbit_camera_control.run_if(any_with_component::<OrbitState>),
      reset_camera_angle.run_if(any_with_component::<OrbitState>),
    ));
  }
}

const DEFAULT_RADIUS: f32 = 30.0;
const DEFAULT_PITCH: f32 = -45.0;
const DEFAULT_YAW: f32 = 45.0;

const MIN_ZOOM: f32 = 60.0;
const MAX_ZOOM: f32 = 15.0;

#[derive(Bundle, Default)]
pub struct OrbitCameraBundle {
  pub camera: Camera3dBundle,
  pub state: OrbitState,
  pub settings: OrbitSettings,
}
#[derive(Component)]
pub struct OrbitState {
  pub center: Vec3,
  pub radius: f32,
  pub upside_down: bool,
  pub pitch: f32,
  pub yaw: f32,
}

#[derive(Component)]
pub struct OrbitSettings {

  pub orbit_sensitivity: f32, /// Radians per pixel of mouse motion
  pub zoom_sensitivity: f32, /// Exponent per pixel of mouse motion

  pub reset_rotate: Option<KeyCode>,
  pub reset_rotate_m: Option<MouseButton>,

  pub rotate_left: Option<KeyCode>,
  pub rotate_right: Option<KeyCode>,
  pub rotate_up: Option<KeyCode>,
  pub rotate_down: Option<KeyCode>,

  pub orbit_key: Option<MouseButton>,
  pub scroll_action: Option<OrbitAction>,
  
  pub scroll_line_sensitivity: f32, // notched scroll wheel (desktops)
  pub scroll_pixel_sensitivity: f32, // smooth scroll (touchpads)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OrbitAction {
  Orbit,
  Zoom,
}

impl Default for OrbitState {
  fn default() -> Self {
    OrbitState {
      center: Vec3::ZERO, // always 0,0,0 for this app
      radius: DEFAULT_RADIUS,
      upside_down: false,
      pitch: DEFAULT_PITCH.to_radians(),
      yaw: DEFAULT_YAW.to_radians(),
    }
  }
}

impl Default for OrbitSettings {
  fn default() -> Self {
    OrbitSettings {
      orbit_sensitivity: 0.5f32.to_radians(), // 0.1 degree per pixel
      zoom_sensitivity: 0.01,
      reset_rotate: Some(KeyCode::Home),
      reset_rotate_m: Some(MouseButton::Right),
      rotate_left: Some(KeyCode::Delete),
      rotate_right: Some(KeyCode::Insert),
      rotate_up: Some(KeyCode::PageUp),
      rotate_down: Some(KeyCode::PageDown),
      orbit_key: Some(MouseButton::Middle),
      scroll_action: Some(OrbitAction::Zoom),
      scroll_line_sensitivity: 16.0, // 1 "line" == 16 "pixels of motion"
      scroll_pixel_sensitivity: 1.0,
    }
  }
}

pub fn setup_camera(mut commands: Commands) {
  commands.spawn(OrbitCameraBundle::default());
}


pub fn orbit_camera_control(
  mouse: Res<ButtonInput<MouseButton>>,
  kbd: Res<ButtonInput<KeyCode>>,
  mut evr_motion: EventReader<MouseMotion>,
  mut evr_scroll: EventReader<MouseWheel>,
  mut q_camera: Query<(
    &OrbitSettings,
    &mut OrbitState,
    &mut Transform,
  )>,
) {
  // First, accumulate the total amount of
  // mouse motion and scroll, from all pending events:
  let mut total_motion: Vec2 = evr_motion.read()
    .map(|ev| ev.delta).sum();

  // Reverse Y (Bevy's Worldspace coordinate system is Y-Up, but events are in window/ui coordinates, which are Y-Down)
  total_motion.y = -total_motion.y;

  let mut total_scroll_lines = Vec2::ZERO;
  let mut total_scroll_pixels = Vec2::ZERO;
  for ev in evr_scroll.read() {
    match ev.unit {
      MouseScrollUnit::Line => {
        total_scroll_lines.x += ev.x;
        total_scroll_lines.y -= ev.y;
      }
      MouseScrollUnit::Pixel => {
        total_scroll_pixels.x += ev.x;
        total_scroll_pixels.y -= ev.y;
      }
    }
  }

  let (settings, mut state, mut transform) = q_camera.single_mut();
  // Check how much of each thing we need to apply.
  // Accumulate values from motion and scroll,
  // based on our configuration settings.
  {
    let mut total_orbit = Vec2::ZERO;
    if settings.orbit_key.map(|mb| mouse.pressed(mb)).unwrap_or(false) {
      total_orbit -= total_motion * settings.orbit_sensitivity;
    }

    if settings.rotate_left.map(|key| kbd.pressed(key)).unwrap_or(false) {
      total_orbit.x += 0.06;
    }
    if settings.rotate_right.map(|key| kbd.pressed(key)).unwrap_or(false) {
      total_orbit.x -= 0.06;
    }

    if settings.rotate_up.map(|key| kbd.pressed(key)).unwrap_or(false) {
      total_orbit.y += 0.06;
    }
    if settings.rotate_down.map(|key| kbd.pressed(key)).unwrap_or(false) {
      total_orbit.y -= 0.06;
    }

    let mut total_zoom = Vec2::ZERO;

    if settings.scroll_action == Some(OrbitAction::Zoom) {
      total_zoom -= total_scroll_lines * settings.scroll_line_sensitivity * settings.zoom_sensitivity;
      total_zoom -= total_scroll_pixels * settings.scroll_pixel_sensitivity * settings.zoom_sensitivity;
    }

    // Upon starting a new orbit maneuver, check if we are starting it upside-down
    if settings.orbit_key.map(|mb| mouse.just_pressed(mb)).unwrap_or(false) {
      state.upside_down = state.pitch < -FRAC_PI_2 || state.pitch > FRAC_PI_2;
    }
    if state.upside_down { total_orbit.x = -total_orbit.x; }

    let mut trans = false;

    if total_zoom != Vec2::ZERO {
      trans = true;
      state.radius *= (-total_zoom.y).exp();
    }

    // To ORBIT, we change our pitch and yaw values
    if total_orbit != Vec2::ZERO {
      trans = true;
      state.yaw += total_orbit.x;
      state.pitch += total_orbit.y;
      // wrap around, to stay between +- 180 degrees
      if state.yaw > PI { state.yaw -= TAU; }
      if state.yaw < -PI { state.yaw += TAU; }
      if state.pitch > PI { state.pitch -= TAU; }
      if state.pitch < -PI { state.pitch += TAU; }
    }

    // Finally, compute the new camera transform (if we changed anything, or if the orbit
    // controller was just added and thus we are running or the first time and need to initialize)
    if trans || state.is_added() {
      // YXZ Euler Rotation performs yaw/pitch/roll.
      transform.rotation = Quat::from_euler(EulerRot::YXZ, state.yaw, state.pitch, 0.0);
      // To position the camera, get the backward direction vector and place the camera at the desired radius from the center.
      if state.radius > MIN_ZOOM { state.radius = MIN_ZOOM }
      else if state.radius < MAX_ZOOM { state.radius = MAX_ZOOM };
      transform.translation = state.center + transform.back() * state.radius;
    }

  }
  
}

pub fn reset_camera_angle(
  mouse: Res<ButtonInput<MouseButton>>,
  kbd: Res<ButtonInput<KeyCode>>,
  mut q_camera: Query<(
    &OrbitSettings,
    &mut OrbitState,
    &mut Transform,
  )>,
) {

  let (settings, mut state, mut transform) = q_camera.single_mut();
    
  if settings.reset_rotate_m.map(|mb| mouse.pressed(mb)).unwrap_or(false) ||
    settings.reset_rotate.map(|key| kbd.pressed(key)).unwrap_or(false) || 
    state.is_added() {
    state.pitch = DEFAULT_PITCH.to_radians();
    state.yaw = DEFAULT_YAW.to_radians();
    
    state.radius = DEFAULT_RADIUS;

    transform.rotation = Quat::from_euler(EulerRot::YXZ, state.yaw, state.pitch, 0.0);
    transform.translation = state.center + transform.back() * state.radius;
  }
}