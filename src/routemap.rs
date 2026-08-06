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
pub struct HexName {
    pub name: String,
}

#[derive(Component)]
pub struct MapTile {
    pub coord: Hex,
    pub hex_name: HexName,
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

impl HexName
{
    pub fn new(nm: &str) -> Self
    {
        Self
        {
            name: String::from(nm),
        }
    }
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
    // This is the map.
    //
    // You can think of it as a hexagonal map in various coordinates
    // You can think of it as a vector of hexes.
    // You can think of it as rows and columns of hexes
    //
    // The map is "splotchy" around the edges, so not every
    // row and column in [A-K,1-24] exists.
    //
    // Here we set up the raw data
    // Parse tile names from assets/Map directory
    // Row(r) (A..K/1..11) is the Y axis. Col(q) (1..24) is the X axis.
    //
    // We're using F12 as the logical center of the screen,
    // everything from there.

    let hexes = vec![
        (HexName::new("A9"),  Hex::new( 1, -5),  "A9"),
        (HexName::new("A11"), Hex::new( 2, -5), "A11"),
        (HexName::new("A17"), Hex::new( 5, -5), "A17"),
        (HexName::new("A19"), Hex::new( 6, -5), "A19"),

        (HexName::new("B10"), Hex::new( 1, -4), "blank_1large"),
        (HexName::new("B12"), Hex::new( 2, -4), "blank"),
        (HexName::new("B14"), Hex::new( 3, -4), "blank"),
        (HexName::new("B16"), Hex::new( 4, -4), "blank_1large"),
        (HexName::new("B18"), Hex::new( 5, -4), "B18"),
        (HexName::new("B20"), Hex::new( 6, -4), "B20"),
        (HexName::new("B22"), Hex::new( 7, -4), "blank"),
        (HexName::new("B24"), Hex::new( 8, -4), "B24"),

        (HexName::new("C7"),  Hex::new(-1, -3), "blank"),
        (HexName::new("C9"),  Hex::new( 0, -3), "blank"),
        (HexName::new("C11"), Hex::new( 1, -3), "blank"),
        (HexName::new("C13"), Hex::new( 2, -3), "blank"),
        (HexName::new("C15"), Hex::new( 3, -3), "C15"),
        (HexName::new("C17"), Hex::new( 4, -3), "blank_mountain"),
        (HexName::new("C19"), Hex::new( 5, -3), "C19"),
        (HexName::new("C21"), Hex::new( 6, -3), "blank_mountain"),
        (HexName::new("C23"), Hex::new( 7, -3), "blank"),

        (HexName::new("D2"),  Hex::new(-4, -2), "D2"),
        (HexName::new("D4"),  Hex::new(-3, -2), "blank_1small"),
        (HexName::new("D6"),  Hex::new(-2, -2), "D6"),
        (HexName::new("D8"),  Hex::new(-1, -2), "blank"),
        (HexName::new("D10"), Hex::new( 0, -2), "D10"),
        (HexName::new("D12"), Hex::new( 1, -2), "blank"),
        (HexName::new("D14"), Hex::new( 2, -2), "D14"),
        (HexName::new("D16"), Hex::new( 3, -2), "blank"),
        (HexName::new("D18"), Hex::new( 4, -2), "D18"),
        (HexName::new("D20"), Hex::new( 5, -2), "blank"),
        (HexName::new("D22"), Hex::new( 6, -2), "blank_mountain"),
        (HexName::new("D24"), Hex::new( 7, -2), "D24"),

        (HexName::new("E3"),  Hex::new(-4, -1), "blank"),
        (HexName::new("E5"),  Hex::new(-3, -1), "E5"),
        (HexName::new("E7"),  Hex::new(-2, -1), "blank_1small"),
        (HexName::new("E9"),  Hex::new(-1, -1), "E9"),
        (HexName::new("E11"), Hex::new( 0, -1), "E11"),
        (HexName::new("E13"), Hex::new( 1, -1), "blank"),
        (HexName::new("E15"), Hex::new( 2, -1), "blank"),
        (HexName::new("E17"), Hex::new( 3, -1), "blank_mountain"),
        (HexName::new("E19"), Hex::new( 4, -1), "E19"),
        (HexName::new("E21"), Hex::new( 5, -1), "blank_mountain"),
        (HexName::new("E23"), Hex::new( 6, -1), "E23"),

        (HexName::new("F2"),  Hex::new(-5,  0), "F2"),
        (HexName::new("F4"),  Hex::new(-4,  0), "F4"),
        (HexName::new("F6"),  Hex::new(-3,  0), "F6"),
        (HexName::new("F8"),  Hex::new(-2,  0), "blank"),
        (HexName::new("F10"), Hex::new(-1,  0), "blank_1small"),
                // F12 -- CENTER OF THE SCREEN
        (HexName::new("F12"), Hex::new( 0,  0), "blank"),
                // F12 -- CENTER OF THE SCREEN
        (HexName::new("F14"), Hex::new( 1,  0), "blank"),
        (HexName::new("F16"), Hex::new( 2,  0), "F16"),
        (HexName::new("F18"), Hex::new( 3,  0), "blank"),
        (HexName::new("F20"), Hex::new( 4,  0), "blank_2small"),
        (HexName::new("F22"), Hex::new( 5,  0), "F22"),
        (HexName::new("F24"), Hex::new( 6,  0), "F24"),

        (HexName::new("G3"),  Hex::new(-5,  1), "blank"),
        (HexName::new("G5"),  Hex::new(-4,  1), "blank"),
        (HexName::new("G7"),  Hex::new(-3,  1), "blank_2small"),
        (HexName::new("G9"),  Hex::new(-2,  1), "blank"), // G9
        (HexName::new("G11"), Hex::new(-1,  1), "blank"),
        (HexName::new("G13"), Hex::new( 0,  1), "blank_mountain"),
        (HexName::new("G15"), Hex::new( 1,  1), "G15"),
        (HexName::new("G17"), Hex::new( 2,  1), "blank_2small"),
        (HexName::new("G19"), Hex::new( 3,  1), "G19"),

        (HexName::new("H2"),  Hex::new(-6,  2), "blank"),
        (HexName::new("H4"),  Hex::new(-5,  2), "blank_1large"),
        (HexName::new("H6"),  Hex::new(-4,  2), "blank"),
        (HexName::new("H8"),  Hex::new(-3,  2), "blank"),
        (HexName::new("H10"), Hex::new(-2,  2), "blank_1large"),
        (HexName::new("H12"), Hex::new(-1,  2), "H12"),
        (HexName::new("H14"), Hex::new( 0,  2), "blank"),
        (HexName::new("H16"), Hex::new( 1,  2), "blank_1large"),
        (HexName::new("H18"), Hex::new( 2,  2), "H18"),

        (HexName::new("I1"),  Hex::new(-7,  3), "blank"),
        (HexName::new("I3"),  Hex::new(-6,  3), "blank"),
        (HexName::new("I5"),  Hex::new(-5,  3), "blank"),
        (HexName::new("I7"),  Hex::new(-4,  3), "blank"),
        (HexName::new("I9"),  Hex::new(-3,  3), "blank"),
        (HexName::new("I11"), Hex::new(-2,  3), "blank_mountain"),
        (HexName::new("I13"), Hex::new(-1,  3), "blank"),
        (HexName::new("I15"), Hex::new( 0,  3), "I15"),
        (HexName::new("I17"), Hex::new( 1,  3), "I17"),
        (HexName::new("I19"), Hex::new( 2,  3), "I19"),

        (HexName::new("J2"),  Hex::new(-7,  4), "J2"),
        (HexName::new("J4"),  Hex::new(-6,  4), "blank"),
        (HexName::new("J6"),  Hex::new(-5,  4), "blank"),
        (HexName::new("J8"),  Hex::new(-4,  4), "blank"),
        (HexName::new("J10"), Hex::new(-3,  4), "blank_mountain"),
        (HexName::new("J12"), Hex::new(-2,  4), "blank_mountain"),
        (HexName::new("J14"), Hex::new(-1,  4), "J14"),

        (HexName::new("K13"), Hex::new(-2,  5), "K13"),
        (HexName::new("K15"), Hex::new(-1,  5), "K15"),
    ];

    for (hex_name, coord, tile_name) in hexes
    {
        let world_pos = settings.hex_to_world_pos(coord);

        commands.spawn((
            Sprite::from_image(asset_server.load(format!("Map/{}.png", tile_name))),
            Transform::from_xyz(world_pos.x, world_pos.y, 0.0),
            MapTile {
                coord,
                hex_name,
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
