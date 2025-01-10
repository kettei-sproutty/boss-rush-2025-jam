mod settings;

use crate::prelude::*;

#[derive(Component)]
struct PlayButton;

#[derive(Component)]
struct SettingsButton;

pub struct MainMenuPlugin<S: States> {
  pub state: S,
}

impl<S: States> Plugin for MainMenuPlugin<S> {
  fn build(&self, app: &mut App) {
    app
      .add_plugins(settings::MainMenuSettingsPlugin)
      .add_systems(
        OnEnter(MainMenuState::MainScreen),
        setup_main_menu,
      )
      .add_systems(
        Update,
        (start_game, go_to_settings).run_if(in_state(self.state.clone())),
      );
  }
}

fn setup_main_menu(mut commands: Commands, res: Res<assets::UiAssets>) {
  commands.spawn((
    StateScoped(AppState::MainMenu),
    Camera2d,
  ));

  let container = commands
    .spawn((
      StateScoped(MainMenuState::MainScreen),
      Name::new("MainMenuContainer"),
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
    ))
    .id();

  let play_button = commands
    .spawn((
      PlayButton,
      Button,
      StateOnPress::from(AppState::InGame),
    ))
    .with_children(|parent| {
      parent.spawn((
        Text::new("Play"),
        TextFont {
          font: res.font.clone(),
          font_size: 32.,
          ..Default::default()
        },
      ));
    })
    .id();

  let settings_button = commands
    .spawn((
      SettingsButton,
      Button,
      StateOnPress::from(MainMenuState::Settings),
    ))
    .with_children(|parent| {
      parent.spawn((
        Text::new("Settings"),
        TextFont {
          font: res.font.clone(),
          font_size: 32.,
          ..Default::default()
        },
      ));
    })
    .id();

  commands
    .entity(container)
    .add_children(&[play_button, settings_button]);
}

fn start_game(
  mut play_interaction_query: Query<
    (&Interaction, &StateOnPress<AppState>),
    (
      Changed<Interaction>,
      With<Button>,
      With<PlayButton>,
    ),
  >,
  mut next_app_state: ResMut<NextState<AppState>>,
) {
  for (interaction, state) in &mut play_interaction_query {
    if interaction == &Interaction::Pressed {
      next_app_state.set(state.action);
    }
  }
}

fn go_to_settings(
  mut settings_interaction_query: Query<
    (
      &Interaction,
      &StateOnPress<MainMenuState>,
    ),
    (
      Changed<Interaction>,
      With<Button>,
      With<SettingsButton>,
    ),
  >,
  mut next_main_menu_state: ResMut<NextState<MainMenuState>>,
) {
  for (interaction, state) in &mut settings_interaction_query {
    if interaction == &Interaction::Pressed {
      next_main_menu_state.set(state.action);
    }
  }
}
