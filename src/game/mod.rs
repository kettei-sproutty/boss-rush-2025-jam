use bevy::time::update_virtual_time;
use bevy_inspector_egui::egui::style;

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

/// `Virtual` time related marker
#[derive(Component)]
struct VirtualTime;

pub struct GamePlugin<S: States> {
  pub state: S,
}

impl<S: States> Plugin for GamePlugin<S> {
  fn build(&self, app: &mut App) {
    app
      .add_systems(
        OnEnter(self.state.clone()),
        (
          setup_game,
          spawn_example_tree,
          spawn_timer,
        ),
      )
      .add_systems(
        Update,
        (
          animate_sprite,
          update_virtual_time_info_text,
          update_real_time_info_text,
        ),
      );
  }
}

fn setup_game(mut commands: Commands) {
  commands.spawn((StateDespawnMarker, Camera2d));
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

      // virtual time info
      builder.spawn((
        Text::default(),
        TextFont {
          font_size,
          ..default()
        },
        TextColor(Color::srgb(0.85, 0.85, 0.85)),
        TextLayout::new_with_justify(JustifyText::Right),
        VirtualTime,
      ));
    });
}

/// Update the `Real` time info text
fn update_real_time_info_text(
  time: Res<Time<Real>>,
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

/// Update the `Virtual` time info text
fn update_virtual_time_info_text(
  time: Res<Time<Virtual>>,
  mut query: Query<&mut Text, With<VirtualTime>>,
) {
  for mut text in &mut query {
    let total_seconds = time.elapsed_secs();
    let hours = (total_seconds / 3600.0).floor() as u32;
    let minutes = ((total_seconds % 3600.0) / 60.0).floor() as u32;
    let seconds = (total_seconds % 60.0).floor() as u32;

    **text = format!(
      "Virtual: {:02}:{:02}:{:02}",
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
