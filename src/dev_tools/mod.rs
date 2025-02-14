// TODO: refactor this
// TODO: wait https://github.com/jakobhellermann/bevy-inspector-egui/pull/233 to be merged
// TODO: remove unused components
use crate::prelude::*;

use bevy::asset::{ReflectAsset, UntypedAssetId};
use bevy::dev_tools::fps_overlay::{FpsOverlayConfig, FpsOverlayPlugin};
use bevy::dev_tools::states::log_transitions;
use bevy::dev_tools::ui_debug_overlay::DebugUiPlugin;
use bevy::reflect::TypeRegistry;
use bevy::render::camera::Viewport;
use bevy::window::{PrimaryWindow, Window};
use bevy_egui::{EguiContext, EguiContextSettings, EguiPostUpdateSet};
use bevy_inspector_egui::bevy_inspector::hierarchy::{
  hierarchy_ui, SelectedEntities,
};
use bevy_inspector_egui::bevy_inspector::{
  self, ui_for_entities_shared_components, ui_for_entity_with_children,
};
use bevy_inspector_egui::DefaultInspectorConfigPlugin;
use egui_dock::{DockArea, DockState, NodeIndex, Style};
use std::any::TypeId;

#[cfg(egui_dock_gizmo)]
use transform_gizmo_egui::GizmoMode;

/// Placeholder type if gizmo is disabled.
#[cfg(not(egui_dock_gizmo))]
#[derive(Clone, Copy)]
struct GizmoMode;

pub struct DevToolsPlugin;

impl Plugin for DevToolsPlugin {
  fn build(&self, app: &mut App) {
    let fps_plugin = FpsOverlayPlugin {
      config: FpsOverlayConfig {
        text_config: TextFont {
          font_size: 14.0,
          ..Default::default()
        },
        ..Default::default()
      },
    };

    app
      .add_plugins((
        fps_plugin,
        DebugUiPlugin,
        DefaultInspectorConfigPlugin,
        bevy_egui::EguiPlugin,
      ))
      .insert_resource(UiState::new())
      .add_systems(
        PostUpdate,
        show_ui_system
          .before(EguiPostUpdateSet::ProcessOutput)
          .before(bevy_egui::end_pass_system)
          .before(bevy::transform::TransformSystem::TransformPropagate),
      )
      .add_systems(
        PostUpdate,
        set_camera_viewport.after(show_ui_system),
      )
      .add_systems(Update, set_gizmo_mode)
      .add_systems(
        Update,
        (
          log_transitions::<AppState>,
          log_transitions::<MainMenuState>,
          log_transitions::<InGameState>,
        ),
      )
      .register_type::<Option<Handle<Image>>>()
      .register_type::<AlphaMode>();
  }
}

#[derive(Eq, PartialEq)]
enum InspectorSelection {
  Entities,
  Resource(TypeId, String),
  Asset(TypeId, String, UntypedAssetId),
}

#[derive(Resource)]
struct UiState {
  state: DockState<EguiWindow>,
  viewport_rect: egui::Rect,
  selected_entities: SelectedEntities,
  selection: InspectorSelection,
  gizmo_mode: GizmoMode,
}

impl UiState {
  pub fn new() -> Self {
    let mut state = DockState::new(vec![EguiWindow::GameView]);
    let tree = state.main_surface_mut();
    let [game, _inspector] = tree.split_right(
      NodeIndex::root(),
      0.75,
      vec![EguiWindow::Inspector],
    );
    let [game, _hierarchy] =
      tree.split_left(game, 0.2, vec![EguiWindow::Hierarchy]);
    let [_game, _bottom] = tree.split_below(
      game,
      0.8,
      vec![EguiWindow::Resources, EguiWindow::Assets],
    );

    Self {
      state,
      selected_entities: SelectedEntities::default(),
      selection: InspectorSelection::Entities,
      viewport_rect: egui::Rect::NOTHING,
      #[cfg(egui_dock_gizmo)]
      gizmo_mode: GizmoMode::Translate,
      #[cfg(not(egui_dock_gizmo))]
      gizmo_mode: GizmoMode,
    }
  }

  fn ui(&mut self, world: &mut World, ctx: &mut egui::Context) {
    let mut tab_viewer = TabViewer {
      world,
      viewport_rect: &mut self.viewport_rect,
      selected_entities: &mut self.selected_entities,
      selection: &mut self.selection,
      gizmo_mode: self.gizmo_mode,
    };
    DockArea::new(&mut self.state)
      .style(Style::from_egui(ctx.style().as_ref()))
      .show(ctx, &mut tab_viewer);
  }
}

#[derive(Debug)]
enum EguiWindow {
  GameView,
  Hierarchy,
  Resources,
  Assets,
  Inspector,
}

struct TabViewer<'a> {
  world: &'a mut World,
  selected_entities: &'a mut SelectedEntities,
  selection: &'a mut InspectorSelection,
  viewport_rect: &'a mut egui::Rect,
  gizmo_mode: GizmoMode,
}

impl egui_dock::TabViewer for TabViewer<'_> {
  type Tab = EguiWindow;

  fn ui(&mut self, ui: &mut egui_dock::egui::Ui, window: &mut Self::Tab) {
    let type_registry = self.world.resource::<AppTypeRegistry>().0.clone();
    let type_registry = type_registry.read();

    match window {
      EguiWindow::GameView => {
        *self.viewport_rect = ui.clip_rect();
        draw_gizmo(
          ui,
          self.world,
          self.selected_entities,
          self.gizmo_mode,
        );
      }
      EguiWindow::Hierarchy => {
        let selected = hierarchy_ui(self.world, ui, self.selected_entities);
        if selected {
          *self.selection = InspectorSelection::Entities;
        }
      }
      EguiWindow::Resources => {
        select_resource(ui, &type_registry, self.selection)
      }
      EguiWindow::Assets => select_asset(
        ui,
        &type_registry,
        self.world,
        self.selection,
      ),
      EguiWindow::Inspector => match *self.selection {
        InspectorSelection::Entities => match self.selected_entities.as_slice()
        {
          &[entity] => ui_for_entity_with_children(self.world, entity, ui),
          entities => {
            ui_for_entities_shared_components(self.world, entities, ui)
          }
        },
        InspectorSelection::Resource(type_id, ref name) => {
          ui.label(name);
          bevy_inspector::by_type_id::ui_for_resource(
            self.world,
            type_id,
            ui,
            name,
            &type_registry,
          )
        }
        InspectorSelection::Asset(type_id, ref name, handle) => {
          ui.label(name);
          bevy_inspector::by_type_id::ui_for_asset(
            self.world,
            type_id,
            handle,
            ui,
            &type_registry,
          );
        }
      },
    }
  }

  fn title(&mut self, window: &mut Self::Tab) -> egui_dock::egui::WidgetText {
    format!("{window:?}").into()
  }

  fn clear_background(&self, window: &Self::Tab) -> bool {
    !matches!(window, EguiWindow::GameView)
  }
}

fn show_ui_system(world: &mut World) {
  let Ok(egui_context) = world
    .query_filtered::<&mut EguiContext, With<PrimaryWindow>>()
    .get_single(world)
  else {
    return;
  };
  let mut egui_context = egui_context.clone();

  world.resource_scope::<UiState, _>(|world, mut ui_state| {
    ui_state.ui(world, egui_context.get_mut())
  });
}

fn set_camera_viewport(
  ui_state: Res<UiState>,
  primary_window: Query<&mut Window, With<PrimaryWindow>>,
  egui_settings: Query<&EguiContextSettings>,
  mut cameras: Query<&mut Camera>,
) {
  let Ok(window) = primary_window.get_single() else {
    return;
  };

  let scale_factor =
    window.scale_factor() * egui_settings.single().scale_factor;

  let viewport_pos = ui_state.viewport_rect.left_top().to_vec2() * scale_factor;
  let viewport_size = ui_state.viewport_rect.size() * scale_factor;

  let physical_position = UVec2::new(
    viewport_pos.x as u32,
    viewport_pos.y as u32,
  );
  let physical_size = UVec2::new(
    viewport_size.x as u32,
    viewport_size.y as u32,
  );

  let rect = physical_position + physical_size;

  let window_size = window.physical_size();

  for mut camera in cameras.iter_mut() {
    if rect.x <= window_size.x && rect.y <= window_size.y {
      camera.viewport = Some(Viewport {
        physical_position,
        physical_size,
        depth: 0.0..1.0,
      });
    }
  }
}

fn set_gizmo_mode(
  input: Res<ButtonInput<KeyCode>>,
  mut ui_state: ResMut<UiState>,
) {
  #[cfg(egui_dock_gizmo)]
  let keybinds = [
    (KeyCode::KeyR, GizmoMode::Rotate),
    (KeyCode::KeyT, GizmoMode::Translate),
    (KeyCode::KeyS, GizmoMode::Scale),
  ];
  #[cfg(not(egui_dock_gizmo))]
  let keybinds = [];

  for (key, mode) in keybinds {
    if input.just_pressed(key) {
      ui_state.gizmo_mode = mode;
    }
  }
}

#[allow(unused)]
fn draw_gizmo(
  ui: &mut egui::Ui,
  world: &mut World,
  selected_entities: &SelectedEntities,
  gizmo_mode: GizmoMode,
) {
  if selected_entities.len() != 1 {}

  // Note: Implement gizmo drawing logic here based on your camera system
}

fn select_resource(
  ui: &mut egui::Ui,
  type_registry: &TypeRegistry,
  selection: &mut InspectorSelection,
) {
  let mut resources: Vec<_> = type_registry
    .iter()
    .filter(|registration| registration.data::<ReflectResource>().is_some())
    .map(|registration| {
      (
        registration.type_info().type_path_table().short_path(),
        registration.type_id(),
      )
    })
    .collect();
  resources.sort_by(|(name_a, _), (name_b, _)| name_a.cmp(name_b));

  for (resource_name, type_id) in resources {
    let selected = match *selection {
      InspectorSelection::Resource(selected, _) => selected == type_id,
      _ => false,
    };

    if ui.selectable_label(selected, resource_name).clicked() {
      *selection =
        InspectorSelection::Resource(type_id, resource_name.to_string());
    }
  }
}

fn select_asset(
  ui: &mut egui::Ui,
  type_registry: &TypeRegistry,
  world: &World,
  selection: &mut InspectorSelection,
) {
  let mut assets: Vec<_> = type_registry
    .iter()
    .filter_map(|registration| {
      let reflect_asset = registration.data::<ReflectAsset>()?;
      Some((
        registration.type_info().type_path_table().short_path(),
        registration.type_id(),
        reflect_asset,
      ))
    })
    .collect();
  assets.sort_by(|(name_a, ..), (name_b, ..)| name_a.cmp(name_b));

  for (asset_name, asset_type_id, reflect_asset) in assets {
    let handles: Vec<_> = reflect_asset.ids(world).collect();

    ui.collapsing(
      format!("{asset_name} ({})", handles.len()),
      |ui| {
        for handle in handles {
          let selected = match *selection {
            InspectorSelection::Asset(_, _, selected_id) => {
              selected_id == handle
            }
            _ => false,
          };

          if ui
            .selectable_label(selected, format!("{:?}", handle))
            .clicked()
          {
            *selection = InspectorSelection::Asset(
              asset_type_id,
              asset_name.to_string(),
              handle,
            );
          }
        }
      },
    );
  }
}
