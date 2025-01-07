use iyes_progress::prelude::*;

use crate::prelude::*;

pub struct LoadscreenPlugin<S: States> {
  pub state: S,
}

impl<S: States> Plugin for LoadscreenPlugin<S> {
  fn build(&self, app: &mut App) {
    app.add_systems(
      OnEnter(self.state.clone()),
      setup_loadscreen,
    );
    app.add_systems(
      Last,
      update_loading_pct.run_if(in_state(self.state.clone())),
    );
  }
}

#[derive(Component)]
struct LoadingProgressIndicator;

fn setup_loadscreen(mut commands: Commands) {
  commands.spawn((StateDespawnMarker, Camera2d));

  let container = commands
    .spawn((
      StateDespawnMarker,
      Node {
        width: Val::Auto,
        height: Val::Auto,
        position_type: PositionType::Absolute,
        bottom: Val::Percent(48.0),
        top: Val::Percent(48.0),
        left: Val::Percent(20.0),
        right: Val::Percent(20.0),
        padding: UiRect::all(Val::Px(2.0)),
        flex_direction: FlexDirection::Column,
        justify_content: JustifyContent::Center,
        ..Default::default()
      },
    ))
    .id();

  let inner = commands
    .spawn((
      LoadingProgressIndicator,
      Node {
        width: Val::Percent(0.0),
        height: Val::Percent(100.0),
        ..Default::default()
      },
      BackgroundColor(bevy::prelude::Color::Srgba(
        bevy::color::palettes::tailwind::GREEN_500,
      )),
    ))
    .id();

  commands.entity(container).insert_children(0, &[inner]);
}

fn update_loading_pct(
  mut q: Query<&mut Node, With<LoadingProgressIndicator>>,
  progress: Res<ProgressTracker<AppState>>,
) {
  let progress: f32 = progress.get_global_progress().into();
  println!("update_loading_pct {}", progress);
  for mut style in q.iter_mut() {
    style.width = Val::Percent(progress * 100.0);
  }
}
