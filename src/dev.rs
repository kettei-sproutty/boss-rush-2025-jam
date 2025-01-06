use bevy::dev_tools::fps_overlay::FpsOverlayPlugin;
use bevy::dev_tools::states::log_transitions;
use bevy::dev_tools::ui_debug_overlay::DebugUiPlugin;
use bevy_inspector_egui::quick::WorldInspectorPlugin;

use crate::prelude::*;

pub struct DevPlugin;

impl Plugin for DevPlugin {
  fn build(&self, app: &mut App) {
    app
      // bevy dev_tools plugins
      .add_plugins((
        FpsOverlayPlugin::default(),
        DebugUiPlugin,
      ))
      .add_systems(Update, log_transitions::<AppState>)
      // bevy_inspector_egui plugin
      .add_plugins(WorldInspectorPlugin::default());
  }
}
