use bevy::dev_tools::fps_overlay::FpsOverlayPlugin;
use bevy::dev_tools::states::log_transitions;
use bevy::dev_tools::ui_debug_overlay::DebugUiPlugin;

use crate::prelude::*;

pub struct DevPlugin;

impl Plugin for DevPlugin {
  fn build(&self, app: &mut App) {
    app
      .add_plugins(FpsOverlayPlugin::default())
      .add_plugins(DebugUiPlugin)
      .add_systems(Update, log_transitions::<AppState>);
  }
}
