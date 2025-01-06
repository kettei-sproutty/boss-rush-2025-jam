mod dev;
mod prelude;
mod state;
mod utils;

use bevy::window::{WindowMode, WindowResolution};
use state::AppStatePlugin;

use crate::prelude::*;

fn main() {
  let mut app = App::new();

  let default_plugins = DefaultPlugins;

  let default_plugins = default_plugins.set(WindowPlugin {
    primary_window: Some(Window {
      fit_canvas_to_parent: true,
      canvas: Some("#game".to_string()),
      title: "Boss Rush 2025".to_string(),
      #[cfg(not(any(feature = "web-dev", feature = "web")))]
      resolution: WindowResolution::new(1280., 720.),
      mode: WindowMode::Windowed,
      present_mode: bevy::window::PresentMode::Fifo,
      ..Default::default()
    }),
    ..Default::default()
  });

  #[cfg(target_arch = "wasm32")]
  // Disable assets meta check on wasm to throw 4xx errors
  let default_plugins = default_plugins.set(AssetPlugin {
    meta_check: bevy::asset::AssetMetaCheck::Never,
    ..Default::default()
  });

  app.add_plugins(default_plugins);

  app.add_plugins(AppStatePlugin);

  #[cfg(feature = "dev")]
  app.add_plugins(dev::DevPlugin);

  // TODO: move to state-based setup
  app.add_systems(Startup, camera_setup);

  app.run();
}

fn camera_setup(mut commands: Commands) {
  commands.spawn(Camera2d);
}
