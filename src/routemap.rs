
use std::collections::HashMap;

use bevy::prelude::*;
use bevy_egui::EguiContexts;
use hexx::Hex;
use hexx::HexLayout;
use hexx::algorithms::a_star;

use crate::stockmarket::GridBox;
use crate::gamemodel::GameState;

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

#[derive(Component,Eq, Hash, PartialEq, Clone)]
pub struct HexName {
    pub name: String,
}

#[derive(Resource)]
pub struct MapTile {
    pub coord: Hex,
    pub hex_name: HexName,
    pub tile_name: String,
    pub connectivity: HashMap<String, u32>,
    pub market: HashMap<String, GridBox>,
}

impl MapTile {
/*
    pub fn route_cost(start: &MapTile, end: &MapTile) -> Option<u32> {
        if let Some(cost) = start.connectivity.get(&end.hex_name.name)
        {
            cost.unwrap()
        }
        
        Some(0 as u32)
    }
*/
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
    mut game_state: ResMut<GameState>,
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
        let mut map_tile = MapTile {
                coord,
                hex_name,
                connectivity: HashMap::new(),
                market: HashMap::new(),
                tile_name: tile_name.to_string(),
            };
        commands.insert_resource( map_tile );

        commands.spawn((
            Sprite::from_image(asset_server.load(format!("Map/{}.png", tile_name))),
            Transform::from_xyz(world_pos.x, world_pos.y, 0.0),
            map_tile,
        ));
        // game_state.route_map.insert(hex_name.name, map_tile);
        // game_state.route_tiles.insert(coord, map_tile);
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
    Route finding overview.
   
    A route must connect two cities. There are 48 (check this!) cities
    in the 1830 map. So there are potentially a lot of routes, but
    most of these routes do not exist because track must be laid first.
   
    We can use the a_star algorithm to compute the routes.
   
    For each hex in the map (93? check this), we need to keep track
    of whether or it is connected to each of its 6 neighbors. This
    enables us to have a cost function for the a_start algorithm that
    returns either "yes" or "no" when the algorithm asks us if a
    pair of hexes are connected.
   
    At the start of the game, we set up the connectivity table for
    each hex. Nearly all entries are "no connection" at this point.
   
    Each time a track tile is placed, we update the connectivity
    table for that hex and for each of its 6 neighbors. Over time
    the track starts to form connections and routes emerge. Note
    that this computation is also where we ensure legal tile upgrades
    as they must maintain existing connectivity.
   
    Some of the connectivity is dynamic, for example a station marker
    may restrict a route to only the railroad that placed that marker.
   
    Some route restrictions may only be evaluated after the
    potential route has been computed, for example the route must
    contain at least one of the railroad's own station markers.
   
    Some route restrictions may only be evaluated after a complete
    set of candidate routes has been computed, e.g. when a
    railroad has two or more trains each runing a route,
    none of those routes may use the same track.
   
    Due to applying restrictions, the route computation system needs
    needs to:
    - compute all the potential results, discarding those that meet
      any single-route restrictions.
    - considering the multi-route restrictions, form route sets
      that pass those restrictions by discarding one or more routes
      until the restrictions are met.
   
    At that point the routes can be combined with the railroad's
    available trains to compute the highest-revenue set, and that
    is used to pay dividends or add to the corporate treasury.
 */

/// System to perform some simple routefinding tests on the empty
/// map present at startup. Since this leaves the game map modified,
/// it will be removed once the routefinding module is stabilized.
pub fn do_simple_routefinding_tests(
    mut game_state: ResMut<GameState>,
) {         
    // On an empty map, there should be no path from A9 to B10.

/*
    let start = Hex::new(-1, -5);
    let end = Hex::new(1, -4);

    let path = a_star(start, end, |a, b| {
        MapTile::route_cost(game_state.route_tiles.get(&a),
                   game_state.route_tiles.get(&b))
    });

    info!("Empty map path from A9 to B10 is: {:?}", path);
*/
}
