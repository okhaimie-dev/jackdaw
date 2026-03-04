use std::path::PathBuf;

use bevy::{
    feathers::theme::ThemedText,
    prelude::*,
    tasks::{AsyncComputeTaskPool, Task, futures_lite::future},
    ui_widgets::observe,
    window::{PrimaryWindow, RawHandleWrapper},
};
use jackdaw_feathers::{file_browser, icons, icons::IconFont, tokens};
use jackdaw_widgets::file_browser::{FileBrowserItem, FileItemDoubleClicked};
use rfd::AsyncFileDialog;

use crate::{
    EditorEntity,
    brush::{BrushEditMode, BrushSelection, EditMode},
    texture_browser::{ApplyTextureToFaces, to_asset_relative_path},
};

pub struct AssetBrowserPlugin;

impl Plugin for AssetBrowserPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AssetBrowserState>()
            .add_systems(OnEnter(crate::AppState::Editor), setup_initial_directory)
            .add_systems(
                Update,
                (refresh_browser_on_change, poll_asset_browser_folder)
                    .run_if(in_state(crate::AppState::Editor)),
            )
            .add_observer(handle_file_double_click);
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum BrowserViewMode {
    #[default]
    Grid,
    List,
}

#[derive(Resource)]
pub struct AssetBrowserState {
    pub current_directory: PathBuf,
    pub root_directory: PathBuf,
    pub filter: String,
    pub view_mode: BrowserViewMode,
    pub needs_refresh: bool,
    pub entries: Vec<DirEntry>,
}

impl Default for AssetBrowserState {
    fn default() -> Self {
        let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        Self {
            current_directory: cwd.clone(),
            root_directory: cwd,
            filter: String::new(),
            view_mode: BrowserViewMode::Grid,
            needs_refresh: true,
            entries: Vec::new(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct DirEntry {
    pub path: PathBuf,
    pub file_name: String,
    pub is_directory: bool,
}

/// Marker for the asset browser panel container.
#[derive(Component)]
pub struct AssetBrowserPanel;

/// Marker for the asset browser content area (where items are displayed).
#[derive(Component)]
pub struct AssetBrowserContent;

/// Marker for the breadcrumb bar.
#[derive(Component)]
pub struct AssetBrowserBreadcrumb;

/// Marker for the root directory label text.
#[derive(Component)]
struct AssetBrowserRootLabel;

/// Resource holding the async folder picker task for the asset browser.
#[derive(Resource)]
struct AssetBrowserFolderTask(Task<Option<rfd::FileHandle>>);

fn setup_initial_directory(
    mut state: ResMut<AssetBrowserState>,
    project_root: Option<Res<crate::project::ProjectRoot>>,
) {
    // Use ProjectRoot if available, otherwise fall back to CWD.
    if let Some(project) = project_root {
        let assets_dir = project.assets_dir();
        state.root_directory = assets_dir.clone();
        state.current_directory = assets_dir;
    } else {
        let assets_dir = state.root_directory.join("assets");
        if assets_dir.is_dir() {
            state.current_directory = assets_dir.clone();
            state.root_directory = assets_dir;
        }
    }
    state.needs_refresh = true;
}

fn refresh_browser_on_change(
    mut state: ResMut<AssetBrowserState>,
    mut commands: Commands,
    icon_font: Res<IconFont>,
    content_query: Query<(Entity, Option<&Children>), With<AssetBrowserContent>>,
    breadcrumb_query: Query<(Entity, Option<&Children>), With<AssetBrowserBreadcrumb>>,
    mut root_label_query: Query<&mut Text, With<AssetBrowserRootLabel>>,
) {
    if !state.needs_refresh {
        return;
    }
    state.needs_refresh = false;

    // Scan directory
    state.entries.clear();
    if let Ok(read_dir) = std::fs::read_dir(&state.current_directory) {
        let mut entries: Vec<DirEntry> = read_dir
            .filter_map(|entry| {
                let entry = entry.ok()?;
                let file_name = entry.file_name().to_string_lossy().to_string();
                // Skip hidden files
                if file_name.starts_with('.') {
                    return None;
                }
                // Apply filter
                if !state.filter.is_empty()
                    && !file_name
                        .to_lowercase()
                        .contains(&state.filter.to_lowercase())
                {
                    return None;
                }
                Some(DirEntry {
                    path: entry.path(),
                    file_name,
                    is_directory: entry.file_type().ok()?.is_dir(),
                })
            })
            .collect();

        // Sort: directories first, then alphabetically
        entries.sort_by(|a, b| {
            b.is_directory
                .cmp(&a.is_directory)
                .then_with(|| a.file_name.to_lowercase().cmp(&b.file_name.to_lowercase()))
        });

        state.entries = entries;
    }

    // Clear content area children
    let Ok((content_entity, content_children)) = content_query.single() else {
        return;
    };
    if let Some(children) = content_children {
        for child in children.iter() {
            commands.entity(child).despawn();
        }
    }

    // Spawn items
    for entry in &state.entries {
        let item = FileBrowserItem {
            path: entry.path.to_string_lossy().to_string(),
            is_directory: entry.is_directory,
            file_name: entry.file_name.clone(),
        };

        let path_for_click = entry.path.to_string_lossy().to_string();
        let is_dir = entry.is_directory;

        let item_entity = match state.view_mode {
            BrowserViewMode::Grid => commands
                .spawn((
                    file_browser::file_browser_item(&item, &icon_font),
                    ChildOf(content_entity),
                ))
                .id(),
            BrowserViewMode::List => commands
                .spawn((
                    file_browser::file_browser_list_item(&item, &icon_font),
                    ChildOf(content_entity),
                ))
                .id(),
        };

        // Hover effects
        commands.entity(item_entity).observe(
            |hover: On<Pointer<Over>>, mut bg: Query<&mut BackgroundColor>| {
                if let Ok(mut bg) = bg.get_mut(hover.event_target()) {
                    bg.0 = tokens::HOVER_BG;
                }
            },
        );
        commands.entity(item_entity).observe(
            |out: On<Pointer<Out>>, mut bg: Query<&mut BackgroundColor>| {
                if let Ok(mut bg) = bg.get_mut(out.event_target()) {
                    bg.0 = Color::NONE;
                }
            },
        );
        // Click handler — trigger FileItemDoubleClicked
        commands.entity(item_entity).observe(
            move |_: On<Pointer<Click>>, mut commands: Commands| {
                commands.trigger(FileItemDoubleClicked {
                    path: path_for_click.clone(),
                    is_directory: is_dir,
                });
            },
        );
    }

    // Update breadcrumb
    let Ok((breadcrumb_entity, breadcrumb_children)) = breadcrumb_query.single() else {
        return;
    };
    if let Some(children) = breadcrumb_children {
        for child in children.iter() {
            commands.entity(child).despawn();
        }
    }

    let relative = state
        .current_directory
        .strip_prefix(&state.root_directory)
        .unwrap_or(&state.current_directory);
    let path_str = relative.to_string_lossy().to_string();

    commands.spawn((
        Text::new(if path_str.is_empty() {
            "/".to_string()
        } else {
            format!("/ {}", path_str.replace(std::path::MAIN_SEPARATOR, " / "))
        }),
        TextFont {
            font_size: tokens::FONT_SM,
            ..Default::default()
        },
        ThemedText,
        ChildOf(breadcrumb_entity),
    ));

    // Update root label
    for mut text in root_label_query.iter_mut() {
        **text = state.root_directory.to_string_lossy().to_string();
    }
}

fn handle_file_double_click(
    event: On<FileItemDoubleClicked>,
    mut state: ResMut<AssetBrowserState>,
    edit_mode: Res<EditMode>,
    brush_selection: Res<BrushSelection>,
    mut commands: Commands,
) {
    if event.is_directory {
        state.current_directory = PathBuf::from(&event.path);
        state.needs_refresh = true;
        return;
    }

    // If in face edit mode with faces selected and double-clicking an image, apply it
    if *edit_mode == EditMode::BrushEdit(BrushEditMode::Face)
        && !brush_selection.faces.is_empty()
        && brush_selection.entity.is_some()
    {
        if is_image_file(&event.path) {
            if let Some(relative) = to_asset_relative_path(&event.path) {
                commands.trigger(ApplyTextureToFaces { path: relative });
            }
        }
    }
}

fn spawn_asset_folder_dialog(
    _: On<Pointer<Click>>,
    mut commands: Commands,
    raw_handle: Query<&RawHandleWrapper, With<PrimaryWindow>>,
) {
    let mut dialog = AsyncFileDialog::new().set_title("Select assets directory");
    if let Ok(rh) = raw_handle.single() {
        let handle = unsafe { rh.get_handle() };
        dialog = dialog.set_parent(&handle);
    }
    let task = AsyncComputeTaskPool::get().spawn(async move { dialog.pick_folder().await });
    commands.insert_resource(AssetBrowserFolderTask(task));
}

fn poll_asset_browser_folder(world: &mut World) {
    let Some(mut task_res) = world.get_resource_mut::<AssetBrowserFolderTask>() else {
        return;
    };
    let Some(result) = future::block_on(future::poll_once(&mut task_res.0)) else {
        return;
    };
    world.remove_resource::<AssetBrowserFolderTask>();

    if let Some(handle) = result {
        let path = handle.path().to_path_buf();
        let mut state = world.resource_mut::<AssetBrowserState>();
        state.root_directory = path.clone();
        state.current_directory = path.clone();
        state.needs_refresh = true;

        // Update the root label text
        let mut label_query = world.query_filtered::<&mut Text, With<AssetBrowserRootLabel>>();
        for mut text in label_query.iter_mut(world) {
            **text = path.to_string_lossy().to_string();
        }
    }
}

fn is_image_file(path: &str) -> bool {
    let path_lower = path.to_lowercase();
    path_lower.ends_with(".png")
        || path_lower.ends_with(".jpg")
        || path_lower.ends_with(".jpeg")
        || path_lower.ends_with(".bmp")
        || path_lower.ends_with(".tga")
        || path_lower.ends_with(".webp")
}

pub fn asset_browser_panel(icon_font: Handle<Font>) -> impl Bundle {
    let folder_icon_font = icon_font;
    (
        AssetBrowserPanel,
        EditorEntity,
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            padding: UiRect::all(Val::Px(tokens::SPACING_SM)),
            ..Default::default()
        },
        BackgroundColor(tokens::PANEL_BG),
        children![
            // Root directory header
            (
                Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::SpaceBetween,
                    width: Val::Percent(100.0),
                    height: Val::Px(tokens::ROW_HEIGHT),
                    padding: UiRect::horizontal(Val::Px(tokens::SPACING_MD)),
                    flex_shrink: 0.0,
                    ..Default::default()
                },
                BackgroundColor(tokens::PANEL_HEADER_BG),
                children![
                    // Left side: title + path
                    (
                        Node {
                            flex_direction: FlexDirection::Row,
                            align_items: AlignItems::Center,
                            column_gap: Val::Px(tokens::SPACING_MD),
                            overflow: Overflow::clip(),
                            flex_shrink: 1.0,
                            ..Default::default()
                        },
                        children![
                            (
                                Text::new("Assets"),
                                TextFont {
                                    font_size: tokens::FONT_MD,
                                    ..Default::default()
                                },
                                ThemedText,
                            ),
                            (
                                AssetBrowserRootLabel,
                                Text::new(""),
                                TextFont {
                                    font_size: tokens::FONT_SM,
                                    ..Default::default()
                                },
                                TextColor(tokens::TEXT_SECONDARY),
                            ),
                        ],
                    ),
                    // Right side: folder picker button
                    asset_folder_button(folder_icon_font),
                ],
            ),
            // Breadcrumb bar
            (
                AssetBrowserBreadcrumb,
                EditorEntity,
                Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    padding: UiRect::axes(Val::Px(tokens::SPACING_MD), Val::Px(tokens::SPACING_SM)),
                    width: Val::Percent(100.0),
                    height: Val::Px(tokens::HEADER_HEIGHT),
                    flex_shrink: 0.0,
                    ..Default::default()
                },
                BackgroundColor(tokens::TOOLBAR_BG),
            ),
            // Content area
            (
                AssetBrowserContent,
                EditorEntity,
                Node {
                    flex_direction: FlexDirection::Row,
                    flex_wrap: FlexWrap::Wrap,
                    align_content: AlignContent::FlexStart,
                    width: Val::Percent(100.0),
                    flex_grow: 1.0,
                    min_height: Val::Px(0.0),
                    overflow: Overflow::scroll_y(),
                    padding: UiRect::all(Val::Px(tokens::SPACING_SM)),
                    row_gap: Val::Px(tokens::SPACING_SM),
                    column_gap: Val::Px(tokens::SPACING_SM),
                    ..Default::default()
                },
            )
        ],
    )
}

fn asset_folder_button(icon_font: Handle<Font>) -> impl Bundle {
    (
        Node {
            padding: UiRect::all(Val::Px(tokens::SPACING_XS)),
            border_radius: BorderRadius::all(Val::Px(tokens::BORDER_RADIUS_SM)),
            ..Default::default()
        },
        icons::icon_colored(
            icons::Icon::FolderOpen,
            tokens::FONT_MD,
            icon_font,
            tokens::TEXT_SECONDARY,
        ),
        observe(spawn_asset_folder_dialog),
    )
}
