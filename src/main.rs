use bevy::prelude::*;
use bevy_egui::EguiPlugin;

mod gamemodel;
mod routemap;

use gamemodel::Game1830Plugin;
use routemap::{setup_routemap, spawn_routemap, handle_tile_clicks};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Fixed Window".to_string(),
                resolution: ( (100+1680) , (100+770) ).into(),
                resizable: false, // Prevents the user from resizing
                ..default()
            }),
            ..default()
        }))
        .add_plugins(EguiPlugin::default())
        .add_plugins(Game1830Plugin)
        .add_systems(Startup, (setup_camera, setup_routemap, spawn_routemap).chain())
        .add_systems(Update, handle_tile_clicks)
        .run();
}

#[derive(Component)]
pub struct MainCamera;

fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        MainCamera,
    ));
}

