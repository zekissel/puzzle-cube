use bevy::prelude::*;
use bevy_prng::ChaCha8Rng;
use bevy_rand::prelude::GlobalEntropy;
use rand_core::{RngCore, SeedableRng};

// °s per second
const TURN_SPEED: f32 = 420.0;
const SCRAMBLE_SPEED: f32 = 1080.0;

/* MARK: CUBE PLUGIN
*/
pub struct CubeModels;

impl Plugin for CubeModels {
  fn build(&self, app: &mut App) {
    app.add_systems(Startup, setup_cube);
    app.add_systems(Update, (
      adjust_cube.run_if(any_with_component::<Block>),
      reset_cube.run_if(any_with_component::<Block>),
      cube_control.run_if(any_with_component::<Block>),
      scramble_cube.run_if(any_with_component::<Block>),
      rotate_cube.run_if(any_with_component::<Block>),
      rotate_scramble.run_if(any_with_component::<Block>),
      
    ));
    app.insert_resource(AggregateMovement { 
      active: false, 
      speed: TURN_SPEED,
      axis: Vec3::ZERO,
      positive: true,
      direction: 1.0,
      turn_timer: Timer::from_seconds(0.18, TimerMode::Repeating),
      rotation_timer: Timer::from_seconds(0.075, TimerMode::Repeating),
      scramble: 0,
    });
    app.insert_resource(GlobalEntropy::new(ChaCha8Rng::seed_from_u64(0)));
  }
}

/* MARK: MOVEMENT <RES>
*/
#[derive(Resource)]
struct AggregateMovement {
  active: bool,
  speed: f32,

  // used on player-initiated turns
  axis: Vec3,
  positive: bool,
  direction: f32,
  turn_timer: Timer,

  // used with scramble turns
  rotation_timer: Timer,
  scramble: i32,
}

#[derive(Component, Default)]
struct Block;

#[derive(Component)]
struct Target {
  translation: Vec3,
  rotation: Quat,
}

/* MARK: ROTATION NODE
*/
#[derive(Component)]
struct MovementNode {
  active: bool,
  target: Target,
}

impl Default for MovementNode {
  fn default() -> Self {
    MovementNode {
      active: false,
      target: Target { translation: Vec3::ZERO, rotation: Quat::IDENTITY },
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

/* MARK: BLOCK BUNDLE
 */
#[derive(Bundle, Default)]
struct BlockBundle {
  block: Block,
  scene_bundle: SceneBundle,
  settings: ControlBinds,
  default_position: DefaultPosition,
  movement_node: MovementNode,
}



enum UnpackBlocks { Center, Edge, Corner }
/* MARK: CUBE SETUP
*/
fn unpack_coords(name: &str, area: UnpackBlocks) -> (f32, f32, f32) {

  let mut index = 0;

  let x_trans = match name.chars().nth(index).unwrap() {
    'w' => 2.2, 'y' => -2.2, _ => 0.0,
  };

  index = match area {
    UnpackBlocks::Edge => { if x_trans != 0.0 { index + 1 } else { index } },
    UnpackBlocks::Corner => 1, _ => 0,
  };

  let y_trans = match name.chars().nth(index).unwrap() {
    'r' => 2.2, 'o' => -2.2, _ => 0.0,
  };

  index = match area {
    UnpackBlocks::Edge => { if y_trans != 0.0 { index + 1 } else { index } },
    UnpackBlocks::Corner => 2, _ => 0,
  };

  let z_trans = match name.chars().nth(index).unwrap_or('x') {
    'b' => 2.2, 'g' => -2.2, _ => 0.0,
  };

  (x_trans, y_trans, z_trans)
}

fn setup_cube(
  mut commands: Commands,
  assets: Res<AssetServer>,
) {
  let cores = vec!["r", "b", "w", "o", "g", "y"];
  let edges = vec!["rb", "yb", "ob", "wb", "rg", "yg", "og", "wg", "yr", "yo", "wr", "wo"];
  let corners = vec!["wrb", "wrg", "wob", "wog", "yrb", "yrg", "yob", "yog"];

  let all = corners.iter().chain(edges.iter().chain(cores.iter())).collect::<Vec<_>>();

  for block in all {

    let root = match block.len() {
      2 => "edge",
      3 => "corner",
      _ => "center",
    };
    let path = root.to_owned() + "/" + block + ".glb#Scene0";
    let part_handle = assets.load(path);

    let (x_trans, y_trans, z_trans) = unpack_coords(&block, match root {
      "edge" => UnpackBlocks::Edge,
      "corner" => UnpackBlocks::Corner,
      _ => UnpackBlocks::Center,
    });

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


// MARK: UPDATE SYSTEMS:




/* MARK: ROTATE CUBE
*/
fn adjust_cube(
  kbd: Res<ButtonInput<KeyCode>>,
  mut cubes: Query<(&mut Transform, &Block, &ControlBinds, &mut MovementNode)>,
  mut agg_mov: ResMut<AggregateMovement>,
) {

  if agg_mov.active { return }
  let mut axis = Vec3::X;
  let mut direction = 1.0;

  for (mut transform, _cube, binds, mut b_rotate) in &mut cubes {

    let (c_cc, c_cw, c_up, c_do) = (
      kbd.just_pressed(binds.rotate_x_pos.unwrap()), kbd.just_pressed(binds.rotate_x_neg.unwrap()), 
      kbd.just_pressed(binds.rotate_y_pos.unwrap()), kbd.just_pressed(binds.rotate_y_neg.unwrap())
    );

    if c_cc || c_cw || c_up || c_do { agg_mov.active = true; b_rotate.active = true; } else { return }

    if c_cc || c_cw { axis = Vec3::Y; }
    else if c_up || c_do { axis = Vec3::Z; };
    if c_cc || c_up { direction = 1.0 } else { direction = -1.0 };

    let (tl, rt) = fetch_target(&mut transform, axis, direction);
    b_rotate.target = Target { translation: tl, rotation: rt };
  }

  agg_mov.positive = true;
  agg_mov.axis = axis;
  agg_mov.direction = direction;

  agg_mov.turn_timer.reset()
}

/* MARK: REGULAR CTRL
 */
fn cube_control(
  kbd: Res<ButtonInput<KeyCode>>,
  mut cubes: Query<(&mut Transform, &Block, &ControlBinds, &mut MovementNode)>,
  mut agg_mov: ResMut<AggregateMovement>,
) {

  if agg_mov.active { return }

  let mut axis = Vec3::X;
  let mut direction = 1.0;

  for (mut transform, _cube, binds, mut move_node) in &mut cubes {

    let (f_cc, f_cw, t_cc, t_cw) = (
      kbd.just_pressed(binds.front_cc.unwrap()), kbd.just_pressed(binds.front_cw.unwrap()), 
      kbd.just_pressed(binds.top_cc.unwrap()), kbd.just_pressed(binds.top_cw.unwrap())
    );

    if f_cc || f_cw || t_cc || t_cw { agg_mov.active = true; agg_mov.turn_timer.reset(); } else { return }

    let limit = if agg_mov.positive { 2.20 } else { -2.20 };

    if f_cc || f_cw {
      if transform.translation.x == limit {
        axis = Vec3::X;
        move_node.active = true;
        if f_cw { direction = -1.0 } else { direction = 1.0 };
        
        let (tl, rt) = fetch_target(&mut transform, axis, direction);
        move_node.target = Target { translation: tl, rotation: rt };
      }
    }
    else if t_cc || t_cw {
      if transform.translation.y == limit {
        axis = Vec3::Y;
        move_node.active = true;
        if t_cw { direction = -1.0 } else { direction = 1.0 };
        
        let (tl, rt) = fetch_target(&mut transform, axis, direction);
        move_node.target = Target { translation: tl, rotation: rt };
      }
    }
    
  }

  agg_mov.axis = axis;
  agg_mov.direction = direction;
  agg_mov.positive = true;
}

/* MARK: SCRAMBLE CTRL
 */
fn scramble_cube(
  kbd: Res<ButtonInput<KeyCode>>,
  mut cubes: Query<(&Block, &ControlBinds)>,
  mut agg_mov: ResMut<AggregateMovement>,
) {

  for (_cube, binds) in &mut cubes {

    if binds.scramble_cube.map(|key| kbd.just_pressed(key)).unwrap_or(false) {
      agg_mov.active = false;
      agg_mov.scramble = 100;
      agg_mov.speed = SCRAMBLE_SPEED;
    }
  }
}

/* MARK: RESET CTRL
 */
fn reset_cube(
  kbd: Res<ButtonInput<KeyCode>>,
  mut cubes: Query<(&mut Transform, &Block, &ControlBinds, &DefaultPosition, &mut MovementNode)>,
  mut agg_mov: ResMut<AggregateMovement>,
) {

  for (mut transform, _cube, binds, default, mut move_node) in &mut cubes {

    if binds.reset_cube.map(|key| kbd.just_pressed(key)).unwrap_or(false) {
      agg_mov.active = false;
      agg_mov.scramble = 0;
      agg_mov.speed = TURN_SPEED;
      agg_mov.axis = Vec3::ZERO;
      transform.translation = default.0;
      transform.rotation = Quat::IDENTITY;
      move_node.active = false;
    }

  }
  
}



/* MARK: REGULAR TURN
 */
fn rotate_cube(
  mut cubes: Query<(&mut Transform, &Block, &mut MovementNode)>,
  time: Res<Time>,
  mut agg_mov: ResMut<AggregateMovement>,
) {

  if !agg_mov.active || agg_mov.scramble > 0 { return }

  agg_mov.speed = 420.0;
  let mut close = false;

  agg_mov.turn_timer.tick(time.delta());

  for (mut transform, _cube, mut b_rotate) in &mut cubes {

    if b_rotate.active {

      transform.rotate_around(Vec3::ZERO, Quat::from_axis_angle(agg_mov.axis, agg_mov.direction * agg_mov.speed.to_radians() * time.delta_seconds()));

      if agg_mov.turn_timer.just_finished() {
        b_rotate.active = false;
        close = true;

        transform.translation = b_rotate.target.translation;
        transform.rotation = b_rotate.target.rotation;
      }
    }
  }

  if close { agg_mov.active = false; }
    
}


/* MARK: SCRAMBLE TURN
 */
fn rotate_scramble(
  mut cubes: Query<(&mut Transform, &Block, &mut MovementNode)>,
  time: Res<Time>,
  mut agg_mov: ResMut<AggregateMovement>,
  mut rng: ResMut<GlobalEntropy<ChaCha8Rng>>,
) {
  if agg_mov.scramble == 0 { return }

  if !agg_mov.active {

    let (axis, positive, direction) = randomize_vars(&mut rng);
    agg_mov.axis = axis;
    agg_mov.direction = direction;
    agg_mov.positive = positive;

    let limit = if positive { 2.20 } else { -2.20 };

    for (mut transform, _cube, mut move_node) in &mut cubes {

      let comparison = match axis {
        Vec3::X => transform.translation.x,
        Vec3::Y => transform.translation.y,
        _       => transform.translation.z
      };
      if comparison == limit {
        let (tl, rt) = fetch_target(&mut transform, axis, direction);
        move_node.target = Target { translation: tl, rotation: rt };
        move_node.active = true;
      }
    }

    agg_mov.active = true;
    agg_mov.scramble -= 1;
    agg_mov.rotation_timer.reset();
  } 
  
  if agg_mov.active {
    let mut close = false;
    let mut stop_rotate = false;

    agg_mov.rotation_timer.tick(time.delta());

    for (mut transform, _cube, mut b_rotate) in &mut cubes {

      if b_rotate.active {
        transform.rotate_around(Vec3::ZERO, Quat::from_axis_angle(agg_mov.axis, agg_mov.direction * agg_mov.speed.to_radians() * time.delta_seconds()));
  
        if agg_mov.rotation_timer.just_finished() {
          stop_rotate = true;
          b_rotate.active = false;
  
          transform.translation = b_rotate.target.translation;
          transform.rotation = b_rotate.target.rotation;
      
          if agg_mov.scramble == 0 {
            close = true;
          }
        }
      }
    }

    if stop_rotate { agg_mov.active = false; }
    if close { 
      agg_mov.active = false; 
      agg_mov.speed = TURN_SPEED; 
      agg_mov.axis = Vec3::ZERO;
      agg_mov.direction = 0.0;
      agg_mov.positive = true;
    }
  }
    
}

// MARK: UTIL
fn randomize_vars(rng: &mut ResMut<GlobalEntropy<ChaCha8Rng>>) -> (Vec3, bool, f32) {

  let positive = rng.next_u32() % 2 == 0;
  let axis = match rng.next_u32() % 3 {
    0 => Vec3::X, 
    1 => Vec3::Y, 
    _ => Vec3::Z,
  };
  let direction = if rng.next_u32() % 2 == 0 { 1.0 } else { -1.0 };

  (axis, positive, direction)
}

/* Use mutable transform to find exact translation and rotation after 90° rotation is completed in future (with time.delta()) */
fn fetch_target(transform: &mut Transform, axis: Vec3, direction: f32) -> (Vec3, Quat) {

  transform.rotate_around(Vec3::ZERO, Quat::from_axis_angle(axis, direction * 90.0f32.to_radians()));

  let tl = transform.translation;
  let rt = transform.rotation;
  // return to same position as beginning of frame
  transform.rotate_around(Vec3::ZERO, Quat::from_axis_angle(axis, direction * 90.0f32.to_radians() * -1.0));

  ((tl * 10.0).round() / 10.0, rt)
}
