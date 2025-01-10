use assets::UiAssets;

use crate::{prelude::*, settings};

#[derive(Component)]
struct VSyncButton;

#[derive(Component)]
struct BackButton;

pub struct MainMenuSettingsPlugin;

impl Plugin for MainMenuSettingsPlugin {
  fn build(&self, app: &mut App) {
    app
      .add_systems(
        OnEnter(MainMenuState::Settings),
        setup_main_menu_settings,
      )
      .add_systems(
        Update,
        (go_back, toggle_vsync).run_if(in_state(MainMenuState::Settings)),
      );
  }
}

fn setup_main_menu_settings(mut commands: Commands, res: Res<UiAssets>) {
  let settings_container = commands
    .spawn((
      Name::new("SettingsContainer"),
      StateScoped(MainMenuState::Settings),
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

  let vsync_toggle = commands
    .spawn((Button, VSyncButton))
    .with_children(|parent| {
      parent.spawn((
        Text::new("VSync Enabled"),
        VSyncButton,
        TextFont {
          font: res.font.clone(),
          font_size: 32.0,
          ..Default::default()
        },
      ));
    })
    .id();

  let back_button = commands
    .spawn((
      Button,
      BackButton,
      StateOnPress::from(MainMenuState::MainScreen),
    ))
    .with_children(|parent| {
      parent.spawn((
        Text::new("Back"),
        TextFont {
          font: res.font.clone(),
          font_size: 32.0,
          ..Default::default()
        },
      ));
    })
    .id();

  commands
    .entity(settings_container)
    .add_children(&[vsync_toggle, back_button]);
}

fn toggle_vsync(
  mut settings: ResMut<settings::Settings>,
  vsync_button_query: Query<
    &Interaction,
    (Changed<Interaction>, With<VSyncButton>),
  >,
  mut vsync_button_text: Query<&mut Text, With<VSyncButton>>,
) {
  for interaction in &vsync_button_query {
    if interaction == &Interaction::Pressed {
      settings.vsync_enabled = !settings.vsync_enabled;
      let mut text = vsync_button_text.single_mut();

      text.0 = if settings.vsync_enabled {
        "VSync Enabled".to_string()
      } else {
        "VSync Disabled".to_string()
      };
    }
  }
}

fn go_back(
  go_back_query: Query<
    (
      &Interaction,
      &StateOnPress<MainMenuState>,
    ),
    Changed<Interaction>,
  >,
  mut next_state: ResMut<NextState<MainMenuState>>,
) {
  for (interaction, state) in &go_back_query {
    if interaction == &Interaction::Pressed {
      next_state.set(state.action);
    }
  }
}
