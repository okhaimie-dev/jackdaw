use std::path::PathBuf;

use bevy::prelude::*;
use jackdaw::AppState;
use jackdaw_jsn::format::{JsnHeader, JsnProject, JsnProjectConfig};

mod util;

/// Set up a minimal `ProjectRoot` so that systems requiring `Res<ProjectRoot>`
/// (e.g. `update_material_browser_ui`) do not panic in headless tests.
fn setup_test_project(app: &mut App) {
    let root = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    app.world_mut().insert_resource(jackdaw::project::ProjectRoot {
        root: root.clone(),
        config: JsnProject {
            jsn: JsnHeader::default(),
            project: JsnProjectConfig {
                name: root
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| "test".to_string()),
                description: String::new(),
                default_scene: None,
                layout: None,
            },
        },
    });
}

/// Transition the app into the Editor state so that `OnEnter(AppState::Editor)`
/// systems (including `scan_material_definitions`) have a chance to run.
fn enter_editor_state(app: &mut App) {
    app.world_mut()
        .resource_mut::<NextState<AppState>>()
        .set(AppState::Editor);
    // One tick to process the state transition and run OnEnter systems.
    app.update();
}

/// Verifies that the material browser always exposes a "None" entry
/// (issue #154) after the editor state is entered.
#[test]
fn material_registry_has_none_entry_after_startup() {
    let mut app = util::editor_test_app();
    setup_test_project(&mut app);
    enter_editor_state(&mut app);

    let registry = app.world().resource::<jackdaw::material_browser::MaterialRegistry>();

    assert!(
        !registry.entries.is_empty(),
        "MaterialRegistry should not be empty after startup"
    );

    let first = &registry.entries[0];
    assert_eq!(
        first.name, "None",
        "first registry entry should be 'None'"
    );
    assert_eq!(
        first.handle,
        Handle::default(),
        "None entry should carry the default (empty) handle"
    );
}

#[test]
fn material_registry_none_entry_survives_rescan() {
    let mut app = util::editor_test_app();
    setup_test_project(&mut app);
    enter_editor_state(&mut app);

    // Trigger a material rescan by flipping the flag that
    // `rescan_material_definitions` watches in Update.
    app.world_mut()
        .resource_mut::<jackdaw::material_browser::MaterialBrowserState>()
        .needs_rescan = true;

    // Tick once so the rescan system runs.
    app.update();

    let registry = app.world().resource::<jackdaw::material_browser::MaterialRegistry>();

    assert!(
        !registry.entries.is_empty(),
        "MaterialRegistry should not be empty after rescan"
    );

    let first = &registry.entries[0];
    assert_eq!(
        first.name, "None",
        "first registry entry should still be 'None' after rescan"
    );
    assert_eq!(
        first.handle,
        Handle::default(),
        "None entry should still carry the default handle after rescan"
    );
}

#[test]
fn none_entry_is_not_persisted_to_asset_catalog() {
    let mut app = util::editor_test_app();
    setup_test_project(&mut app);
    enter_editor_state(&mut app);

    let catalog = app.world().resource::<jackdaw::asset_catalog::AssetCatalog>();

    assert!(
        !catalog.contains_name("@None"),
        "The '@None' pseudo-entry must not leak into the AssetCatalog"
    );
}
