mod prelude;
mod dev;

use crate::prelude::*;

fn main() {
  let mut app = App::new();

  let default_plugins = DefaultPlugins;

  #[cfg(any(feature = "web", feature = "web-dev"))]
  let default_plugins = default_plugins.set(WindowPlugin {
    primary_window: Some(Window {
      fit_canvas_to_parent: true,
      canvas: Some("#game".to_string()),
      ..Default::default()
    }),
    ..Default::default()
  });

  #[cfg(not(any(feature = "web-dev", feature = "web")))]
  let default_plugins = default_plugins.set(WindowPlugin {
    primary_window: Some(Window {
      title: "Boss Rush 2025".to_string(),
      ..Default::default()
    }),
    ..Default::default()
  });

  app.add_plugins(default_plugins);

  #[cfg(feature = "dev")]
  app.add_plugins(dev::DevPlugin);

  // TODO: move to state-based setup
  app.add_systems(Startup, camera_setup);

  app.run();
}

fn camera_setup(mut commands: Commands) {
  commands.spawn(Camera2d);
}
