use bevy::{
    picking::mesh_picking::ray_cast::{MeshRayCast, MeshRayCastSettings, RayCastVisibility},
    prelude::*,
    ui::UiGlobalTransform,
};
use jackdaw_api::prelude::*;

use crate::{
    JackdawDrawSystems,
    viewport::{MainViewportCamera, SceneViewport},
    viewport_util::window_to_viewport_cursor,
};

// ── Plugin ──

pub struct MeasureToolPlugin;

impl Plugin for MeasureToolPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MeasureToolState>()
            .init_resource::<MeasureLabelEntities>()
            .init_gizmo_group::<MeasureToolGizmoGroup>()
            .add_systems(Startup, configure_measure_tool_gizmos)
            .add_systems(
                PostUpdate,
                (draw_measure_line, update_measure_labels)
                    .in_set(JackdawDrawSystems)
                    .run_if(in_state(crate::AppState::Editor)),
            );
    }
}

#[derive(Default, Reflect, GizmoConfigGroup)]
struct MeasureToolGizmoGroup;

fn configure_measure_tool_gizmos(mut config_store: ResMut<GizmoConfigStore>) {
    let (config, _) = config_store.config_mut::<MeasureToolGizmoGroup>();
    config.depth_bias = -1.0;
}

// ── State ──

#[derive(Resource, Default, Debug, Clone, Copy)]
pub struct MeasureToolState {
    pub active: bool,
    pub start_point: Vec3,
    pub end_point: Vec3,
}

#[derive(Resource, Default)]
struct MeasureLabelEntities {
    distance: Option<Entity>,
    start: Option<Entity>,
    end: Option<Entity>,
}

#[derive(Component)]
struct MeasureLabel;

// ── Extension registration ──

pub(crate) fn add_to_extension(ctx: &mut ExtensionContext) {
    use crate::core_extension::CoreExtensionInputContext;
    use bevy_enhanced_input::prelude::Press;

    ctx.entity_mut()
        .with_related::<ActionOf<CoreExtensionInputContext>>((
            Action::<MeasureDistanceOp>::new(),
            bindings![(KeyCode::KeyM, Press::default())],
        ));

    ctx.register_operator::<MeasureDistanceOp>()
        .menu_entry_for::<MeasureDistanceOp>("Tools");
}

// ── Operator ──

#[operator(
    id = "tools.measure_distance",
    label = "Measure Distance",
    description = "Click two points in the viewport to measure the distance between them",
    modal = true,
    allows_undo = false,
    cancel = cancel_measure_distance
)]
fn measure_distance(
    _: In<OperatorParameters>,
    mut state: ResMut<MeasureToolState>,
    mouse: Res<ButtonInput<MouseButton>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    window: Single<&Window>,
    camera: Single<(&Camera, &GlobalTransform), With<MainViewportCamera>>,
    viewport_query: Query<(&ComputedNode, &UiGlobalTransform), With<SceneViewport>>,
    mut ray_cast: MeshRayCast,
) -> OperatorResult {
    let Some(cursor_pos) = window.cursor_position() else {
        return OperatorResult::Cancelled;
    };
    let (camera, cam_tf) = *camera;
    let Some(viewport_cursor) = window_to_viewport_cursor(cursor_pos, camera, &viewport_query)
    else {
        return OperatorResult::Cancelled;
    };
    let Ok(ray) = camera.viewport_to_world(cam_tf, viewport_cursor) else {
        return OperatorResult::Cancelled;
    };

    let current_point = raycast_closest_point(ray, &mut ray_cast)
        .or_else(|| ray_plane_intersection(ray, Vec3::ZERO, Vec3::Y))
        .unwrap_or(cam_tf.translation() + *ray.direction * 10.0);

    if !state.active {
        // First invocation: capture the start point and enter modal mode.
        state.active = true;
        state.start_point = current_point;
        state.end_point = current_point;
        return OperatorResult::Running;
    }

    // Update the live end point every frame.
    state.end_point = current_point;

    // Left-click commits the measurement.
    if mouse.just_pressed(MouseButton::Left) {
        state.active = false;
        return OperatorResult::Finished;
    }

    // Escape cancels.
    if keyboard.just_pressed(KeyCode::Escape) {
        state.active = false;
        return OperatorResult::Cancelled;
    }

    OperatorResult::Running
}

fn cancel_measure_distance(mut state: ResMut<MeasureToolState>) {
    state.active = false;
}

// ── Raycasting helpers ──

fn raycast_closest_point(ray: Ray3d, ray_cast: &mut MeshRayCast) -> Option<Vec3> {
    let settings = MeshRayCastSettings::default().with_visibility(RayCastVisibility::Any);
    let mut closest: Option<(Vec3, f32)> = None;
    for (_entity, hit_data) in ray_cast.cast_ray(ray, &settings) {
        if hit_data.distance >= 0.0 {
            match closest {
                None => closest = Some((hit_data.point, hit_data.distance)),
                Some((_, best_dist)) if hit_data.distance < best_dist => {
                    closest = Some((hit_data.point, hit_data.distance));
                }
                _ => {}
            }
        }
    }
    closest.map(|(point, _)| point)
}

fn ray_plane_intersection(ray: Ray3d, plane_point: Vec3, plane_normal: Vec3) -> Option<Vec3> {
    let denom = ray.direction.dot(plane_normal);
    if denom.abs() < 1e-6 {
        return None;
    }
    let t = (plane_point - ray.origin).dot(plane_normal) / denom;
    if t < 0.0 {
        return None;
    }
    Some(ray.origin + *ray.direction * t)
}

// ── Viewport drawing ──

fn draw_measure_line(mut gizmos: Gizmos<MeasureToolGizmoGroup>, state: Res<MeasureToolState>) {
    if !state.active {
        return;
    }

    let color = crate::default_style::MEASURE_TOOL_LINE;
    let start = state.start_point;
    let end = state.end_point;

    // Main measurement line
    gizmos.line(start, end, color);

    // Endpoint markers (small crosses)
    let marker_size = 0.08;
    for point in [start, end] {
        gizmos.line(
            point - Vec3::X * marker_size,
            point + Vec3::X * marker_size,
            color,
        );
        gizmos.line(
            point - Vec3::Y * marker_size,
            point + Vec3::Y * marker_size,
            color,
        );
        gizmos.line(
            point - Vec3::Z * marker_size,
            point + Vec3::Z * marker_size,
            color,
        );
    }
}

fn update_measure_labels(
    mut commands: Commands,
    state: Res<MeasureToolState>,
    mut label_entities: ResMut<MeasureLabelEntities>,
    camera: Single<(&Camera, &GlobalTransform), With<MainViewportCamera>>,
    viewport_node: Query<&ComputedNode, With<SceneViewport>>,
    mut label_query: Query<(Entity, &mut Text, &mut Node, &mut Visibility), With<MeasureLabel>>,
    viewport_entity: Single<Entity, With<SceneViewport>>,
) {
    if !state.active {
        // Tear down any labels that are still alive.
        for (entity, _, _, _) in &mut label_query {
            commands.entity(entity).despawn();
        }
        *label_entities = MeasureLabelEntities::default();
        return;
    }

    let (camera, cam_tf) = *camera;
    let vp_node_size = viewport_node
        .single()
        .map(ComputedNode::size)
        .unwrap_or(Vec2::ONE);
    let render_target_size = camera.logical_viewport_size().unwrap_or(vp_node_size);

    let start = state.start_point;
    let end = state.end_point;
    let mid = (start + end) / 2.0;
    let dist = start.distance(end);

    let distance_text = format!("{:.2}", dist);
    let start_text = format!("({:.2}, {:.2}, {:.2})", start.x, start.y, start.z);
    let end_text = format!("({:.2}, {:.2}, {:.2})", end.x, end.y, end.z);

    let mut update_label = |slot: &mut Option<Entity>, world_pos: Vec3, text_content: &str| {
        let entity = if let Some(e) = *slot {
            if label_query.contains(e) {
                if let Ok((_, mut text, mut node, mut vis)) = label_query.get_mut(e) {
                    text.0 = text_content.to_string();
                    if let Ok(vp_coords) = camera.world_to_viewport(cam_tf, world_pos) {
                        let ui_pos = vp_coords * vp_node_size / render_target_size;
                        node.left = Val::Px(ui_pos.x + 10.0);
                        node.top = Val::Px(ui_pos.y - 10.0);
                        *vis = Visibility::Inherited;
                    } else {
                        *vis = Visibility::Hidden;
                    }
                }
                e
            } else {
                spawn_measure_label(&mut commands, *viewport_entity, text_content)
            }
        } else {
            spawn_measure_label(&mut commands, *viewport_entity, text_content)
        };
        *slot = Some(entity);
    };

    update_label(&mut label_entities.distance, mid, &distance_text);
    update_label(&mut label_entities.start, start, &start_text);
    update_label(&mut label_entities.end, end, &end_text);
}

fn spawn_measure_label(commands: &mut Commands, viewport: Entity, text: &str) -> Entity {
    commands
        .spawn((
            MeasureLabel,
            Text::new(text),
            TextFont {
                font_size: 13.0,
                ..default()
            },
            TextColor(crate::default_style::MEASURE_TOOL_LABEL),
            Node {
                position_type: PositionType::Absolute,
                ..default()
            },
            Visibility::Inherited,
        ))
        .insert(ChildOf(viewport))
        .id()
}
