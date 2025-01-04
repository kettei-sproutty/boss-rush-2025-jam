use bevy::dev_tools::fps_overlay::{FpsOverlayConfig, FpsOverlayPlugin};

use crate::prelude::*;

pub struct DevPlugin;

impl Plugin for DevPlugin {
  fn build(&self, app: &mut App) {
    app.add_plugins(FpsOverlayPlugin::default());
  }
}

