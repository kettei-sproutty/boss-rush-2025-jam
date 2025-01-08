use crate::{assets::ExampleAssets, prelude::*};

#[derive(Component, Deref, DerefMut)]
struct AnimationTimer(Timer);

#[derive(Component)]
struct AnimationIndices {
  first: usize,
  last: usize,
}

/// `Real` time related marker
#[derive(Component)]
struct RealTime;

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
          spawn_example_tree,
          spawn_timer,
        ),
      )
      .add_systems(
        OnEnter(IsPaused::Paused),
        setup_paused_screen,
      )
      .add_systems(
        Update,
        (
          animate_sprite.run_if(in_state(IsPaused::Running)),
          update_real_time_info_text.run_if(in_state(IsPaused::Running)),
          toggle_pause,
        )
          .run_if(in_state(AppState::InGame)),
      );
  }
}

fn setup_game(mut commands: Commands) {
  commands.spawn(Camera2d);
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
        RealTime,
      ));
    });
}

/// Update the `Real` time info text
fn update_real_time_info_text(
  time: Res<Time<Virtual>>,
  mut query: Query<&mut Text, With<RealTime>>,
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
    StateDespawnMarker,
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
        align_items: AlignItems::FlexEnd,
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
  if input.just_pressed(KeyCode::Space) {
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
