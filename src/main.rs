#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use bevy::prelude::*;
// use bevy::time::Stopwatch;
use setup::cube::CubeModels;
use setup::camera::CameraComponent;

pub mod setup;

fn main() {
  App::new()
    .add_plugins(DefaultPlugins)
    .add_plugins((CameraComponent, CubeModels, VisualStyles))
    .run();
}

pub struct VisualStyles;

impl Plugin for VisualStyles {
  fn build(&self, app: &mut App) {
    app.insert_resource(AmbientLight {
      color: Color::WHITE,
      brightness: 500.0,
    });
    app.insert_resource(ClearColor(Color::srgb(0.5, 0.52, 0.55)));
  }
}
