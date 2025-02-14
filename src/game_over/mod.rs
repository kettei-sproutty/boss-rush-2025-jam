use crate::prelude::*;

#[derive(Component)]
pub struct StateOnPress<S: States> {
  pub action: S,
}

impl<S: States> StateOnPress<S> {
  fn from(state: S) -> Self {
    Self { action: state }
  }
}

pub struct GameOverPlugin<S: States> {
  pub state: S,
}

impl<S: States> Plugin for GameOverPlugin<S> {
  fn build(&self, app: &mut App) {
    app
      .add_systems(
        OnEnter(self.state.clone()),
        setup_game_over,
      )
      .add_systems(Update, action_on_press);
  }
}

fn setup_game_over(mut commands: Commands) {
  let container = commands
    .spawn((
      StateScoped(AppState::GameOver),
      Camera2d,
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

  let play_again = commands
    .spawn((
      Button,
      StateOnPress::from(AppState::InGame),
    ))
    .with_children(|parent| {
      parent.spawn(Text::new("Play again"));
    })
    .id();

  commands.entity(container).add_children(&[play_again]);
}

fn action_on_press(
  mut interaction_query: Query<
    (&Interaction, &StateOnPress<AppState>),
    (Changed<Interaction>, With<Button>),
  >,
  mut next_state: ResMut<NextState<AppState>>,
) {
  for (interaction, state) in &mut interaction_query {
    if interaction == &Interaction::Pressed {
      next_state.set(state.action);
    }
  }
}
