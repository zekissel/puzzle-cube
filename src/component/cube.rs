use bevy::prelude::*;
use bevy_prng::ChaCha8Rng;
use bevy_rand::prelude::GlobalEntropy;
use rand_core::{RngCore, SeedableRng};

// °s per second
const TURN_SPEED: f32 = 560.0;
const SCRAMBLE_SPEED: f32 = 1080.0;

// delay in seconds between turns
const TURN_DELAY: f32 = 0.14;
const SCRAMBLE_DELAY: f32 = 0.075;

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
      turn_timer: Timer::from_seconds(TURN_DELAY, TimerMode::Repeating),
      scramble: 0,
      scramble_turn_timer: Timer::from_seconds(SCRAMBLE_DELAY, TimerMode::Repeating),
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
  positive: bool, // positive quadrant or not
  direction: f32,
  turn_timer: Timer,

  // used with scramble turns
  scramble_turn_timer: Timer,
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

  TARGET marks where individual cubes should be after rotation timer ends (snap cubes into place)
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

// used to reset cube positions
#[derive(Component, Default)]
struct DefaultPosition(Vec3);

#[derive(Component)]
struct ControlBinds {
  button_rotate_x: Option<KeyCode>,
  button_rotate_y: Option<KeyCode>,
  button_rotate_z: Option<KeyCode>,

  button_front_turn: Option<KeyCode>,
  button_right_turn: Option<KeyCode>,
  button_up_turn: Option<KeyCode>,
  button_back_turn: Option<KeyCode>,
  button_left_turn: Option<KeyCode>,
  button_down_turn: Option<KeyCode>,

  button_prime: Option<KeyCode>,
  button_wide: Option<KeyCode>,
  button_alt: Option<KeyCode>,

  button_reset: Option<KeyCode>,
  button_scramble: Option<KeyCode>,
}

impl Default for ControlBinds {
  fn default() -> Self {
    ControlBinds {
      button_rotate_x: Some(KeyCode::ArrowUp),
      button_rotate_y: Some(KeyCode::ArrowRight),
      button_rotate_z: Some(KeyCode::ArrowDown),

      button_front_turn: Some(KeyCode::KeyW),
      button_right_turn: Some(KeyCode::KeyD),
      button_up_turn: Some(KeyCode::KeyE),
      button_back_turn: Some(KeyCode::KeyS),
      button_left_turn: Some(KeyCode::KeyA),
      button_down_turn: Some(KeyCode::KeyQ),

      button_prime: Some(KeyCode::ShiftLeft),
      button_wide: Some(KeyCode::ControlLeft),
      button_alt: Some(KeyCode::AltLeft),

      button_reset: Some(KeyCode::KeyR),
      button_scramble: Some(KeyCode::KeyT),
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
// use part (and file) names defined in setup_cube() to create blocks in correct position
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

    // bring blocks into game world
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

  for (mut transform, _cube, binds, mut move_node) in &mut cubes {

    let (button_x, button_y, button_z) = (
      kbd.just_pressed(binds.button_rotate_x.unwrap()), 
      kbd.just_pressed(binds.button_rotate_y.unwrap()), 
      kbd.just_pressed(binds.button_rotate_z.unwrap()),
    );

    if button_x || button_y || button_z { agg_mov.active = true; move_node.active = true; } else { return }

    let button_prime = kbd.pressed(binds.button_prime.unwrap());
    if button_prime { direction = 1.0 } else { direction = -1.0 };

    if button_x { axis = Vec3::X; } else if button_y { axis = Vec3::Y; }
    else if button_z { axis = Vec3::Z; };

    let (tl, rt) = fetch_target(&mut transform, axis, direction);
    move_node.target = Target { translation: tl, rotation: rt };
  }

  agg_mov.positive = true;
  agg_mov.axis = axis;
  agg_mov.direction = direction;

  agg_mov.turn_timer.reset();
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
  let mut positive = true;

  for (mut transform, _cube, binds, mut move_node) in &mut cubes {

    let (button_f, button_b, button_u, button_d, button_r, button_l) = (
      kbd.just_pressed(binds.button_front_turn.unwrap()), kbd.just_pressed(binds.button_back_turn.unwrap()), 
      kbd.just_pressed(binds.button_up_turn.unwrap()), kbd.just_pressed(binds.button_down_turn.unwrap()),
      kbd.just_pressed(binds.button_right_turn.unwrap()), kbd.just_pressed(binds.button_left_turn.unwrap())
    );

    // initiate aggregate movement if any button is pressed
    if button_f || button_b || button_u || button_d || button_r || button_l { agg_mov.active = true; agg_mov.turn_timer.reset(); } else { return }
    if button_b || button_l || button_d { positive = false; } else { positive = true; }

    let button_prime = kbd.pressed(binds.button_prime.unwrap());
    let button_wide = kbd.pressed(binds.button_wide.unwrap());
    let button_alt = kbd.pressed(binds.button_alt.unwrap());

    let limit = if positive { 2.20 } else { -2.20 };
    if button_prime { direction = -1.0 } else { direction = 1.0 };
    if button_f || button_r || button_u { direction *= -1.0 };
    //if button_alt && (button_b || button_r || button_u) { direction *= -1.0; } // both axes move in same direction
    // forward axis is regular, back axis is reversed
    if button_alt { 
      direction *= -1.0; 
      if button_b || button_f { direction *= -1.0; }
    }

    if button_f || button_b { axis = Vec3::Z; } else if button_u || button_d { axis = Vec3::Y; } else { axis = Vec3::X; }
    let comparison = match axis {
      Vec3::X => transform.translation.x,
      Vec3::Y => transform.translation.y,
      Vec3::Z => transform.translation.z,
      _ => 0.0,
    };

    if (!button_alt && comparison == limit) || ((button_wide || button_alt) && comparison == 0.0) {
      move_node.active = true;
      
      let (tl, rt) = fetch_target(&mut transform, axis, direction);
      move_node.target = Target { translation: tl, rotation: rt };
    }
    
  }

  agg_mov.axis = axis;
  agg_mov.direction = direction;
  agg_mov.positive = positive;
}

/* MARK: SCRAMBLE CTRL
 */
fn scramble_cube(
  kbd: Res<ButtonInput<KeyCode>>,
  mut cubes: Query<(&Block, &ControlBinds)>,
  mut agg_mov: ResMut<AggregateMovement>,
) {

  for (_cube, binds) in &mut cubes {

    if binds.button_scramble.map(|key| kbd.just_pressed(key)).unwrap_or(false) {
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

    if binds.button_reset.map(|key| kbd.just_pressed(key)).unwrap_or(false) {
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

  for (mut transform, _cube, mut move_node) in &mut cubes {

    if move_node.active {

      transform.rotate_around(Vec3::ZERO, Quat::from_axis_angle(agg_mov.axis, agg_mov.direction * agg_mov.speed.to_radians() * time.delta_seconds()));

      if agg_mov.turn_timer.just_finished() {
        move_node.active = false;
        close = true;

        transform.translation = move_node.target.translation;
        transform.rotation = move_node.target.rotation;
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
    agg_mov.scramble_turn_timer.reset();
  } 
  
  if agg_mov.active {
    let mut close = false;
    let mut stop_rotate = false;

    agg_mov.scramble_turn_timer.tick(time.delta());

    for (mut transform, _cube, mut move_node) in &mut cubes {

      if move_node.active {
        transform.rotate_around(Vec3::ZERO, Quat::from_axis_angle(agg_mov.axis, agg_mov.direction * agg_mov.speed.to_radians() * time.delta_seconds()));
  
        if agg_mov.scramble_turn_timer.just_finished() {
          stop_rotate = true;
          move_node.active = false;
  
          transform.translation = move_node.target.translation;
          transform.rotation = move_node.target.rotation;
      
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
