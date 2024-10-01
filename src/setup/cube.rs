use bevy::prelude::*;

#[derive(Component)]
pub struct Block;


#[derive(Component)]
pub struct Xpos(f32);

#[derive(Component)]
pub struct Ypos(f32);

#[derive(Component)]
pub struct Zpos(f32);

#[derive(Component)]
pub struct CubeSettings {
  pub rotate_x: Option<KeyCode>,
}

impl Default for CubeSettings {
  fn default() -> Self {
    CubeSettings {
      rotate_x: Some(KeyCode::KeyW),
    }
  }
}

enum UnpackBlocks {
  Center,
  Edge,
  Corner,
}

fn unpack_coords(name: &str, area: UnpackBlocks) -> (f32, f32, f32) {

  let mut index = 0;

  let x_trans = match name.chars().nth(index).unwrap() {
    'w' => 2.2,
    'y' => -2.2,
    _ => 0.0,
  };

  index = match area {
    UnpackBlocks::Center => 0,
    UnpackBlocks::Edge => { if x_trans != 0.0 { index + 1 } else { index } },
    UnpackBlocks::Corner => 1,
  };

  let y_trans = match name.chars().nth(index).unwrap() {
    'r' => 2.2,
    'o' => -2.2,
    _ => 0.0,
  };

  index = match area {
    UnpackBlocks::Center => 0,
    UnpackBlocks::Edge => { if y_trans != 0.0 { index + 1 } else { index } },
    UnpackBlocks::Corner => 2,
  };

  let z_trans = match name.chars().nth(index).unwrap_or('x') {
    'b' => 2.2,
    'g' => -2.2,
    _ => 0.0,
  };

  (x_trans, y_trans, z_trans)
}


pub fn setup_core(
  mut commands: Commands,
  assets: Res<AssetServer>,
) {
  let cores = vec!["center", "r", "b", "w", "o", "g", "y"];

  for core in cores {

    let path = "core/".to_owned() + core + ".glb#Scene0";
    let part_handle = assets.load(path);

    let (x_trans, y_trans, z_trans) = unpack_coords(core, UnpackBlocks::Center);

    commands.spawn((SceneBundle {
      scene: part_handle,
      transform: Transform::from_xyz(x_trans, y_trans, z_trans),
      visibility: Visibility::Visible,
      ..Default::default()
    }, Block,
      Xpos(x_trans),
      Ypos(y_trans),
      Zpos(z_trans),
    ));
  }

}


pub fn setup_edges(
  mut commands: Commands,
  assets: Res<AssetServer>,
) {

  let edges = vec!["rb", "yb", "ob", "wb", "rg", "yg", "og", "wg", "yr", "yo", "wr", "wo"];

  for edge in edges {

    let path = "edge/".to_owned() + edge + ".glb#Scene0";
    let part_handle = assets.load(path);

    let (x_trans, y_trans, z_trans) = unpack_coords(edge, UnpackBlocks::Edge);

    commands.spawn((SceneBundle {
      scene: part_handle,
      transform: Transform::from_xyz(x_trans, y_trans, z_trans),
      visibility: Visibility::Visible,
      ..Default::default()
    }, Block,
      Xpos(x_trans),
      Ypos(y_trans),
      Zpos(z_trans),
    ));
  }

}

pub fn setup_corners(
  mut commands: Commands,
  assets: Res<AssetServer>,
) {

  let corners = vec!["wrb", "wrg", "wob", "wog", "yrb", "yrg", "yob", "yog"];

  for corner in corners {

    let path = "corner/".to_owned() + corner + ".glb#Scene0";
    let part_handle = assets.load(path);

    let (x_trans, y_trans, z_trans) = unpack_coords(corner, UnpackBlocks::Corner);

    commands.spawn((SceneBundle {
      scene: part_handle,
      transform: Transform::from_xyz(x_trans, y_trans, z_trans),
      visibility: Visibility::Visible,
      ..Default::default()
    }, Block,
      Xpos(x_trans),
      Ypos(y_trans),
      Zpos(z_trans),
    ));
  }
  
}



pub fn front_counter(
  kbd: Res<ButtonInput<KeyCode>>,
  mut cubes: Query<(&mut Transform, &Block, &Xpos, &mut Ypos, &mut Zpos)>,
) {

  let point = Vec3::from_slice(&[2.2, 0.0, 0.0]);

  for (mut transform, _cube, x, mut y, mut z) in &mut cubes {

    if x.0 == 2.2 {
      if kbd.just_pressed(KeyCode::KeyS) {
        transform.rotate_around(point, Quat::from_euler(EulerRot::XYZ, 45.0f32.to_radians(), 0.0, 0.0));
      }
      if kbd.just_released(KeyCode::KeyS) {
        transform.rotate_around(point, Quat::from_euler(EulerRot::XYZ, 45.0f32.to_radians(), 0.0, 0.0));
      }

      let temp = y.0;
      y.0 = z.0 * -1.0;
      z.0 = temp;
    }
    
  }
}

pub fn front_clockwise(
  kbd: Res<ButtonInput<KeyCode>>,
  mut cubes: Query<(&mut Transform, &Block, &Xpos, &mut Ypos, &mut Zpos)>,
) {
  let point = Vec3::from_slice(&[2.2, 0.0, 0.0]);

  for (mut transform, _cube, x, mut y, mut z) in &mut cubes {

    if x.0 == 2.2 {
      if kbd.just_pressed(KeyCode::KeyW) {
        transform.rotate_around(point, Quat::from_euler(EulerRot::XYZ, -45.0f32.to_radians(), 0.0, 0.0));
      }
      if kbd.just_released(KeyCode::KeyW) {
        transform.rotate_around(point, Quat::from_euler(EulerRot::XYZ, -45.0f32.to_radians(), 0.0, 0.0));
      }

      let temp = z.0;
      z.0 = y.0 * -1.0;
      y.0 = temp;
    }
    
  }
}

pub fn top_counter(
  kbd: Res<ButtonInput<KeyCode>>,
  mut cubes: Query<(&mut Transform, &Block, &mut Xpos, &Ypos, &mut Zpos)>,
) {
  let point = Vec3::from_slice(&[0.0, 2.2, 0.0]);

  for (mut transform, _cube, mut x, y, mut z) in &mut cubes {

    if y.0 == 2.2 {
      if kbd.just_pressed(KeyCode::KeyD) {
        transform.rotate_around(point, Quat::from_euler(EulerRot::XYZ, 0.0, 45.0f32.to_radians(), 0.0));
      }
      if kbd.just_released(KeyCode::KeyD) {
        transform.rotate_around(point, Quat::from_euler(EulerRot::XYZ, 0.0, 45.0f32.to_radians(), 0.0));
      }

      let temp = z.0;
      z.0 = x.0 * -1.0;
      x.0 = temp;
    }
    
  }
}

pub fn top_clockwise(
  kbd: Res<ButtonInput<KeyCode>>,
  mut cubes: Query<(&mut Transform, &Block, &mut Xpos, &Ypos, &mut Zpos)>,
) {
  let point = Vec3::from_slice(&[0.0, 2.2, 0.0]);

  for (mut transform, _cube, mut x, y, mut z) in &mut cubes {

    if y.0 == 2.2 {
      if kbd.just_pressed(KeyCode::KeyA) {
        transform.rotate_around(point, Quat::from_euler(EulerRot::XYZ, 0.0, -45.0f32.to_radians(), 0.0));
      }
      if kbd.just_released(KeyCode::KeyA) {
        transform.rotate_around(point, Quat::from_euler(EulerRot::XYZ, 0.0, -45.0f32.to_radians(), 0.0));
      }

      let temp = x.0;
      x.0 = z.0 * -1.0;
      z.0 = temp;
    }
    
  }
}