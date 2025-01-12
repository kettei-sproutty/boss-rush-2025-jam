use assets::UiAssets;

use crate::prelude::*;

pub struct PausePlugin;

#[derive(Component)]
struct MainMenuButton;

impl Plugin for PausePlugin {
  fn build(&self, app: &mut App) {
    app
      .add_systems(
        OnEnter(InGameState::Paused),
        setup_paused_screen,
      )
      .add_systems(
        Update,
        toggle_pause.run_if(in_state(AppState::InGame)),
      )
      .add_systems(
        Update,
        go_to_main_menu.run_if(in_state(InGameState::Paused)),
      );
  }
}

//pause setup
pub fn setup_paused_screen(mut commands: Commands, ui: Res<UiAssets>) {
  let container = commands
    .spawn((
      Name::new("PauseScreen"),
      StateScoped(InGameState::Paused),
      Node {
        width: Val::Percent(100.),
        height: Val::Percent(100.),
        position_type: PositionType::Relative,
        flex_direction: FlexDirection::Column,
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        row_gap: Val::Px(32.),
        ..Default::default()
      },
      BackgroundColor(Color::srgba(0.15, 0.15, 0.15, 0.5)),
    ))
    .id();

  let title = commands
    .spawn((
      Text::new("Paused"),
      TextFont {
        font_size: 33.0,
        ..default()
      },
      TextColor(Color::srgb(0.9, 0.9, 0.9)),
    ))
    .id();

  let main_menu_button = commands
    .spawn((Button, MainMenuButton))
    .with_children(|parent| {
      parent.spawn((
        Text::new("Return to Main Menu"),
        TextFont {
          font: ui.font.clone(),
          ..Default::default()
        },
      ));
    })
    .id();

  commands
    .entity(container)
    .add_children(&[title, main_menu_button]);
}

fn toggle_pause(
  input: Res<ButtonInput<KeyCode>>,
  current_state: Res<State<InGameState>>,
  mut next_state: ResMut<NextState<InGameState>>,
  mut time: ResMut<Time<Virtual>>,
) {
  if input.just_pressed(KeyCode::Escape) {
    let state = match current_state.get() {
      InGameState::Running => InGameState::Paused,
      InGameState::Paused => InGameState::Running,
    };

    next_state.set(state);

    if state.eq(&InGameState::Paused) {
      time.pause();
    } else {
      time.unpause();
    };
  }
}

fn go_to_main_menu(
  query: Query<
    &Interaction,
    (
      With<MainMenuButton>,
      Changed<Interaction>,
    ),
  >,
  mut next_state: ResMut<NextState<AppState>>,
) {
  for interaction in &query {
    if interaction == &Interaction::Pressed {
      next_state.set(AppState::MainMenu);
    }
  }
}
