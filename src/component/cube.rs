use bevy::prelude::*;

#[derive(Resource)]
struct CubeRotate {
  active: bool,
  speed: f32,
  scramble_timer: Timer,
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
    app.insert_resource(CubeRotate { 
      active: false, 
      speed: 420.0f32,
      scramble_timer: Timer::from_seconds(4.0, TimerMode::Once) 
    });
  }
}

#[derive(Component, Default)]
struct Block;

#[derive(Component)]
struct Target {
  translation: Vec3,
  rotation: Quat,
}

#[derive(Component)]
struct BlockRotate {
  axis: Vec3,
  direction: f32,
  active: bool,
  target: Target,
  timer: Timer,
}

impl Default for BlockRotate {
  fn default() -> Self {
    BlockRotate { 
      axis: Vec3::X,
      direction: 1.0,
      active: false,
      target: Target { translation: Vec3::ZERO, rotation: Quat::IDENTITY },
      timer: Timer::from_seconds(0.18, TimerMode::Repeating),
    }
  }
}

#[derive(Component, Default)]
struct DefaultPosition(Vec3);

#[derive(Component)]
struct ControlBinds {
  rotate_x_pos: Option<KeyCode>,
  rotate_x_neg: Option<KeyCode>,
  rotate_y_pos: Option<KeyCode>,
  rotate_y_neg: Option<KeyCode>,

  front_cc: Option<KeyCode>,
  front_cw: Option<KeyCode>,
  top_cc: Option<KeyCode>,
  top_cw: Option<KeyCode>,

  reset_cube: Option<KeyCode>,
  scramble_cube: Option<KeyCode>,
}

impl Default for ControlBinds {
  fn default() -> Self {
    ControlBinds {
      rotate_x_pos: Some(KeyCode::ArrowRight),
      rotate_x_neg: Some(KeyCode::ArrowLeft),
      rotate_y_pos: Some(KeyCode::ArrowUp),
      rotate_y_neg: Some(KeyCode::ArrowDown),

      front_cc: Some(KeyCode::KeyS),
      front_cw: Some(KeyCode::KeyW),
      top_cc: Some(KeyCode::KeyD),
      top_cw: Some(KeyCode::KeyA),

      reset_cube: Some(KeyCode::KeyR),
      scramble_cube: Some(KeyCode::KeyT),
    }
  }
}

#[derive(Bundle, Default)]
struct BlockBundle {
  block: Block,
  scene_bundle: SceneBundle,
  settings: ControlBinds,
  default_position: DefaultPosition,
  rotate: BlockRotate,
}

enum UnpackBlocks { Center, Edge, Corner }

/* MARK: SYSTEMS FOR CUBE SETUP and INITIALIZATION */

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




/* MARK: SYSTEMS FOR CONTROLLING CUBE */

fn adjust_cube(
  kbd: Res<ButtonInput<KeyCode>>,
  mut cubes: Query<(&mut Transform, &Block, &ControlBinds, &mut BlockRotate)>,
  mut rotating: ResMut<CubeRotate>,
) {

  if rotating.active { return }

  for (mut transform, _cube, binds, mut b_rotate) in &mut cubes {

    let (c_cc, c_cw, c_up, c_do) = (
      kbd.just_pressed(binds.rotate_x_pos.unwrap()), kbd.just_pressed(binds.rotate_x_neg.unwrap()), 
      kbd.just_pressed(binds.rotate_y_pos.unwrap()), kbd.just_pressed(binds.rotate_y_neg.unwrap())
    );

    if c_cc || c_cw || c_up || c_do { rotating.active = true; b_rotate.timer.reset(); b_rotate.active = true; } else { return }

    if c_cc || c_cw { b_rotate.axis = Vec3::Y; }
    else if c_up || c_do { b_rotate.axis = Vec3::Z; };
    if c_cc || c_up { b_rotate.direction = 1.0 } else { b_rotate.direction = -1.0 };

    let (tl, rt) = fetch_target(&mut transform, &mut b_rotate);
    b_rotate.target = Target { translation: tl, rotation: rt };
    
  }
}


fn cube_control(
  kbd: Res<ButtonInput<KeyCode>>,
  mut cubes: Query<(&mut Transform, &Block, &ControlBinds, &mut BlockRotate)>,
  mut rotating: ResMut<CubeRotate>,
) {

  if rotating.active { return }

  for (mut transform, _cube, binds, mut b_rotate) in &mut cubes {

    let (f_cc, f_cw, t_cc, t_cw) = (
      kbd.just_pressed(binds.front_cc.unwrap()), kbd.just_pressed(binds.front_cw.unwrap()), 
      kbd.just_pressed(binds.top_cc.unwrap()), kbd.just_pressed(binds.top_cw.unwrap())
    );

    if f_cc || f_cw || t_cc || t_cw { rotating.active = true; b_rotate.timer.reset(); } else { return }

    if f_cc || f_cw {
      if transform.translation.x == 2.20 {
        b_rotate.axis = Vec3::X;
        b_rotate.active = true;
        if f_cw { b_rotate.direction = -1.0 } else { b_rotate.direction = 1.0 };
        
        let (tl, rt) = fetch_target(&mut transform, &mut b_rotate);
        b_rotate.target = Target { translation: tl, rotation: rt };
      }
    }
    else if t_cc || t_cw {
      if transform.translation.y == 2.20 {
        b_rotate.axis = Vec3::Y;
        b_rotate.active = true;
        if t_cw { b_rotate.direction = -1.0 } else { b_rotate.direction = 1.0 };
        
        let (tl, rt) = fetch_target(&mut transform, &mut b_rotate);

        b_rotate.target = Target { translation: tl, rotation: rt };
      }
    }
    
  }
}

/* Use mutable transform to find exact translation and rotation after 90° rotation is completed in future (with time.delta()) */
fn fetch_target(transform: &mut Transform, b_rotate: &mut BlockRotate) -> (Vec3, Quat) {

  transform.rotate_around(Vec3::ZERO, Quat::from_axis_angle(b_rotate.axis, b_rotate.direction * 90.0f32.to_radians()));

  let tl = transform.translation;
  let rt = transform.rotation;
  // return to same position as beginning of frame
  transform.rotate_around(Vec3::ZERO, Quat::from_axis_angle(b_rotate.axis, b_rotate.direction * 90.0f32.to_radians() * -1.0));

  ((tl * 10.0).round() / 10.0, rt)
}

fn rotate_cube(
  mut cubes: Query<(&mut Transform, &Block, &mut BlockRotate)>,
  time: Res<Time>,
  mut rotating: ResMut<CubeRotate>,
) {

  if !rotating.active { return }
  let mut close = false;

  for (mut transform, _cube, mut b_rotate) in &mut cubes {

    if b_rotate.active {
      b_rotate.timer.tick(time.delta());
      rotating.scramble_timer.tick(time.delta());
      transform.rotate_around(Vec3::ZERO, Quat::from_axis_angle(b_rotate.axis, b_rotate.direction * rotating.speed.to_radians() * time.delta_seconds()));

      if b_rotate.timer.just_finished() || rotating.scramble_timer.just_finished() {
        b_rotate.active = false;
        b_rotate.direction = 0.0;
        b_rotate.axis = Vec3::ZERO;
        close = true;

        transform.translation = b_rotate.target.translation;
        transform.rotation = b_rotate.target.rotation;
      } 
    }
  }
  if close { 
    rotating.active = false; 
  }
    
}

fn reset_cube(
  kbd: Res<ButtonInput<KeyCode>>,
  mut cubes: Query<(&mut Transform, &Block, &ControlBinds, &DefaultPosition, &mut BlockRotate)>,
  mut rotating: ResMut<CubeRotate>,
) {

  for (mut transform, _cube, binds, default, mut b_rotate) in &mut cubes {

    if binds.reset_cube.map(|key| kbd.just_pressed(key)).unwrap_or(false) {
      rotating.active = false;
      transform.translation = default.0;
      transform.rotation = Quat::from_euler(EulerRot::XYZ, 0.0, 0.0, 0.0);
      b_rotate.active = false;
    }

  }
  
}

fn scramble_cube(
  kbd: Res<ButtonInput<KeyCode>>,
  mut cubes: Query<(&mut Transform, &Block, &ControlBinds, &mut BlockRotate)>,
  mut rotating: ResMut<CubeRotate>,
) {

  for (mut transform, _cube, binds, mut b_rotate) in &mut cubes {

    if binds.scramble_cube.map(|key| kbd.just_pressed(key)).unwrap_or(false) {
      rotating.active = true;
      rotating.speed = 1080.0;
      rotating.scramble_timer.reset();

      // algorithm to randomly choose x/y/z axis and direction +/- and rotate 90° multiple times
      
    }
  }
}