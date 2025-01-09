use crate::prelude::*;

pub struct PausePlugin;

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
      );
  }
}

//pause setup
pub fn setup_paused_screen(mut commands: Commands) {
  commands
    .spawn((
      Name::new("PauseScreen"),
      StateScoped(InGameState::Paused),
      Node {
        width: Val::Percent(100.),
        height: Val::Percent(100.),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::FlexEnd,
        flex_direction: FlexDirection::Column,
        row_gap: Val::Px(10.),
        ..default()
      },
    ))
    .with_children(|parent| {
      parent
        .spawn((
          Node {
            width: Val::Px(400.),
            height: Val::Px(400.),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
          },
          BackgroundColor(Color::srgb(0.15, 0.15, 0.15)),
        ))
        .with_children(|parent| {
          parent.spawn((
            Text::new("Paused"),
            TextFont {
              font_size: 33.0,
              ..default()
            },
            TextColor(Color::srgb(0.9, 0.9, 0.9)),
          ));
        });
    });
}

fn toggle_pause(
  input: Res<ButtonInput<KeyCode>>,
  current_state: Res<State<InGameState>>,
  mut next_state: ResMut<NextState<InGameState>>,
  mut time: ResMut<Time<Virtual>>,
) {
  if input.just_pressed(KeyCode::Space) {
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
