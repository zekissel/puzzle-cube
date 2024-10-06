use bevy::prelude::*;


#[derive(Resource)]
struct CubeRotate {
  active: bool,
}


pub struct CubeModels;

impl Plugin for CubeModels {
  fn build(&self, app: &mut App) {
    app.add_systems(Startup, (setup_core, setup_edges, setup_corners));
    app.add_systems(Update, (
      adjust_cube.run_if(any_with_component::<Block>),
      reset_cube.run_if(any_with_component::<Block>),
      cube_control.run_if(any_with_component::<Block>),
      rotate_cube.run_if(any_with_component::<Block>),
      
    ));
    app.insert_resource(CubeRotate { active: false });
  }
}


#[derive(Component, Default)]
struct Block;

#[derive(Component)]
struct BlockRotate {
  axis: Vec3,
  direction: f32,
  active: bool,
  timer: Timer,
}
impl Default for BlockRotate {
  fn default() -> Self {
    BlockRotate { 
      axis: Vec3::X,
      direction: 1.0,
      active: false,
      timer: Timer::from_seconds(0.25, TimerMode::Repeating),
    }
  }
}

#[derive(Component, Default)]
struct DefaultPosition(Vec3);

#[derive(Component)]
struct CubeSettings {
  rotate_x_pos: Option<KeyCode>,
  rotate_x_neg: Option<KeyCode>,
  rotate_y_pos: Option<KeyCode>,
  rotate_y_neg: Option<KeyCode>,

  front_cc: Option<KeyCode>,
  front_cw: Option<KeyCode>,
  top_cc: Option<KeyCode>,
  top_cw: Option<KeyCode>,

  reset_cube: Option<KeyCode>,
}

impl Default for CubeSettings {
  fn default() -> Self {
    CubeSettings {
      rotate_x_pos: Some(KeyCode::ArrowRight),
      rotate_x_neg: Some(KeyCode::ArrowLeft),
      rotate_y_pos: Some(KeyCode::ArrowUp),
      rotate_y_neg: Some(KeyCode::ArrowDown),

      front_cc: Some(KeyCode::KeyS),
      front_cw: Some(KeyCode::KeyW),
      top_cc: Some(KeyCode::KeyD),
      top_cw: Some(KeyCode::KeyA),

      reset_cube: Some(KeyCode::KeyR),
    }
  }
}

#[derive(Bundle, Default)]
struct BlockBundle {
  block: Block,
  scene_bundle: SceneBundle,
  settings: CubeSettings,
  default_position: DefaultPosition,
  rotate: BlockRotate,
}

enum UnpackBlocks { Center, Edge, Corner }

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


fn setup_core(
  mut commands: Commands,
  assets: Res<AssetServer>,
) {
  let cores = vec!["r", "b", "w", "o", "g", "y"];

  for core in cores {

    let path = "center/".to_owned() + core + ".glb#Scene0";
    let part_handle = assets.load(path);

    let (x_trans, y_trans, z_trans) = unpack_coords(core, UnpackBlocks::Center);

    commands.spawn(BlockBundle { 
      scene_bundle: SceneBundle {
        scene: part_handle,
        transform: Transform::from_xyz(x_trans, y_trans, z_trans),
        visibility: Visibility::Visible,
        ..Default::default()
      },
      default_position: DefaultPosition(Vec3::from_slice(&[x_trans, y_trans, z_trans])),
      ..Default::default()
    });
  }

}


fn setup_edges(
  mut commands: Commands,
  assets: Res<AssetServer>,
) {

  let edges = vec!["rb", "yb", "ob", "wb", "rg", "yg", "og", "wg", "yr", "yo", "wr", "wo"];

  for edge in edges {

    let path = "edge/".to_owned() + edge + ".glb#Scene0";
    let part_handle = assets.load(path);

    let (x_trans, y_trans, z_trans) = unpack_coords(edge, UnpackBlocks::Edge);

    commands.spawn(BlockBundle { 
      scene_bundle: SceneBundle {
        scene: part_handle,
        transform: Transform::from_xyz(x_trans, y_trans, z_trans),
        visibility: Visibility::Visible,
        ..Default::default()
      },
      default_position: DefaultPosition(Vec3::from_slice(&[x_trans, y_trans, z_trans])),
      ..Default::default()
    });
  }

}

fn setup_corners(
  mut commands: Commands,
  assets: Res<AssetServer>,
) {

  let corners = vec!["wrb", "wrg", "wob", "wog", "yrb", "yrg", "yob", "yog"];

  for corner in corners {

    let path = "corner/".to_owned() + corner + ".glb#Scene0";
    let part_handle = assets.load(path);

    let (x_trans, y_trans, z_trans) = unpack_coords(corner, UnpackBlocks::Corner);

    commands.spawn(BlockBundle { 
      scene_bundle: SceneBundle {
        scene: part_handle,
        transform: Transform::from_xyz(x_trans, y_trans, z_trans),
        visibility: Visibility::Visible,
        ..Default::default()
      },
      default_position: DefaultPosition(Vec3::from_slice(&[x_trans, y_trans, z_trans])),
      ..Default::default()
    });
  }
  
}



fn adjust_cube(
  kbd: Res<ButtonInput<KeyCode>>,
  mut cubes: Query<(&mut Transform, &Block, &CubeSettings)>,
) {

  for (mut transform, _cube, binds) in &mut cubes {

    let (c_cc, c_cw, c_up, c_do) = (
      kbd.just_pressed(binds.rotate_x_pos.unwrap()), kbd.just_pressed(binds.rotate_x_neg.unwrap()), 
      kbd.just_pressed(binds.rotate_y_pos.unwrap()), kbd.just_pressed(binds.rotate_y_neg.unwrap())
    );

    if c_cc || c_cw || c_up || c_do {  } else { return }

    let mut b = 0.0;
    let mut c = 0.0;

    if c_cc || c_cw { b = 90.0f32.to_radians(); c = 0.0; }
    else if c_up || c_do { b = 0.0; c = 90.0f32.to_radians(); };
    if c_cw { b *= -1.0 } else if c_do { c *= -1.0 };

    transform.rotate_around(Vec3::ZERO, Quat::from_euler(EulerRot::XYZ, 0.0, b, c));
    
  }
}



fn cube_control(
  kbd: Res<ButtonInput<KeyCode>>,
  mut cubes: Query<(&Transform, &Block, &CubeSettings, &mut BlockRotate)>,
  mut rotating: ResMut<CubeRotate>,
) {

  if rotating.active { return; }

  for (transform, _cube, binds, mut b_rotate) in &mut cubes {

    let (f_cc, f_cw, t_cc, t_cw) = (
      kbd.just_pressed(binds.front_cc.unwrap()), kbd.just_pressed(binds.front_cw.unwrap()), 
      kbd.just_pressed(binds.top_cc.unwrap()), kbd.just_pressed(binds.top_cw.unwrap())
    );

    if f_cc || f_cw || t_cc || t_cw { rotating.active = true; b_rotate.timer.reset(); } else { return }

    if f_cc || f_cw {
      if transform.translation.x >= 2.19 {
        b_rotate.axis = Vec3::X;
        b_rotate.active = true;
        if f_cw { b_rotate.direction = -1.0 } else { b_rotate.direction = 1.0 };
      }
    }
    else if t_cc || t_cw {
      if transform.translation.y >= 2.19 {
        b_rotate.axis = Vec3::Y;
        b_rotate.active = true;
        if t_cw { b_rotate.direction = -1.0 } else { b_rotate.direction = 1.0 };
      }
    }
    
  }
}

fn rotate_cube(
  mut cubes: Query<(&mut Transform, &Block, &mut BlockRotate)>,
  time: Res<Time>,
  mut rotating: ResMut<CubeRotate>,
) {

  if !rotating.active { return }

  for (mut transform, _cube, mut b_rotate) in &mut cubes {

    if b_rotate.active {
      b_rotate.timer.tick(time.delta());
      transform.rotate_around(Vec3::ZERO, Quat::from_axis_angle(b_rotate.axis, b_rotate.direction * 360.0f32.to_radians() * time.delta_seconds()));

      if b_rotate.timer.just_finished() {
        b_rotate.active = false;
        rotating.active = false;
      } 
    }
  }
    
}

fn reset_cube(
  kbd: Res<ButtonInput<KeyCode>>,
  mut cubes: Query<(&mut Transform, &Block, &CubeSettings, &DefaultPosition)>,
) {

  for (mut transform, _cube, binds, default) in &mut cubes {

    if binds.reset_cube.map(|key| kbd.just_pressed(key)).unwrap_or(false) {
      transform.translation = default.0;
      transform.rotation = Quat::from_euler(EulerRot::XYZ, 0.0, 0.0, 0.0);
    }

  }
  
}