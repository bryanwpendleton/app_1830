use bevy::prelude::*;
use hexx::Hex;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(HexSettings {
            hex_size: 35.0, // Half of 70px tile size
        })
        .add_systems(Startup, (setup_camera, spawn_hex_tiles))
        .add_systems(Update, handle_tile_clicks)
        .run();
}

#[derive(Resource)]
struct HexSettings {
    hex_size: f32,
}

impl HexSettings {
    // Convert hex axial coordinates to world position (pointy-top orientation)
    fn hex_to_world_pos(&self, hex: Hex) -> Vec2 {
        let size = self.hex_size;
        let x = size * (3.0 / 2.0 * hex.x as f32);
        let y = size * (3.0_f32.sqrt() / 2.0 * hex.x as f32 + 3.0_f32.sqrt() * hex.y as f32);
info!("WorldPos of {:?} is {},{}", hex,x,y);
        Vec2::new(x, y)
    }

    // Convert world position to hex axial coordinates (pointy-top orientation)
    fn world_pos_to_hex(&self, pos: Vec2) -> Hex {
        let size = self.hex_size;
        let q = (2.0 / 3.0 * pos.x) / size;
        let r = (-1.0 / 3.0 * pos.x + 3.0_f32.sqrt() / 3.0 * pos.y) / size;

info!("location of worldPos {},{} is q,r {},{}", pos.x,pos.y,q,r);
        // Round to nearest hex using cube coordinates
        let x = q;
        let z = r;
        let y = -x - z;

        let mut rx = x.round();
        let mut ry = y.round();
        let mut rz = z.round();

        let x_diff = (rx - x).abs();
        let y_diff = (ry - y).abs();
        let z_diff = (rz - z).abs();

        if x_diff > y_diff && x_diff > z_diff {
            rx = -ry - rz;
        } else if y_diff > z_diff {
            ry = -rx - rz;
        } else {
            rz = -rx - ry;
        }
info!("hex rx,ry {},{} will be returned", rx, ry);
        let _ = ry; // Use ry to silence warning

        Hex::new(rx as i32, rz as i32)
    }
}

#[derive(Component)]
struct HexTile {
    coord: Hex,
    tile_name: String,
}

#[derive(Component)]
struct MainCamera;

fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        MainCamera,
    ));
}

fn spawn_hex_tiles(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    settings: Res<HexSettings>,
) {
    // Parse tile names from assets/Map directory
    // For now, spawn a few example tiles based on your asset names
    let tiles = vec![
        ("A19", Hex::new( 0, 19)),
        ("C19", Hex::new( 2, 19)),
        ("E5",  Hex::new( 4,  5)),
        ("J14", Hex::new(10, 14)),
        ("D10", Hex::new( 3, 10)),
        ("K15", Hex::new(11, 15)),
    ];

    for (tile_name, coord) in tiles {
        let world_pos = settings.hex_to_world_pos(coord);

        commands.spawn((
            Sprite::from_image(asset_server.load(format!("Map/{}.png", tile_name))),
            Transform::from_xyz(world_pos.x, world_pos.y, 0.0),
            HexTile {
                coord,
                tile_name: tile_name.to_string(),
            },
        ));
    }
}

fn handle_tile_clicks(
    buttons: Res<ButtonInput<MouseButton>>,
    camera_query: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    windows: Query<&Window>,
    tile_query: Query<(&HexTile, &Transform)>,
    settings: Res<HexSettings>,
) {
    if buttons.just_pressed(MouseButton::Left) {
        let Ok((camera, camera_transform)) = camera_query.single() else {
            return;
        };

        let Ok(window) = windows.single() else {
            return;
        };

        let Some(cursor_pos) = window.cursor_position() else {
            return;
        };

        // Convert cursor position to world coordinates
        if let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_pos) {
            // Convert world position to hex coordinate
            let hex_coord = settings.world_pos_to_hex(world_pos);

info!("Is there a tile at {:?}", hex_coord);
            // Check if there's a tile at this coordinate
            for (tile, _transform) in tile_query.iter() {
                if tile.coord == hex_coord {
                    info!("Clicked on tile: {} at {:?}", tile.tile_name, hex_coord);
                }
            }
        }
    }
}
