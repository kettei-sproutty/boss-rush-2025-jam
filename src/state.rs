use crate::prelude::*;
use enum_iterator::Sequence;

pub struct AppStatePlugin;

impl Plugin for AppStatePlugin {
  fn build(&self, app: &mut App) {
    app
      .init_state::<AppState>()
      .enable_state_scoped_entities::<AppState>();
  }
}

/// State that describes the current state of the application,
/// that defines in which state the application is.
#[derive(
  Debug, PartialEq, Eq, Clone, Copy, Hash, Default, States, Reflect, Sequence
)]
pub enum AppState {
  /// The `default` state of the application.
  /// In this state the application is loading assets.
  #[default]
  AssetsLoading,
  /// The `main_menu` state of the application.
  /// In this state the application is displaying the main menu.
  /// The user can start a new game or quit the application.
  MainMenu,
  /// The `game` state of the application.
  /// In this state the application is running the game.
  InGame,
  /// The `game_over` state of the application.
  /// In this state the application is displaying the game over screen.
  /// The user can restart the game or return to the main menu.
  GameOver,
}
