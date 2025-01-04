mod prelude;
mod dev;

use crate::prelude::*;

fn main() {
  let mut app = App::new();

  app.add_plugins(DefaultPlugins);

  #[cfg(feature = "dev")]
  app.add_plugins(dev::DevPlugin);

  // TODO: move to state-based setup
  app.add_systems(Startup, camera_setup);

  app.run();
}

fn camera_setup(mut commands: Commands) {
  commands.spawn(Camera2d);
}
