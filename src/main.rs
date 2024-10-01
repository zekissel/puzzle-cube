#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use bevy::prelude::*;

// use bevy::time::Stopwatch;
use setup::camera::orbit_camera_control;
use setup::camera::reset_camera_angle;
use setup::camera::OrbitState;

use setup::cube::*;

use setup::camera::setup_camera;
pub mod setup;

fn main() {
  App::new()
    .add_plugins(DefaultPlugins)
    .add_plugins(CubeModels)
    .add_systems(Startup, setup_camera)
    .add_systems(Update, (
        orbit_camera_control.run_if(any_with_component::<OrbitState>),
        reset_camera_angle.run_if(any_with_component::<OrbitState>),
        front_counter.run_if(any_with_component::<Block>),
        front_clockwise.run_if(any_with_component::<Block>),
        top_counter.run_if(any_with_component::<Block>),
        top_clockwise.run_if(any_with_component::<Block>),
      ),
    )
    .insert_resource(AmbientLight {
      color: Color::WHITE,
      brightness: 500.0,
    })
    .run();
}


pub struct CubeModels;

impl Plugin for CubeModels {
  fn build(&self, app: &mut App) {
    app.add_systems(Startup, (setup_core, setup_edges, setup_corners));
    
  }
}