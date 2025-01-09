use crate::prelude::*;

/// `virtual` time related marker
#[derive(Component)]
struct VirtualTime;

pub struct UiPlugin;

impl Plugin for UiPlugin {
  fn build(&self, app: &mut App) {
    app
      .add_systems(OnEnter(AppState::InGame), spawn_timer)
      .add_systems(
        Update,
        update_virtual_time_info_text.run_if(in_state(InGameState::Running)),
      );
  }
}

fn spawn_timer(mut commands: Commands) {
  let font_size = 12.;

  commands
    .spawn((
      Name::new("VirtualTime"),
      Node {
        display: Display::Flex,
        flex_direction: FlexDirection::Column,
        align_items: AlignItems::FlexEnd,
        position_type: PositionType::Absolute,
        top: Val::Px(0.),
        right: Val::Px(0.),
        row_gap: Val::Px(10.),
        padding: UiRect::all(Val::Px(20.0)),
        ..default()
      },
    ))
    .with_children(|builder| {
      // virtual time info
      builder.spawn((
        Text::default(),
        TextFont {
          font_size,
          ..default()
        },
        VirtualTime,
      ));
    });
}

/// Update the `virtual` time info text
fn update_virtual_time_info_text(
  time: Res<Time<Virtual>>,
  mut query: Query<&mut Text, With<VirtualTime>>,
) {
  for mut text in &mut query {
    let total_seconds = time.elapsed_secs();
    let hours = (total_seconds / 3600.0).floor() as u32;
    let minutes = ((total_seconds % 3600.0) / 60.0).floor() as u32;
    let seconds = (total_seconds % 60.0).floor() as u32;

    **text = format!(
      "virtual: {:02}:{:02}:{:02}",
      hours, minutes, seconds
    );
  }
}
