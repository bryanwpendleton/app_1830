use bevy::prelude::*;
use hexx::Hex;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, (setup_camera, setup_hex_settings, spawn_hex_tiles).chain())
        .add_systems(Update, handle_tile_clicks)
        .run();
}

#[derive(Resource)]
struct HexSettings {
    hex_size: f32,
    center_hex: Hex, // The hex coordinate to center on
}

impl HexSettings {
    // Convert hex axial coordinates to world position (pointy-top orientation)
    fn hex_to_world_pos(&self, hex: Hex) -> Vec2 {
        let size = self.hex_size;
        // Offset by center_hex to center the map
        let offset_hex = hex - self.center_hex;
        let x = size * (3.0 / 2.0 * offset_hex.x as f32);
        let y = size * (3.0_f32.sqrt() / 2.0 * offset_hex.x as f32 + 3.0_f32.sqrt() * offset_hex.y as f32);
        info!("WorldPos of {:?} is {},{}", hex, x, y);
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

fn setup_hex_settings(
    mut commands: Commands,
    primary_window: Query<&Window>,
) {
    let window = match primary_window.iter().next() {
        Some(w) => w,
        None => {
            // Use default if window not available yet
            commands.insert_resource(HexSettings {
                hex_size: 30.0,
                center_hex: Hex::new(12, 6),
            });
            return;
        }
    };

    let window_width = window.resolution.width();
    let window_height = window.resolution.height();

    // Map dimensions: 24 columns (0-23) by 11 rows (0-10)
    let map_cols = 24.0;
    let map_rows = 11.0;

    // For pointy-top hexagons:
    // Width needed: (map_cols - 1) * 3/2 * size + 2 * size (for first and last hex)
    // Height needed: map_rows * sqrt(3) * size + size (for top and bottom)

    // Calculate hex size based on window dimensions with some padding
    let padding = 0.9; // 90% of window size to leave some margin

    // Width constraint: (map_cols - 1) * 1.5 * size + 2 * size = window_width * padding
    // Simplify: ((map_cols - 1) * 1.5 + 2) * size = window_width * padding
    let size_from_width = (window_width * padding) / ((map_cols - 1.0) * 1.5 + 2.0);

    // Height constraint: (map_rows * sqrt(3) + 1) * size = window_height * padding
    let size_from_height = (window_height * padding) / (map_rows * 3.0_f32.sqrt() + 1.0);

    // Use the smaller size to ensure the map fits in both dimensions
    let hex_size = size_from_width.min(size_from_height);

    info!("size_from_width x size_from_height {}x{} results in hex_size min {}",
            size_from_width, size_from_height, hex_size);
    info!("Window size: {}x{}, Calculated hex_size: {}", window_width, window_height, hex_size);

    commands.insert_resource(HexSettings {
        hex_size,
        center_hex: Hex::new(12, 6), // Center on column 12, row 6
    });
}

fn spawn_hex_tiles(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    settings: Res<HexSettings>,
) {
    // Parse tile names from assets/Map directory
    // For now, spawn a few example tiles based on asset names
    // Row(r) (A..K/1..11) is the Y axis. Col(q) (1..24) is the X axis.
    let tiles = vec![
        ("A19", Hex::new(19,  1)),
        ("C19", Hex::new(19,  3)),
        ("E5",  Hex::new( 5,  5)),
        ("J14", Hex::new(14, 10)),
        ("D10", Hex::new(10,  4)),
        ("K15", Hex::new(15, 11)),
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
