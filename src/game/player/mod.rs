use avian2d::{math::*, prelude::*};
use bevy::prelude::*;

use crate::{assets::ExampleAssets, prelude::*};

use super::InGameState;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
  fn build(&self, app: &mut App) {
    app
      .add_event::<MovementAction>()
      .add_systems(OnEnter(AppState::InGame), spawn_player)
      .add_systems(
        FixedUpdate,
        (
          keyboard_input,
          movement,
          apply_movement_damping,
        )
          .run_if(in_state(InGameState::Running))
          .chain(),
      )
      .add_systems(
        Update,
        update_camera.run_if(in_state(InGameState::Running)),
      );
  }
}

// PLAYER SYSTEMS
#[derive(Component)]
struct Player;

/// Spawn the player sprite and a 2D camera.
fn spawn_player(
  mut commands: Commands,
  mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
  example_assets: Res<ExampleAssets>,
  mut meshes: ResMut<Assets<Mesh>>,
) {
  let layout =
    TextureAtlasLayout::from_grid(UVec2::new(32, 48), 8, 3, None, None);

  let texture_atlas_layout = texture_atlas_layouts.add(layout);

  commands.spawn((
    Name::new("Player"),
    Player,
    Mesh2d(meshes.add(Capsule2d::new(12.5, 20.0))),
    Sprite::from_atlas_image(
      example_assets.player.clone(),
      TextureAtlas {
        layout: texture_atlas_layout,
        index: 21,
      },
    ),
    CharacterControllerBundle::new(Collider::capsule(12.5, 20.0))
      .with_movement(1250.0, 0.92),
    Friction::ZERO.with_combine_rule(CoefficientCombine::Min),
    Restitution::ZERO.with_combine_rule(CoefficientCombine::Min),
    Transform::from_scale(Vec3::splat(1.)),
    StateScoped(AppState::InGame),
  ));
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

// MOVEMENT SYSTEMS
/// An event sent for a movement input action.
#[derive(Event)]
pub enum MovementAction {
  Move(Vec2),
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

/// Sends [`MovementAction`] events based on keyboard input.
fn keyboard_input(
  mut movement_event_writer: EventWriter<MovementAction>,
  keyboard_input: Res<ButtonInput<KeyCode>>,
) {
  let left = keyboard_input.any_pressed([KeyCode::KeyA, KeyCode::ArrowLeft]);
  let right = keyboard_input.any_pressed([KeyCode::KeyD, KeyCode::ArrowRight]);
  let up = keyboard_input.any_pressed([KeyCode::KeyW, KeyCode::ArrowUp]);
  let down = keyboard_input.any_pressed([KeyCode::KeyS, KeyCode::ArrowDown]);

  let horizontal = right as i8 - left as i8;
  let vertical = up as i8 - down as i8;

  // Create a direction vector for x and y axes
  let direction = Vec2::new(horizontal as f32, vertical as f32);

  // Send movement event only if there's input
  if direction.length_squared() > 0.0 {
    movement_event_writer.send(MovementAction::Move(direction));
  }
}

/// Responds to [`MovementAction`] events and moves character controllers accordingly.
fn movement(
  time: Res<Time>,
  mut movement_event_reader: EventReader<MovementAction>,
  mut controllers: Query<(
    &MovementAcceleration,
    &mut LinearVelocity,
    &mut Sprite,
  )>,
) {
  // Precision is adjusted so that the example works with
  // both the `f32` and `f64` features. Otherwise you don't need this.
  let delta_time = time.delta_secs_f64().adjust_precision();

  for event in movement_event_reader.read() {
    for (movement_acceleration, mut linear_velocity, mut sprite) in
      &mut controllers
    {
      match event {
        MovementAction::Move(direction) => {
          let normalized = direction.normalize_or_zero();

          linear_velocity.x +=
            normalized.x * movement_acceleration.0 * delta_time;
          linear_velocity.y +=
            normalized.y * movement_acceleration.0 * delta_time;

          if let Some(atlas) = &mut sprite.texture_atlas {
            if direction.y > 0.0 {
              atlas.index = 22;
            } else if direction.y < 0.0 {
              atlas.index = 21;
            } else if direction.x > 0.0 {
              atlas.index = 23;
            } else if direction.x < 0.0 {
              atlas.index = 20;
            }
          }
        }
      }
    }
  }
}

/// Slows down movement in both directions.
fn apply_movement_damping(
  mut query: Query<(
    &MovementDampingFactor,
    &mut LinearVelocity,
  )>,
) {
  for (damping_factor, mut linear_velocity) in &mut query {
    linear_velocity.x *= damping_factor.0;
    linear_velocity.y *= damping_factor.0;
  }
}
