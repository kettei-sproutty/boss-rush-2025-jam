use crate::prelude::*;
use bevy::winit::cursor::{CursorIcon, CustomCursor};
use iyes_progress::prelude::*;

#[derive(Resource)]
pub struct ExampleAssets {
  pub tree: Handle<Image>,
  pub player: Handle<Image>,
}

#[derive(Resource, Reflect)]
#[reflect(Resource)]
pub struct UiAssets {
  pub font: Handle<Font>,
  pub cursors: Vec<Handle<Image>>,
}

pub struct AssetsLoadingPlugin;

impl Plugin for AssetsLoadingPlugin {
  fn build(&self, app: &mut App) {
    app
      .add_plugins(
        ProgressPlugin::<AppState>::new()
          .with_state_transition(
            AppState::AssetsLoading,
            AppState::MainMenu,
          )
          .with_asset_tracking(),
      )
      .add_systems(
        OnEnter(AppState::AssetsLoading),
        (load_example_assets, load_ui_assets),
      )
      .add_systems(
        OnExit(AppState::AssetsLoading),
        add_cursor,
      );
  }
}

fn load_example_assets(
  mut commands: Commands,
  asset_server: Res<AssetServer>,
  mut loading: ResMut<AssetsLoading<AppState>>,
) {
  let tree: Handle<Image> = asset_server.load("tree.png");
  let player: Handle<Image> = asset_server.load("player/player.png");

  loading.add(&tree);
  loading.add(&player);

  commands.insert_resource(ExampleAssets { tree, player });
}

fn load_ui_assets(
  mut commands: Commands,
  asset_server: Res<AssetServer>,
  mut loading: ResMut<AssetsLoading<AppState>>,
) {
  let font: Handle<Font> = asset_server.load("fonts/PixelifySans.ttf");
  let cursor_1: Handle<Image> = asset_server.load("ui/cursor1.png");
  let cursor_2: Handle<Image> = asset_server.load("ui/cursor2.png");
  let cursor_3: Handle<Image> = asset_server.load("ui/cursor3.png");
  let cursor_4: Handle<Image> = asset_server.load("ui/cursor4.png");

  loading.add(&font);
  loading.add(&cursor_1);
  loading.add(&cursor_2);
  loading.add(&cursor_3);
  loading.add(&cursor_4);

  commands.insert_resource(UiAssets {
    font,
    cursors: vec![cursor_1, cursor_2, cursor_3, cursor_4],
  });
}

fn add_cursor(
  mut commands: Commands,
  window: Single<Entity, With<Window>>,
  ui_assets: Res<UiAssets>,
) {
  commands.entity(*window).insert(
    CursorIcon::Custom(CustomCursor::Image {
      handle: ui_assets.cursors[0].clone(),
      hotspot: (0, 0),
    })
    .clone(),
  );
}
