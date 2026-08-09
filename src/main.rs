use bevy::prelude::*;
use bevy_egui::EguiPlugin;

mod gamemodel;
mod routemap;
mod stockmarket;

use bevy::window::{Monitor, PrimaryMonitor};

use gamemodel::Game1830Plugin;
use routemap::{setup_routemap, spawn_routemap, handle_tile_clicks};
use stockmarket::initialize_stock_market;

// Window sizing.
//
// The map's hex centers span roughly 697x525 world units; adding a hex of
// padding on each edge gives the map area below. Both a left and a right egui
// panel column are reserved so tooling always has room. The minimum window
// size must fit the map between the two panels; the maximum is clamped at
// runtime to the primary monitor so the window always fits on screen.
const MAP_AREA_WIDTH: f32 = 760.0;
const MAP_AREA_HEIGHT: f32 = 600.0;
const PANEL_WIDTH: f32 = 220.0;
const MIN_WINDOW_WIDTH: f32 = MAP_AREA_WIDTH + 2.0 * PANEL_WIDTH; // 1200
const MIN_WINDOW_HEIGHT: f32 = MAP_AREA_HEIGHT + 40.0; // room for panel headers

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "1830".to_string(),
                resolution: (MIN_WINDOW_WIDTH as u32, MIN_WINDOW_HEIGHT as u32).into(),
                resizable: true,
                resize_constraints: WindowResizeConstraints {
                    min_width: MIN_WINDOW_WIDTH,
                    min_height: MIN_WINDOW_HEIGHT,
                    // Max is set from the primary monitor once it's available.
                    ..default()
                },
                ..default()
            }),
            ..default()
        }))
        .add_plugins(EguiPlugin::default())
        .add_plugins(Game1830Plugin)
        .add_systems(Startup, (setup_camera,
                    setup_routemap, spawn_routemap,
                    initialize_stock_market).chain())
        .add_systems(Update, (handle_tile_clicks, clamp_window_to_monitor))
        .run();
}

/// Clamps the primary window's maximum size to the primary monitor so the
/// window can never grow larger than the user's screen.
///
/// Monitor entities are only populated once the windowing backend is running,
/// so this runs each frame until it succeeds, then does nothing (the `done`
/// local latches after the constraint is applied).
fn clamp_window_to_monitor(
    mut done: Local<bool>,
    monitors: Query<&Monitor, With<PrimaryMonitor>>,
    mut windows: Query<&mut Window>,
) {
    if *done {
        return;
    }

    let Ok(monitor) = monitors.single() else {
        return;
    };
    let Ok(mut window) = windows.single_mut() else {
        return;
    };

    // Convert physical monitor pixels to the logical units the window uses.
    let scale = monitor.scale_factor as f32;
    let logical_width = monitor.physical_width as f32 / scale;
    let logical_height = monitor.physical_height as f32 / scale;

    // Never let the max drop below the min, and stay at least as large as the
    // minimum on very small displays.
    window.resize_constraints.max_width = logical_width.max(MIN_WINDOW_WIDTH);
    window.resize_constraints.max_height = logical_height.max(MIN_WINDOW_HEIGHT);

    *done = true;
}

#[derive(Component)]
pub struct MainCamera;

fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        MainCamera,
    ));
}

