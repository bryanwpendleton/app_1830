use bevy::prelude::*;
use bevy_egui::EguiContexts;
use hexx::Hex;
use hexx::HexLayout;

/*
    The routemap supports the Operating Round(s) of 1830.

    Specifically, the routemap is used by the railroad president
    to perform the following aspects of the railroad's operating
    turn:

    - Construct track (either new or upgrade track)
    - Place a station
    - Run trains, specifically compute the best route(s) for
      this railroad's train(s), and announce the resulting revenue.

    The routemap itself is a primary resource of the game, and
    evolves during these operating rounds. Since comprehending
    the routemap is crucial during other phases of play, the
    routemap is always visible on the screen.
 */

#[derive(Component)]
pub struct MapTile {
    pub coord: Hex,
    pub tile_name: String,
}

/*
 * Components related to TrackTile:
 * - tile_number
 * - TrackColor
 * - TrackInventoryQuantity
 * - TrackRotation (Also referred to as "tile facings")
 */

#[derive(Component)]
struct TrackTile {
    tile_number: u32,
}

#[derive(Component)]
enum TrackColor {
    Yellow,
    Green,
    Orange,
}

#[derive(Component)]
struct TrackRotation {
    rotation: u32,      // facing 3 means rotated twice.
}

#[derive(Resource)]
pub struct HexSettings {
    pub layout: HexLayout,
}

impl HexSettings {
    // Convert hex axial coordinates to world position (pointy-top orientation)
    pub fn hex_to_world_pos(&self, hex: Hex) -> hexx::Vec2 {
        let wp = self.layout.hex_to_world_pos(hex);
        info!("WorldPos of {:?} is {},{}", hex, wp.x, wp.y);
        wp
    }

    // Convert world position to hex axial coordinates (pointy-top orientation)
    pub fn world_pos_to_hex(&self, pos: hexx::Vec2) -> Hex {
        let hex = self.layout.world_pos_to_hex(pos);

info!("location of worldPos {},{} is {:?}", pos.x,pos.y,hex);
        hex
    }
}

pub fn setup_routemap(
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

pub fn spawn_routemap(
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

        ("blank_1large", Hex::new( 1, -4)), // B10
        ("blank", Hex::new( 2, -4)), // B12
        ("blank", Hex::new( 3, -4)), // B14
        ("blank_1large", Hex::new( 4, -4)), // B16
        ("B18", Hex::new( 5, -4)),
        ("B20", Hex::new( 6, -4)),
        ("blank", Hex::new( 7, -4)), // B22
        ("B24", Hex::new( 8, -4)),

        ("blank", Hex::new(-1, -3)), // C7
        ("blank", Hex::new( 0, -3)), // C9
        ("blank", Hex::new( 1, -3)), // C11
        ("blank", Hex::new( 2, -3)), // C13
        ("C15", Hex::new( 3, -3)),
        ("blank_mountain", Hex::new( 4, -3)), // C17
        ("C19", Hex::new( 5, -3)),
        ("blank_mountain", Hex::new( 6, -3)), // C21
        ("blank", Hex::new( 7, -3)), // C23

        ( "D2", Hex::new(-4, -2)),
        ("blank_1small", Hex::new(-3, -2)), // D4
        ( "D6", Hex::new(-2, -2)),
        ("blank", Hex::new(-1, -2)), // D8
        ("D10", Hex::new( 0, -2)),
        ("blank", Hex::new( 1, -2)), // D12
        ("D14", Hex::new( 2, -2)),
        ("blank", Hex::new( 3, -2)), // D16
        ("D18", Hex::new( 4, -2)),
        ("blank", Hex::new( 5, -2)), // D20
        ("blank_mountain", Hex::new( 6, -2)), // D22
        ("D24", Hex::new( 7, -2)),

        ("blank", Hex::new(-4, -1)), // E3
        ("E5",  Hex::new(-3, -1)),
        ("blank_1small", Hex::new(-2, -1)), // E7
        ("E9",  Hex::new(-1, -1)),
        ("E11",  Hex::new( 0, -1)),
        ("blank", Hex::new( 1, -1)), // E13
        ("blank", Hex::new( 2, -1)), // E15
        ("blank_mountain", Hex::new( 3, -1)), // E17
        ("E19",  Hex::new( 4, -1)),
        ("blank_mountain", Hex::new( 5, -1)), // E21
        ("E23",  Hex::new( 6, -1)),

        ( "F2",  Hex::new(-5,  0)),
        ( "F4",  Hex::new(-4,  0)),
        ( "F6",  Hex::new(-3,  0)),
        ("blank", Hex::new(-2,  0)), // F8
        ("blank_1small", Hex::new(-1,  0)), // F10
        ("blank", Hex::new( 0,  0)), // F12 -- CENTER OF THE SCREEN
        ("blank", Hex::new( 1,  0)), // F14
        ("F16",  Hex::new( 2,  0)),
        ("blank", Hex::new( 3,  0)), // F18
        ("blank_2small", Hex::new( 4,  0)), // F20
        ("F22",  Hex::new( 5,  0)),
        ("F24",  Hex::new( 6,  0)),

        ("blank", Hex::new(-5,  1)), // G3
        ("blank", Hex::new(-4,  1)), // G5
        ("blank_2small", Hex::new(-3,  1)), // G7
        ("blank", Hex::new(-2,  1)), // G9
        ("blank", Hex::new(-1,  1)), // G11
        ("blank_mountain", Hex::new( 0,  1)), // G13
        ("G15", Hex::new( 1,  1)),
        ("blank_2small", Hex::new( 2,  1)), // G17
        ("G19", Hex::new( 3,  1)),

        ("blank", Hex::new(-6,  2)), // H2
        ("blank_1large", Hex::new(-5,  2)), // H4
        ("blank", Hex::new(-4,  2)), // H6
        ("blank", Hex::new(-3,  2)), // H8
        ("blank_1large", Hex::new(-2,  2)), // H10
        ("H12", Hex::new(-1,  2)),
        ("blank", Hex::new( 0,  2)), // H14
        ("blank_1large", Hex::new( 1,  2)), // H10
        ("H18", Hex::new( 2,  2)),

        ("blank", Hex::new(-6,  3)), // I3
        ("blank", Hex::new(-5,  3)), // I5
        ("blank", Hex::new(-4,  3)), // I7
        ("blank", Hex::new(-3,  3)), // I9
        ("blank_mountain", Hex::new(-2,  3)), // I11
        ("blank", Hex::new(-1,  3)), // I13
        ("I15", Hex::new( 0,  3)),
        ("I17", Hex::new( 1,  3)),
        ("I19", Hex::new( 2,  3)),

        ( "J2", Hex::new(-7,  4)),
        ("blank", Hex::new(-6,  4)), // J4
        ("blank", Hex::new(-5,  4)), // J6
        ("blank", Hex::new(-4,  4)), // J8
        ("blank_mountain", Hex::new(-3,  4)), // J10
        ("blank_mountain", Hex::new(-2,  4)), // J12
        ("J14", Hex::new(-1,  4)),

        ("K13", Hex::new(-2,  5)),
        ("K15", Hex::new(-1,  5)),
    ];

    for (tile_name, coord) in tiles {
        let world_pos = settings.hex_to_world_pos(coord);

        commands.spawn((
            Sprite::from_image(asset_server.load(format!("Map/{}.png", tile_name))),
            Transform::from_xyz(world_pos.x, world_pos.y, 0.0),
            MapTile {
                coord,
                tile_name: tile_name.to_string(),
            },
        ));
    }
}

pub fn handle_tile_clicks(
    mut contexts: EguiContexts,
    buttons: Res<ButtonInput<MouseButton>>,
    camera_query: Query<(&Camera, &GlobalTransform), With<super::MainCamera>>,
    windows: Query<&Window>,
    tile_query: Query<(&MapTile, &Transform)>,
    settings: Res<HexSettings>,
) {
    // Don't treat clicks on the egui panel as map clicks.
    if let Ok(ctx) = contexts.ctx_mut() {
        if ctx.egui_wants_pointer_input() {
            return;
        }
    }

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

/*
    Here are the 1830 TrackTile entities.
 */
