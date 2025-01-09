use crate::{assets::ExampleAssets, prelude::*};

#[derive(Component, Deref, DerefMut)]
struct AnimationTimer(Timer);

#[derive(Component)]
struct AnimationIndices {
  first: usize,
  last: usize,
}

#[derive(Component)]
struct VirtualTime;

pub struct GamePlugin<S: States> {
  pub state: S,
}

// In this case, instead of deriving `States`, we derive `SubStates`
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, SubStates)]
// And we need to add an attribute to let us know what the source state is
// and what value it needs to have. This will ensure that unless we're
// in [`AppState::InGame`], the [`IsPaused`] state resource
// will not exist.
#[source(AppState = AppState::InGame)]
enum IsPaused {
  #[default]
  Running,
  Paused,
}

impl<S: States> Plugin for GamePlugin<S> {
  fn build(&self, app: &mut App) {
    app
      .add_sub_state::<IsPaused>()
      .enable_state_scoped_entities::<IsPaused>()
      .add_systems(
        OnEnter(self.state.clone()),
        (
          setup_game,
          spawn_player,
          spawn_example_tree,
          spawn_timer,
        ),
      )
      .add_systems(
        OnEnter(IsPaused::Paused),
        setup_paused_screen,
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
            .run_if(in_state(IsPaused::Running)),
          // The player's visual representation needs to be updated after the physics simulation has been advanced.
          // This could be run in `Update`, but if we run it here instead, the systems in `Update`
          // will be working with the `Transform` that will actually be shown on screen.
          interpolate_rendered_transform
            .in_set(RunFixedMainLoopSystem::AfterFixedMainLoop)
            .run_if(in_state(IsPaused::Running)),
        ),
      )
      .add_systems(
        Update,
        (
          animate_sprite.run_if(in_state(IsPaused::Running)),
          //move_player.run_if(in_state(IsPaused::Running)),
          update_real_time_info_text.run_if(in_state(IsPaused::Running)),
          toggle_pause,
        )
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

fn setup_game(
  mut commands: Commands,
  mut meshes: ResMut<Assets<Mesh>>,
  mut materials: ResMut<Assets<ColorMaterial>>,
) {
  commands.spawn(Camera2d);

  // World where we move the player
  commands.spawn((
    Mesh2d(meshes.add(Rectangle::new(1000., 700.))),
    MeshMaterial2d(materials.add(Color::srgb(0.2, 0.2, 0.3))),
  ));
}

/// Spawn the player sprite and a 2D camera.
fn spawn_player(
  mut commands: Commands,
  mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
  example_assets: Res<ExampleAssets>,
) {
  let layout =
    TextureAtlasLayout::from_grid(UVec2::splat(64), 16, 1, None, None);

  let texture_atlas_layout = texture_atlas_layouts.add(layout);
  commands.spawn((
    Name::new("Player"),
    Sprite::from_atlas_image(
      example_assets.tree.clone(),
      TextureAtlas {
        layout: texture_atlas_layout,
        index: 0,
      },
    ),
    Transform::from_scale(Vec3::splat(1.)),
    AccumulatedInput::default(),
    Velocity::default(),
    AnimationIndices { first: 1, last: 6 },
    PhysicalTranslation::default(),
    PreviousPhysicalTranslation::default(),
    AnimationTimer(Timer::from_seconds(
      0.1,
      TimerMode::Repeating,
    )),
  ));
}

fn spawn_timer(mut commands: Commands) {
  let font_size = 12.;

  commands
    .spawn(Node {
      display: Display::Flex,
      flex_direction: FlexDirection::Column,
      align_items: AlignItems::FlexEnd,
      position_type: PositionType::Absolute,
      top: Val::Px(0.),
      right: Val::Px(0.),
      row_gap: Val::Px(10.),
      padding: UiRect::all(Val::Px(20.0)),
      ..default()
    })
    .with_children(|builder| {
      // real time info
      builder.spawn((
        Text::default(),
        TextFont {
          font_size,
          ..default()
        },
        VirtualTime,
      ));
    });
}

/// Update the `Virtual` time info text
fn update_real_time_info_text(
  time: Res<Time<Virtual>>,
  mut query: Query<&mut Text, With<VirtualTime>>,
) {
  for mut text in &mut query {
    let total_seconds = time.elapsed_secs();
    let hours = (total_seconds / 3600.0).floor() as u32;
    let minutes = ((total_seconds % 3600.0) / 60.0).floor() as u32;
    let seconds = (total_seconds % 60.0).floor() as u32;

    **text = format!(
      "Real: {:02}:{:02}:{:02}",
      hours, minutes, seconds
    );
  }
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

//pause setup
pub fn setup_paused_screen(mut commands: Commands) {
  commands
    .spawn((
      StateScoped(IsPaused::Paused),
      Node {
        width: Val::Percent(100.),
        height: Val::Percent(100.),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        flex_direction: FlexDirection::Column,
        row_gap: Val::Px(10.),
        ..default()
      },
    ))
    .with_children(|parent| {
      parent
        .spawn((
          Node {
            width: Val::Px(400.),
            height: Val::Px(400.),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
          },
          BackgroundColor(Color::srgb(0.15, 0.15, 0.15)),
        ))
        .with_children(|parent| {
          parent.spawn((
            Text::new("Paused"),
            TextFont {
              font_size: 33.0,
              ..default()
            },
            TextColor(Color::srgb(0.9, 0.9, 0.9)),
          ));
        });
    });
}

fn toggle_pause(
  input: Res<ButtonInput<KeyCode>>,
  current_state: Res<State<IsPaused>>,
  mut next_state: ResMut<NextState<IsPaused>>,
  mut time: ResMut<Time<Virtual>>,
) {
  if input.just_pressed(KeyCode::Escape) {
    let state = match current_state.get() {
      IsPaused::Running => IsPaused::Paused,
      IsPaused::Paused => IsPaused::Running,
    };

    next_state.set(state);

    if state.eq(&IsPaused::Paused) {
      time.pause();
    } else {
      time.unpause();
    };
  }
}

fn handle_input(
  keyboard_input: Res<ButtonInput<KeyCode>>,
  mut query: Query<(&mut AccumulatedInput, &mut Velocity)>,
) {
  // Since Bevy's default 2D camera setup is scaled such that
  // one unit is one pixel, you can think of this as
  // "How many pixels per second should the player move?"
  let mut speed: f32 = 210.0;

  for (mut input, mut velocity) in query.iter_mut() {
    if keyboard_input.pressed(KeyCode::KeyW) {
      input.y += 1.0;
    }
    if keyboard_input.pressed(KeyCode::KeyS) {
      input.y -= 1.0;
    }
    if keyboard_input.pressed(KeyCode::KeyA) {
      input.x -= 1.0;
    }
    if keyboard_input.pressed(KeyCode::KeyD) {
      input.x += 1.0;
    }
    if keyboard_input.pressed(KeyCode::ShiftLeft) {
      speed = 300.0;
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
