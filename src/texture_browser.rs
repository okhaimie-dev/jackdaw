use std::path::{Path, PathBuf};

use bevy::{
    asset::RenderAssetUsages,
    feathers::theme::ThemedText,
    image::{CompressedImageFormats, ImageSampler, ImageType},
    prelude::*,
    tasks::{AsyncComputeTaskPool, Task, futures_lite::future},
    ui_widgets::observe,
    window::{PrimaryWindow, RawHandleWrapper},
};
use jackdaw_feathers::{
    icons,
    text_edit::{self, TextEditProps, TextEditValue},
    tokens,
};
use rfd::AsyncFileDialog;

use crate::{
    EditorEntity,
    brush::{Brush, BrushEditMode, BrushSelection, EditMode, LastUsedTexture, SetBrush},
    commands::CommandHistory,
};

pub struct TextureBrowserPlugin;

impl Plugin for TextureBrowserPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AvailableTextures>()
            .init_resource::<PendingTextureLoads>()
            .add_systems(OnEnter(crate::AppState::Editor), scan_textures)
            .add_systems(
                Update,
                (
                    rescan_textures,
                    apply_texture_filter,
                    update_texture_browser_ui,
                    poll_texture_browser_folder,
                    load_pending_textures,
                )
                    .run_if(in_state(crate::AppState::Editor)),
            )
            .add_observer(handle_apply_texture);
    }
}

#[derive(Resource, Default)]
pub struct AvailableTextures {
    pub textures: Vec<TextureEntry>,
    pub needs_rescan: bool,
    pub filter: String,
    pub scan_directory: PathBuf,
}

pub struct TextureEntry {
    pub path: String,
    pub file_name: String,
    pub image: Handle<Image>,
}

/// Apply a texture to currently selected brush faces.
#[derive(Event, Debug, Clone)]
pub struct ApplyTextureToFaces {
    pub path: String,
}

/// Clear texture from currently selected brush faces.
#[derive(Event, Debug, Clone)]
pub struct ClearTextureFromFaces;

fn scan_textures(
    mut available: ResMut<AvailableTextures>,
    asset_server: Res<AssetServer>,
    mut images: ResMut<Assets<Image>>,
    mut pending: ResMut<PendingTextureLoads>,
    project_root: Option<Res<crate::project::ProjectRoot>>,
) {
    let assets_dir = project_root
        .map(|p| p.assets_dir())
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_default().join("assets"));
    available.scan_directory = assets_dir.clone();
    do_scan_textures(
        &mut available,
        &asset_server,
        &mut images,
        &assets_dir,
        &mut pending.0,
    );
}

fn do_scan_textures(
    available: &mut AvailableTextures,
    asset_server: &AssetServer,
    images: &mut Assets<Image>,
    asset_root: &std::path::Path,
    pending: &mut Vec<(PathBuf, Handle<Image>)>,
) {
    available.textures.clear();
    pending.clear();

    let scan_dir = &available.scan_directory;
    if !scan_dir.is_dir() {
        return;
    }

    scan_directory_recursive(
        scan_dir,
        asset_root,
        asset_server,
        images,
        &mut available.textures,
        pending,
    );

    // Sort alphabetically
    available
        .textures
        .sort_by(|a, b| a.file_name.cmp(&b.file_name));
}

fn scan_directory_recursive(
    dir: &Path,
    asset_root: &Path,
    asset_server: &AssetServer,
    images: &mut Assets<Image>,
    entries: &mut Vec<TextureEntry>,
    pending: &mut Vec<(PathBuf, Handle<Image>)>,
) {
    let Ok(read_dir) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in read_dir.flatten() {
        let path = entry.path();
        if path.is_dir() {
            scan_directory_recursive(&path, asset_root, asset_server, images, entries, pending);
        } else if is_image_file(&path) {
            let file_name = path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();

            // Within asset root: use AssetServer for loading
            if let Ok(relative) = path.strip_prefix(asset_root) {
                let relative = relative.to_string_lossy().replace('\\', "/");
                let image: Handle<Image> = asset_server.load(relative.clone());
                entries.push(TextureEntry {
                    path: relative,
                    file_name,
                    image,
                });
            } else {
                // Outside asset root: create a placeholder handle now and
                // queue the real decode for later frames so we don't block
                // the event loop.
                let handle = images.add(Image::default());
                let absolute = path.to_string_lossy().replace('\\', "/");
                entries.push(TextureEntry {
                    path: absolute,
                    file_name,
                    image: handle.clone(),
                });
                pending.push((path, handle));
            }
        }
    }
}

fn is_image_file(path: &Path) -> bool {
    let Some(ext) = path.extension() else {
        return false;
    };
    let ext = ext.to_string_lossy().to_lowercase();
    matches!(
        ext.as_str(),
        "png" | "jpg" | "jpeg" | "bmp" | "tga" | "webp"
    )
}

fn decode_image_from_disk(path: &Path) -> Option<Image> {
    let bytes = std::fs::read(path).ok()?;
    let ext = path.extension()?.to_str()?;
    Image::from_buffer(
        &bytes,
        ImageType::Extension(ext),
        CompressedImageFormats::NONE,
        true,
        ImageSampler::default(),
        RenderAssetUsages::default(),
    )
    .ok()
}

fn load_pending_textures(
    mut pending: ResMut<PendingTextureLoads>,
    mut images: ResMut<Assets<Image>>,
) {
    if pending.0.is_empty() {
        return;
    }
    // Decode a small batch each frame so the event loop stays responsive.
    for _ in 0..2 {
        let Some((path, handle)) = pending.0.pop() else {
            return;
        };
        if let Some(loaded) = decode_image_from_disk(&path) {
            if let Some(img) = images.get_mut(handle.id()) {
                *img = loaded;
            }
        }
    }
}

fn rescan_textures(
    mut available: ResMut<AvailableTextures>,
    asset_server: Res<AssetServer>,
    mut images: ResMut<Assets<Image>>,
    mut pending: ResMut<PendingTextureLoads>,
    project_root: Option<Res<crate::project::ProjectRoot>>,
) {
    if !available.needs_rescan {
        return;
    }
    available.needs_rescan = false;
    let asset_root = project_root
        .map(|p| p.assets_dir())
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_default().join("assets"));
    do_scan_textures(
        &mut available,
        &asset_server,
        &mut images,
        &asset_root,
        &mut pending.0,
    );
}

fn handle_apply_texture(
    event: On<ApplyTextureToFaces>,
    brush_selection: Res<BrushSelection>,
    edit_mode: Res<EditMode>,
    mut brushes: Query<&mut Brush>,
    mut history: ResMut<CommandHistory>,
    mut last_texture: ResMut<LastUsedTexture>,
) {
    if *edit_mode != EditMode::BrushEdit(BrushEditMode::Face) {
        return;
    }
    let Some(brush_entity) = brush_selection.entity else {
        return;
    };
    if brush_selection.faces.is_empty() {
        return;
    }
    let Ok(mut brush) = brushes.get_mut(brush_entity) else {
        return;
    };

    let old = brush.clone();
    for &face_idx in &brush_selection.faces {
        if face_idx < brush.faces.len() {
            brush.faces[face_idx].texture_path = Some(event.path.clone());
        }
    }

    // Remember the last-used texture for new brushes
    last_texture.texture_path = Some(event.path.clone());

    let cmd = SetBrush {
        entity: brush_entity,
        old,
        new: brush.clone(),
        label: "Apply texture".to_string(),
    };
    history.undo_stack.push(Box::new(cmd));
    history.redo_stack.clear();
}

/// Marker for the texture browser panel.
#[derive(Component)]
pub struct TextureBrowserPanel;

/// Marker for the texture browser grid content area.
#[derive(Component)]
pub struct TextureBrowserGrid;

/// Marker for the texture browser filter input.
#[derive(Component)]
pub struct TextureBrowserFilter;

/// Marker for each texture thumbnail.
#[derive(Component)]
pub struct TextureThumbnail {
    pub path: String,
}

/// Marker for the root directory label text in the texture browser.
#[derive(Component)]
struct TextureBrowserRootLabel;

/// Resource holding the async folder picker task for the texture browser.
#[derive(Resource)]
struct TextureBrowserFolderTask(Task<Option<rfd::FileHandle>>);

/// External images that still need to be decoded (one or two per frame to avoid stalling).
#[derive(Resource, Default)]
struct PendingTextureLoads(Vec<(PathBuf, Handle<Image>)>);

/// Update the filter string from the text input.
fn apply_texture_filter(
    filter_input: Query<&TextEditValue, (With<TextureBrowserFilter>, Changed<TextEditValue>)>,
    mut available: ResMut<AvailableTextures>,
) {
    for input in &filter_input {
        if available.filter != input.0 {
            available.filter = input.0.clone();
        }
    }
}

fn spawn_texture_folder_dialog(
    _: On<Pointer<Click>>,
    mut commands: Commands,
    raw_handle: Query<&RawHandleWrapper, With<PrimaryWindow>>,
) {
    let mut dialog = AsyncFileDialog::new().set_title("Select textures directory");
    if let Ok(rh) = raw_handle.single() {
        let handle = unsafe { rh.get_handle() };
        dialog = dialog.set_parent(&handle);
    }
    let task = AsyncComputeTaskPool::get().spawn(async move { dialog.pick_folder().await });
    commands.insert_resource(TextureBrowserFolderTask(task));
}

fn poll_texture_browser_folder(world: &mut World) {
    let Some(mut task_res) = world.get_resource_mut::<TextureBrowserFolderTask>() else {
        return;
    };
    let Some(result) = future::block_on(future::poll_once(&mut task_res.0)) else {
        return;
    };
    world.remove_resource::<TextureBrowserFolderTask>();

    if let Some(handle) = result {
        let path = handle.path().to_path_buf();
        let mut available = world.resource_mut::<AvailableTextures>();
        available.scan_directory = path.clone();
        available.needs_rescan = true;

        let mut label_query = world.query_filtered::<&mut Text, With<TextureBrowserRootLabel>>();
        for mut text in label_query.iter_mut(world) {
            **text = path.to_string_lossy().to_string();
        }
    }
}

fn update_texture_browser_ui(
    mut commands: Commands,
    available: Res<AvailableTextures>,
    grid_query: Query<(Entity, Option<&Children>), With<TextureBrowserGrid>>,
    mut root_label_query: Query<&mut Text, With<TextureBrowserRootLabel>>,
) {
    if !available.is_changed() {
        return;
    }

    // Update the root label text
    for mut text in root_label_query.iter_mut() {
        **text = available.scan_directory.to_string_lossy().to_string();
    }

    let Ok((grid_entity, grid_children)) = grid_query.single() else {
        return;
    };

    // Clear existing thumbnails
    if let Some(children) = grid_children {
        for child in children.iter() {
            commands.entity(child).despawn();
        }
    }

    let filter_lower = available.filter.to_lowercase();

    for entry in &available.textures {
        // Apply filter
        if !filter_lower.is_empty()
            && !entry.file_name.to_lowercase().contains(&filter_lower)
            && !entry.path.to_lowercase().contains(&filter_lower)
        {
            continue;
        }

        let path = entry.path.clone();
        let image = entry.image.clone();

        // Thumbnail container
        let thumb_entity = commands
            .spawn((
                TextureThumbnail { path: path.clone() },
                Node {
                    width: Val::Px(64.0),
                    height: Val::Px(80.0),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    padding: UiRect::all(Val::Px(2.0)),
                    border: UiRect::all(Val::Px(1.0)),
                    border_radius: BorderRadius::all(Val::Px(4.0)),
                    ..Default::default()
                },
                BorderColor::all(Color::NONE),
                BackgroundColor(Color::NONE),
                ChildOf(grid_entity),
            ))
            .id();

        // Image thumbnail
        commands.spawn((
            ImageNode::new(image),
            Node {
                width: Val::Px(56.0),
                height: Val::Px(56.0),
                ..Default::default()
            },
            ChildOf(thumb_entity),
        ));

        // File name label
        let display_name = if entry.file_name.len() > 10 {
            format!("{}...", &entry.file_name[..8])
        } else {
            entry.file_name.clone()
        };
        commands.spawn((
            Text::new(display_name),
            TextFont {
                font_size: 9.0,
                ..Default::default()
            },
            TextColor(tokens::TEXT_SECONDARY),
            Node {
                max_width: Val::Px(60.0),
                overflow: Overflow::clip(),
                ..Default::default()
            },
            ChildOf(thumb_entity),
        ));

        // Hover + click
        let path_for_click = path.clone();
        commands.entity(thumb_entity).observe(
            |hover: On<Pointer<Over>>, mut borders: Query<&mut BorderColor>| {
                if let Ok(mut border) = borders.get_mut(hover.event_target()) {
                    *border = BorderColor::all(tokens::SELECTED_BORDER);
                }
            },
        );
        commands.entity(thumb_entity).observe(
            |out: On<Pointer<Out>>, mut borders: Query<&mut BorderColor>| {
                if let Ok(mut border) = borders.get_mut(out.event_target()) {
                    *border = BorderColor::all(Color::NONE);
                }
            },
        );
        commands.entity(thumb_entity).observe(
            move |_: On<Pointer<Click>>, mut commands: Commands| {
                commands.trigger(ApplyTextureToFaces {
                    path: path_for_click.clone(),
                });
            },
        );
    }
}

pub fn texture_browser_panel(icon_font: Handle<Font>) -> impl Bundle {
    (
        TextureBrowserPanel,
        EditorEntity,
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            ..Default::default()
        },
        BackgroundColor(tokens::PANEL_BG),
        children![
            // Header with directory path and folder picker
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
                                Text::new("Textures"),
                                TextFont {
                                    font_size: tokens::FONT_MD,
                                    ..Default::default()
                                },
                                ThemedText,
                            ),
                            (
                                TextureBrowserRootLabel,
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
                    texture_folder_button(icon_font),
                ],
            ),
            // Filter input
            (
                Node {
                    padding: UiRect::axes(Val::Px(tokens::SPACING_SM), Val::Px(tokens::SPACING_XS)),
                    flex_shrink: 0.0,
                    ..Default::default()
                },
                children![(
                    TextureBrowserFilter,
                    text_edit::text_edit(
                        TextEditProps::default()
                            .with_placeholder("Filter textures")
                            .allow_empty()
                    )
                ),],
            ),
            // Grid
            (
                TextureBrowserGrid,
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
                    row_gap: Val::Px(tokens::SPACING_XS),
                    column_gap: Val::Px(tokens::SPACING_XS),
                    ..Default::default()
                },
            ),
        ],
    )
}

fn texture_folder_button(icon_font: Handle<Font>) -> impl Bundle {
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
        observe(spawn_texture_folder_dialog),
    )
}

/// Convert an absolute filesystem path to an asset-relative path.
/// Tries the project root first, then falls back to CWD.
pub fn to_asset_relative_path(absolute: &str) -> Option<String> {
    let abs_path = Path::new(absolute);

    // Try ProjectRoot via recent projects config
    if let Some(project_dir) = crate::project::read_last_project() {
        let assets_dir = project_dir.join("assets");
        if let Ok(relative) = abs_path.strip_prefix(&assets_dir) {
            return Some(relative.to_string_lossy().replace('\\', "/"));
        }
    }

    // Fallback to CWD
    let assets_dir = std::env::current_dir().ok()?.join("assets");
    let relative = abs_path
        .strip_prefix(&assets_dir)
        .ok()?
        .to_string_lossy()
        .replace('\\', "/");
    Some(relative)
}
