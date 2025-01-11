use bevy::window::PresentMode;

use crate::prelude::*;

#[derive(Reflect)]
pub enum Language {
  English,
}

#[derive(Resource, Reflect)]
#[reflect(Resource)]
pub struct Settings {
  pub vsync_enabled: bool,
  pub music_level: f32,
  pub sound_level: f32,
  pub language: Language,
}

impl Default for Settings {
  fn default() -> Self {
    Self {
      vsync_enabled: true,
      music_level: 1.0,
      sound_level: 1.0,
      language: Language::English,
    }
  }
}

pub struct SettingsPlugin;

impl Plugin for SettingsPlugin {
  fn build(&self, app: &mut App) {
    app
      .insert_resource(Settings::default())
      .register_type::<Settings>()
      .add_systems(Update, update_settings);
  }
}

fn update_settings(
  settings: Res<Settings>,
  mut window_query: Query<&mut Window>,
) {
  if settings.is_changed() {
    let mut window = window_query.single_mut();
    window.present_mode = match settings.vsync_enabled {
      true => PresentMode::AutoVsync,
      false => PresentMode::AutoNoVsync,
    };
  }
}
