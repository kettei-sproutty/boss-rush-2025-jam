use avian2d::{math::*, prelude::*};
use bevy::prelude::*;
use leafwing_input_manager::prelude::*;
use seldom_state::prelude::*;

use crate::{assets::ExampleAssets, prelude::*};

use super::InGameState;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
  fn build(&self, app: &mut App) {
    app
      .add_plugins((
        InputManagerPlugin::<PlayerAction>::default(),
        StateMachinePlugin,
      ))
      .add_systems(OnEnter(AppState::InGame), spawn_player)
      .add_systems(
        Update,
        (use_actions, apply_movement_damping)
          .run_if(in_state(InGameState::Running))
          .chain(),
      )
      .add_systems(
        Update,
        update_camera.run_if(in_state(InGameState::Running)),
      );
  }
}

#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug, Reflect)]
enum PlayerAction {
  #[actionlike(DualAxis)]
  Move,
  Dash,
  Attack,
}

impl PlayerAction {
  fn input_map() -> InputMap<Self> {
    let mut input_map = InputMap::default();

    // gamepad input bindings
    input_map.insert_dual_axis(Self::Move, GamepadStick::LEFT);
    input_map.insert(Self::Dash, GamepadButton::South);
    input_map.insert(
      Self::Attack,
      GamepadButton::RightTrigger2,
    );

    // kbm input bindings
    input_map.insert_dual_axis(Self::Move, VirtualDPad::wasd());
    input_map.insert(Self::Dash, KeyCode::Space);
    input_map.insert(Self::Attack, MouseButton::Left);

    input_map
  }
}

#[derive(Component)]
struct Player;

#[derive(Clone, Copy, Component, Reflect)]
#[component(storage = "SparseSet")]
struct Idle;

#[derive(Clone, Copy, Component, Reflect)]
#[component(storage = "SparseSet")]
struct Moving;

#[derive(Clone, Copy, Component, Reflect)]
#[component(storage = "SparseSet")]
struct Dashing;

#[derive(Clone, Copy, Component, Reflect)]
#[component(storage = "SparseSet")]
struct Attacking;

/// Spawn the player sprite and a 2D camera.
fn spawn_player(
  mut commands: Commands,
  mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
  example_assets: Res<ExampleAssets>,
  mut meshes: ResMut<Assets<Mesh>>,
) {
  let layout =
    TextureAtlasLayout::from_grid(UVec2::new(32, 48), 4, 1, None, None);

  let texture_atlas_layout = texture_atlas_layouts.add(layout);

  commands.spawn((
    Name::new("Player"),
    Player,
    StateScoped(AppState::InGame),
    Mesh2d(meshes.add(Capsule2d::new(12.5, 20.0))),
    CharacterControllerBundle::new(Collider::capsule(12.5, 20.0))
      .with_movement(1250.0, 0.92),
    Friction::ZERO.with_combine_rule(CoefficientCombine::Min),
    Restitution::ZERO.with_combine_rule(CoefficientCombine::Min),
    Transform::from_xyz(500., 0., 0.),
    Sprite::from_atlas_image(
      example_assets.player.clone(),
      TextureAtlas {
        layout: texture_atlas_layout,
        index: 2,
      },
    ),
    InputManagerBundle::with_map(PlayerAction::input_map()),
    //  Idle,
    //  StateMachine::default()
    //    .trans::<Idle, _>(trigger, Moving)
    //    .set_trans_logging(true),
  ));
}

//fn trigger(action: &PlayerAction, _time: &Time) -> Result<(), ()> {
//  match action {
//    PlayerAction::Move => Ok(()),
//    _ => Err(()),
//  }
//}

fn use_actions(
  time: Res<Time>,
  query: Query<&ActionState<PlayerAction>, With<Player>>,
  mut controllers: Query<(
    &mut MovementAcceleration,
    &mut LinearVelocity,
  )>,
) {
  for action_state in query.iter() {
    let direction = action_state
      .axis_pair(&PlayerAction::Move)
      .normalize_or_zero();
    let delta_time = time.delta_secs();

    if direction != Vec2::ZERO {
      for (movement_acceleration, mut linear_velocity) in &mut controllers {
        linear_velocity.x += direction.x * movement_acceleration.0 * delta_time;
        linear_velocity.y += direction.y * movement_acceleration.0 * delta_time;
      }
    }

    if action_state.just_pressed(&PlayerAction::Dash) {
      println!("Dash!");
    }

    if action_state.just_pressed(&PlayerAction::Attack) {
      println!("Attack!");
    }
  }
}

// CAMERA SYSTEMS
const CAMERA_DECAY_RATE: f32 = 5.0;
/// Update the camera position by tracking the player.
fn update_camera(
  mut camera: Query<&mut Transform, (With<Camera2d>, Without<Player>)>,
  player: Query<&Transform, (With<Player>, Without<Camera2d>)>,
  time: Res<Time>,
) {
  let Ok(mut camera) = camera.get_single_mut() else {
    return;
  };

  let Ok(player) = player.get_single() else {
    return;
  };

  let Vec3 { x, y, .. } = player.translation;
  let direction = Vec3::new(x, y, camera.translation.z);

  // Applies a smooth effect to camera movement using stable interpolation
  // between the camera position and the player position on the x and y axes.
  camera.translation.smooth_nudge(
    &direction,
    CAMERA_DECAY_RATE,
    time.delta_secs(),
  );
}

/// A marker component indicating that an entity is using a character controller.
#[derive(Component)]
pub struct CharacterController;

/// A marker component indicating that an entity is on the ground.
/// The acceleration used for character movement.
#[derive(Component)]
pub struct MovementAcceleration(Scalar);

/// The damping factor used for slowing down movement.
#[derive(Component)]
pub struct MovementDampingFactor(Scalar);

/// A bundle that contains the components needed for a basic
/// kinematic character controller.
#[derive(Bundle)]
pub struct CharacterControllerBundle {
  character_controller: CharacterController,
  rigid_body: RigidBody,
  collider: Collider,
  ground_caster: ShapeCaster,
  locked_axes: LockedAxes,
  movement: MovementBundle,
}

/// A bundle that contains components for character movement.
#[derive(Bundle)]
pub struct MovementBundle {
  acceleration: MovementAcceleration,
  damping: MovementDampingFactor,
}

impl MovementBundle {
  pub const fn new(acceleration: Scalar, damping: Scalar) -> Self {
    Self {
      acceleration: MovementAcceleration(acceleration),
      damping: MovementDampingFactor(damping),
    }
  }
}

impl Default for MovementBundle {
  fn default() -> Self {
    Self::new(30.0, 0.9)
  }
}

impl CharacterControllerBundle {
  pub fn new(collider: Collider) -> Self {
    // Create shape caster as a slightly smaller version of collider
    let mut caster_shape = collider.clone();
    caster_shape.set_scale(Vector::ONE * 0.99, 10);

    Self {
      character_controller: CharacterController,
      rigid_body: RigidBody::Dynamic,
      collider,
      ground_caster: ShapeCaster::new(
        caster_shape,
        Vector::ZERO,
        0.0,
        Dir2::NEG_Y,
      )
      .with_max_distance(10.0),
      locked_axes: LockedAxes::ROTATION_LOCKED,
      movement: MovementBundle::default(),
    }
  }

  pub fn with_movement(
    mut self,
    acceleration: Scalar,
    damping: Scalar,
  ) -> Self {
    self.movement = MovementBundle::new(acceleration, damping);
    self
  }
}

fn apply_movement_damping(
  mut query: Query<(
    &MovementDampingFactor,
    &mut LinearDamping,
  )>,
) {
  for (damping_factor, mut linear_damping) in &mut query {
    println!("Damping ====>: {:?}", damping_factor.0);
    linear_damping.0 = damping_factor.0;
  }
}
