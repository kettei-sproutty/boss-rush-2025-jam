mod pause;
mod player;
mod ui;

use avian2d::prelude::*;

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
      .add_plugins((
        pause::PausePlugin,
        ui::UiPlugin,
        PhysicsPlugins::default().with_length_unit(20.),
        player::PlayerPlugin,
      ))
      .insert_resource(Gravity(Vec2::new(0., 0.)))
      .add_systems(
        OnEnter(self.state.clone()),
        (setup_game, spawn_example_tree),
      )
      .add_systems(
        Update,
        (animate_sprite.run_if(in_state(InGameState::Running)))
          .run_if(in_state(AppState::InGame)),
      );
  }
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
    Name::new("Tree"),
    StateScoped(AppState::InGame),
    Transform::from_xyz(250., 0., 0.),
    Sprite::from_atlas_image(
      example_assets.tree.clone(),
      TextureAtlas {
        layout: texture_atlas_layout,
        index: 0,
      },
    ),
    RigidBody::Static,
    Collider::rectangle(64.0, 64.0), // Add a collider (adjust size as needed)
    Friction::ZERO.with_combine_rule(CoefficientCombine::Min), // Optional: configure physics properties
    Restitution::ZERO.with_combine_rule(CoefficientCombine::Min),
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
