mod pause;
mod ui;

use bevy::transform;
use bevy_light_2d::light::AmbientLight2d;

use crate::{assets::ExampleAssets, prelude::*};

#[derive(Component, Deref, DerefMut)]
struct AnimationTimer(Timer);

#[derive(Component)]
struct AnimationIndices {
  first: usize,
  last: usize,
}

pub struct GamePlugin<S: States> {
  pub state: S,
}

impl<S: States> Plugin for GamePlugin<S> {
  fn build(&self, app: &mut App) {
    app
      .add_plugins((pause::PausePlugin, ui::UiPlugin))
      .add_systems(
        OnEnter(self.state.clone()),
        (
          setup_game,
          spawn_example_tree,
          spawn_player,
        ),
      )
      .add_systems(FixedUpdate, advance_physics)
      .add_systems(
        // The `RunFixedMainLoop` schedule allows us to schedule systems to run before and after the fixed timestep loop.
        RunFixedMainLoop,
        (
          // The physics simulation needs to know the player's input, so we run this before the fixed timestep loop.
          // Note that if we ran it in `Update`, it would be too late, as the physics simulation would already have been advanced.
          // If we ran this in `FixedUpdate`, it would sometimes not register player input, as that schedule may run zero times per frame.
          handle_input
            .in_set(RunFixedMainLoopSystem::BeforeFixedMainLoop)
            .run_if(in_state(InGameState::Running)),
          // The player's visual representation needs to be updated after the physics simulation has been advanced.
          // This could be run in `Update`, but if we run it here instead, the systems in `Update`
          // will be working with the `Transform` that will actually be shown on screen.
          interpolate_rendered_transform
            .in_set(RunFixedMainLoopSystem::AfterFixedMainLoop)
            .run_if(in_state(InGameState::Running)),
        ),
      )
      .add_systems(
        Update,
        (animate_sprite.run_if(in_state(InGameState::Running)))
          .run_if(in_state(AppState::InGame)),
      );
  }
}

/// A vector representing the player's input, accumulated over all frames that ran
/// since the last time the physics simulation was advanced.
#[derive(Debug, Component, Clone, Copy, PartialEq, Default, Deref, DerefMut)]
struct AccumulatedInput(Vec2);

/// A vector representing the player's velocity in the physics simulation.
#[derive(Debug, Component, Clone, Copy, PartialEq, Default, Deref, DerefMut)]
struct Velocity(Vec3);

/// The actual position of the player in the physics simulation.
/// This is separate from the `Transform`, which is merely a visual representation.
///
/// If you want to make sure that this component is always initialized
/// with the same value as the `Transform`'s translation, you can
/// use a [component lifecycle hook](https://docs.rs/bevy/0.14.0/bevy/ecs/component/struct.ComponentHooks.html)
#[derive(Debug, Component, Clone, Copy, PartialEq, Default, Deref, DerefMut)]
struct PhysicalTranslation(Vec3);

/// The value [`PhysicalTranslation`] had in the last fixed timestep.
/// Used for interpolation in the `interpolate_rendered_transform` system.
#[derive(Debug, Component, Clone, Copy, PartialEq, Default, Deref, DerefMut)]
struct PreviousPhysicalTranslation(Vec3);

/// Spawn the player sprite and a 2D camera.
fn spawn_player(
  mut commands: Commands,
  mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
  example_assets: Res<ExampleAssets>,
) {
  let layout =
    TextureAtlasLayout::from_grid(UVec2::new(32, 48), 8, 3, None, None);

  let texture_atlas_layout = texture_atlas_layouts.add(layout);
  commands.spawn((
    Name::new("Player"),
    Sprite::from_atlas_image(
      example_assets.player.clone(),
      TextureAtlas {
        layout: texture_atlas_layout,
        index: 21,
      },
    ),
    Transform::from_scale(Vec3::splat(1.)),
    AccumulatedInput::default(),
    Velocity::default(),
    PhysicalTranslation::default(),
    PreviousPhysicalTranslation::default(),
    StateScoped(AppState::InGame),
  ));
}

fn setup_game(mut commands: Commands) {
  commands.spawn((
    Name::new("GameCamera"),
    StateScoped(AppState::InGame),
    Camera2d,
  ));

  commands.spawn((
    Name::new("AmbientLight"),
    AmbientLight2d::default(),
    StateScoped(AppState::InGame),
  ));
}

fn spawn_example_tree(
  mut commands: Commands,
  example_assets: Res<ExampleAssets>,
  mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
  let layout =
    TextureAtlasLayout::from_grid(UVec2::splat(64), 16, 1, None, None);

  let texture_atlas_layout = texture_atlas_layouts.add(layout);

  commands.spawn((
    StateScoped(AppState::InGame),
    Transform::from_xyz(0., 0., 0.),
    Sprite::from_atlas_image(
      example_assets.tree.clone(),
      TextureAtlas {
        layout: texture_atlas_layout,
        index: 0,
      },
    ),
    AnimationIndices { first: 0, last: 15 },
    AnimationTimer(Timer::from_seconds(
      0.1,
      TimerMode::Repeating,
    )),
  ));
}

fn animate_sprite(
  time: Res<Time>,
  mut query: Query<(
    &AnimationIndices,
    &mut AnimationTimer,
    &mut Sprite,
  )>,
) {
  for (indices, mut timer, mut sprite) in &mut query {
    timer.tick(time.delta());

    if timer.just_finished() {
      if let Some(atlas) = &mut sprite.texture_atlas {
        atlas.index = if atlas.index == indices.last {
          indices.first
        } else {
          atlas.index + 1
        };
      }
    }
  }
}

fn handle_input(
  keyboard_input: Res<ButtonInput<KeyCode>>,
  mut query: Query<(
    &mut AccumulatedInput,
    &mut Velocity,
    &mut Sprite,
  )>,
) {
  // Since Bevy's default 2D camera setup is scaled such that
  // one unit is one pixel, you can think of this as
  // "How many pixels per second should the player move?"
  let mut speed: f32 = 210.0;

  for (mut input, mut velocity, mut sprite) in query.iter_mut() {
    if let Some(atlas) = &mut sprite.texture_atlas {
      if keyboard_input.pressed(KeyCode::KeyA) {
        input.x -= 1.0;
        atlas.index = 20
      }
      if keyboard_input.pressed(KeyCode::KeyD) {
        input.x += 1.0;
        atlas.index = 23
      }
      if keyboard_input.pressed(KeyCode::ShiftLeft) {
        speed = 300.0;
      }
      if keyboard_input.pressed(KeyCode::KeyW) {
        input.y += 1.0;
        atlas.index = 22
      }
      if keyboard_input.pressed(KeyCode::KeyS) {
        input.y -= 1.0;
        atlas.index = 21
      }
    }

    // Need to normalize and scale because otherwise
    // diagonal movement would be faster than horizontal or vertical movement.
    // This effectively averages the accumulated input.

    velocity.0 = input.extend(0.0).normalize_or_zero() * speed;
  }
}

/// Advance the physics simulation by one fixed timestep. This may run zero or multiple times per frame.
///
/// Note that since this runs in `FixedUpdate`, `Res<Time>` would be `Res<Time<Fixed>>` automatically.
/// We are being explicit here for clarity.
fn advance_physics(
  fixed_time: Res<Time<Fixed>>,
  mut query: Query<(
    &mut PhysicalTranslation,
    &mut PreviousPhysicalTranslation,
    &mut AccumulatedInput,
    &Velocity,
  )>,
) {
  for (
    mut current_physical_translation,
    mut previous_physical_translation,
    mut input,
    velocity,
  ) in query.iter_mut()
  {
    previous_physical_translation.0 = current_physical_translation.0;
    current_physical_translation.0 += velocity.0 * fixed_time.delta_secs();

    // Reset the input accumulator, as we are currently consuming all input that happened since the last fixed timestep.
    input.0 = Vec2::ZERO;
  }
}

fn interpolate_rendered_transform(
  fixed_time: Res<Time<Fixed>>,
  mut query: Query<(
    &mut Transform,
    &PhysicalTranslation,
    &PreviousPhysicalTranslation,
  )>,
) {
  for (
    mut transform,
    current_physical_translation,
    previous_physical_translation,
  ) in query.iter_mut()
  {
    let previous = previous_physical_translation.0;
    let current = current_physical_translation.0;
    // The overstep fraction is a value between 0 and 1 that tells us how far we are between two fixed timesteps.
    let alpha = fixed_time.overstep_fraction();

    let rendered_translation = previous.lerp(current, alpha);
    transform.translation = rendered_translation;
  }
}
