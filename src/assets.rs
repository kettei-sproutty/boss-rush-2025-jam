use crate::prelude::*;
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
      );
  }
}

fn load_example_assets(
  mut commands: Commands,
  asset_server: Res<AssetServer>,
  mut loading: ResMut<AssetsLoading<AppState>>,
) {
  let tree: Handle<Image> = asset_server.load("tree.png");
  let player: Handle<Image> = asset_server.load("placeholder_char.png");
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
  loading.add(&font);

  commands.insert_resource(UiAssets { font });
}
