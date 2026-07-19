use bevy::prelude::*;
use hexx::Hex;
use hexx::HexLayout;

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
        .add_systems(Startup, (setup_camera, setup_hex_settings, spawn_hex_tiles).chain())
        .add_systems(Update, handle_tile_clicks)
        .run();
}

#[derive(Resource)]
struct HexSettings {
    layout: HexLayout,
}

impl HexSettings {
    // Convert hex axial coordinates to world position (pointy-top orientation)
    fn hex_to_world_pos(&self, hex: Hex) -> hexx::Vec2 {
        let wp = self.layout.hex_to_world_pos(hex);
        info!("WorldPos of {:?} is {},{}", hex, wp.x, wp.y);
        wp
    }

    // Convert world position to hex axial coordinates (pointy-top orientation)
    fn world_pos_to_hex(&self, pos: hexx::Vec2) -> Hex {
        let hex = self.layout.world_pos_to_hex(pos);

info!("location of worldPos {},{} is {:?}", pos.x,pos.y,hex);
        hex
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
    _primary_window: Query<&Window>,
) {
    // Map dimensions: 24 columns (0-23) by 11 rows (0-10)
    let _map_cols = 24.0;
    let _map_rows = 11.0;

    let mut hl = HexLayout::pointy().with_hex_size(35.0);
    hl.invert_y();

    commands.insert_resource(HexSettings {
        layout: hl,
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
    //
    // We're using F12 as the logical center of the screen,
    // everything from there.

    let tiles = vec![
        ( "A9", Hex::new( 1, -5)),
        ("A11", Hex::new( 2, -5)),
        ("A17", Hex::new( 5, -5)),
        ("A19", Hex::new( 6, -5)),

        ("B20", Hex::new( 6, -4)),
        ("B24", Hex::new( 8, -4)),

        ("blank", Hex::new(-1, -3)),
        ("blank", Hex::new( 0, -3)),
        ("blank", Hex::new( 1, -3)),
        ("blank", Hex::new( 2, -3)),
        ("C15", Hex::new( 3, -3)),
        ("C19", Hex::new( 5, -3)),

        ( "D2", Hex::new(-4, -2)),
        ("D10", Hex::new( 0, -2)),
        ("D14", Hex::new( 2, -2)),
        ("D18", Hex::new( 4, -2)),
        ("D24", Hex::new( 6, -2)),

        ("E5",  Hex::new(-3, -1)),
        ("E9",  Hex::new(-1, -1)),
        ("E11",  Hex::new( 0, -1)),
        ("E19",  Hex::new( 4, -1)),

        ( "F2",  Hex::new(-5,  0)),
        ( "F4",  Hex::new(-4,  0)),
        ( "F6",  Hex::new(-3,  0)),
        ("F16",  Hex::new( 2,  0)),
        ("F24",  Hex::new( 6,  0)),

        ("G15", Hex::new( 1,  1)),
        ("G19", Hex::new( 3,  1)),

        // ("H10", Hex::new( 0,  2)),
        ("H12", Hex::new(-1,  2)),
        ("H18", Hex::new( 2,  2)),

        ("I15", Hex::new( 0,  3)),
        ("I17", Hex::new( 1,  3)),
        ("I19", Hex::new( 2,  3)),

        ( "J2", Hex::new(-7,  4)),
        ("J14", Hex::new(-1,  4)),

        ("K13", Hex::new(-2,  5)),
        ("K15", Hex::new(-1,  5)),
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
            let hex_pos = hexx::Vec2::new(world_pos.x as f32, world_pos.y as f32);
            let hex_coord = settings.world_pos_to_hex(hex_pos);

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
