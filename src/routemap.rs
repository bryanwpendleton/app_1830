
use std::collections::HashMap;

use bevy::prelude::*;
use bevy_egui::EguiContexts;
use hexx::Hex;
use hexx::HexLayout;
use hexx::algorithms::a_star;

use crate::stockmarket::GridBox;
use crate::gamemodel::GameState;
use crate::gamemodel::place_tile_impl;
use crate::gamemodel::Train;

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

/// MapTile describes a single hex of the route map.
///
/// Each MapTile component wraps a single hexx::hex instance with some
/// additional metadata such as which track tile (if any) has been
/// currently placed here and whether there is a station marker present.
///
#[derive(Component)]
pub struct MapTile {
    pub coord: Hex,
    pub hex_color: HexColor,

    // hex_name is a row/column name of the Cell, often used for
    // indexing into other data structures. tile_name is a sprite
    // name, used to load the proper visualization for the map
    // So, e.g., hex_name G5 has tile_name blank, while
    // hex_name J14 which has a custom visual as tile_name J14.

    pub hex_name: HexName,
    pub tile_name: String,
    /// The `tile_number` of the [`Tile`] currently laid on this hex, or
    /// `0` when no track tile is placed here (`0` is not a valid tile number).
    pub placed_tile: u32,
    pub market: HashMap<String, GridBox>,

    pub track: [usize;6],

    // For a tan/clear or yellow map hex, we show the valid first tiles
    // that could be placed there. See the first 8 entries on the
    // Tile Upgrade Chart at http://www.fwtwr.com/18xx/info/tuc-1830.asp
    pub tile_choices: Vec<u32>,
}

/// Each hex on the board has a color that indicates what type of
/// track tile (if any) may be placed on that hex:
/// -  Gray hexes contain track and cities at the beginning of the
///    game. You may not place tiles on gray hexes.
/// -  Red hexes contain track representing connections to cities
///    outside of the board. You may not place tiles on red hexes.
/// -  Yellow hexes each contain 1 or 2 cities, and some also contain
///    track. You may only place green tiles on yellow hexes
/// -  Tan hexes can contain cities and/or track. You may only
///    place yellow tiles on tan hexes. ("clear" on original map)
#[derive(Component)]
pub enum HexColor {
    Gray,
    Red,
    Yellow,
    Tan,
}

/// The seven yellow hexes on the map have a HexLabel. 
/// - OO : These 4 hexes are the "Map 2 cities" hexes D10, E5, E11, H18
/// - NY : This hex is New York City: G19
/// -  B : These two hexes are Boston and Baltimore: E23, I15
/// These hexes have special tile placement and upgrade rules, e.g.:
/// only green tiles may be placed on yellow hexes. If the hex has
/// a label (OO, B, or NY), the tile must have a matching label.
#[derive(Component)]
pub enum HexLabel {
    OO,
    NY,
    B,
}

pub enum Face {
    NE = 0,
    E = 1,
    SE = 2,
    SW = 3,
    W = 4,
    NW = 5,
}

impl MapTile {
    /// Cost to traverse from this tile to `dest`.
    ///
    /// - self and dest share a face; we can figure out which face by
    ///   comparing their axial coordinates
    /// - if both self and dest have track to that face, we have
    ///   connectivity and return Some(0).
    /// - otherwise you can't get there from here and we return None.
    ///
    pub fn route_cost(&self, dest: &MapTile) -> Option<u32> {

        match self.faces(dest) {
            Some((self_face, dest_face)) => {
                if self.track[self_face as usize] == 1 &&
                                    dest.track[dest_face as usize] == 1 {
                    return Some(0);
                }
                return None;
            }
            _ => return None
        }
    }

    pub fn faces(&self, dest: &MapTile) -> Option<(i32, i32)>
    {
        let cdiff = self.coordinate_diff(dest);

        match cdiff {
            (1,-1) => { return Some((Face::NE as i32, Face::SW as i32)) }
            (1,0)  => { return Some((Face::E as i32,  Face::W as i32)) }
            (0,1)  => { return Some((Face::SE as i32, Face::NW as i32)) }
            (-1,1) => { return Some((Face::SW as i32, Face::NE as i32)) }
            (-1,0) => { return Some((Face::W as i32,  Face::E as i32)) }
            (0,-1) => { return Some((Face::NW as i32, Face::SE as i32)) }
            _ => { return None }
        }
    }

    pub fn coordinate_diff(&self, dest: &MapTile) -> (i32, i32)
    {
        (dest.coord.x - self.coord.x,
         dest.coord.y - self.coord.y)
    }
}

/*
 * Components related to Tile:
 * - tile_number
 * - TileColor
 * - TileInventoryQuantity
 * - TileRotation (Also referred to as "tile facings")
 */

#[derive(Component)]
pub struct Tile {
    pub tile_number: u32,
}

/// For a particular tile, encodes whether each face has track to it.
/// Encode as a 6-element array, where value 1 means yes there is track
/// on that face, and 0 means no there is no track on that face.
/// The faces are number clockwise from the NorthEast face (see also)
/// the enum Face.
#[derive(Component)]
pub struct TileTrack {
    pub track: [usize;6],
}

#[derive(Component)]
pub enum TileColor {
    Yellow,
    Green,
    Orange,
}
impl TileColor
{
    pub fn name(&self) -> &str
    {
        match self
        {
            TileColor::Yellow => "yellow",
            TileColor::Green  => "green",
            TileColor::Orange => "orange",
        }
    }
}

// How may different ways may this tile face? Equivalently, how many
// possible rotations are there (one less than the number of facings)?

pub enum TileFacings {
    One = 1,
    Two = 2,
    Three = 3,
    Six = 6,
}
impl TileFacings
{
    pub fn image_name(&self, rotation: usize) -> &str
    {
        match self
        {
            TileFacings::One => "base",
            TileFacings::Two => {
                match rotation
                {
                    0 => "base",
                    1 => "f2",
                    2 => "base",
                    3 => "f2",
                    4 => "base",
                    5 => "f2",
                    _ => "bad rotation",
                }
            },
            TileFacings::Three => {
                match rotation
                {
                    0 => "base",
                    1 => "f2",
                    2 => "f3",
                    3 => "base",
                    4 => "f2",
                    5 => "f3",
                    _ => "bad rotation",
                }
            },
            TileFacings::Six => {
                match rotation
                {
                    0 => "base",
                    1 => "f2",
                    2 => "f3",
                    3 => "f4",
                    4 => "f5",
                    5 => "f6",
                    _ => "bad rotation",
                }
            },
        }
    }
}
#[derive(Component)]
struct TileRotation {
    rotation: usize,      // facing 3 means rotated twice.
}

/// Marker of placement data
#[derive(Component)]
pub struct TilePlacementData
{
    pub color: TileColor,
    pub upgrade_list: Vec<u32>,
    pub facings: TileFacings,
}

/// Marker that a particular tile has a junction in its track
#[derive(Component)]
pub struct TileHasJunction;

/// Marker that a particular tile has a crossover in its track
#[derive(Component)]
pub struct TileHasCrossover;

// The notion of a City is very important for computing routes.
// A route must include two or more cities that are connected by track.
// The train generates revenue based upon the values of the cities on
// that route. “city” refers to a large city, a small city,
// or an off-board red hex. 
//
// The locations and types of cities is fixed. 
//
// Each city has a revenue value. When a train runs, it generates
// revenue equal to the sum of the revenue values of all of the cities
// (a minimum of 2) on its route. Each of the red off-board areas
// has two values. Initially, use the lesser values. After the first
// 5-train is purchased, use the greater values.
pub enum CityType
{
    LargeCity,
    SmallCity,
    RedHex,
}
pub struct City
{
    city_type: CityType,
    hex: String, // e.g., "A19"
    name: String, // e.g., "Baltimore", "Chicago", "Gulf"
    value: u32,
    value_phase_five: u32, // only for RedHex, means value starting in phase 5
}
    

/// How many copies of a given track tile remain available to be placed.
///
/// Each Tile has its own current inventory, which determines
/// whether a tile is by that number can be placed at this time.
///
/// `quantity` drops by one each time the tile is laid on the map, and rises by
/// one when a placed tile is lifted (i.e. replaced by an upgrade).
#[derive(Component)]
pub struct TileInventoryQuantity {
    pub quantity: u32,
}

/// Startup system that builds the initial track-tile inventory.
///
/// For each distinct track tile in the 1830 set, this should:
///   1. spawn one entity with `(Tile { tile_number },
///      TileInventoryQuantity { quantity })`, `quantity` being the starting
///      count for that tile, and
///   2. record it in `game_state.inventory_by_number`, keyed by `tile_number`,
///      so `place_tile` can look it up directly.
pub fn spawn_tracktile_inventory(
    mut commands: Commands,
    mut game_state: ResMut<GameState>,
) {
    game_state.inventory_by_number.insert(1,
        commands.spawn(( Tile { tile_number: 1 },
                    TileInventoryQuantity { quantity: 1 },
        )).id() );
    game_state.inventory_by_number.insert(2,
        commands.spawn(( Tile { tile_number: 2 },
                    TileInventoryQuantity { quantity: 1 },
        )).id() );
    game_state.inventory_by_number.insert(3,
        commands.spawn(( Tile { tile_number: 3 },
                    TileInventoryQuantity { quantity: 2 },
        )).id() );
    game_state.inventory_by_number.insert(4,
        commands.spawn(( Tile { tile_number: 4 },
                    TileInventoryQuantity { quantity: 2 },
        )).id() );
    game_state.inventory_by_number.insert(7,
        commands.spawn(( Tile { tile_number: 7 },
                    TileInventoryQuantity { quantity: 4 },
        )).id() );
    game_state.inventory_by_number.insert(8,
        commands.spawn(( Tile { tile_number: 8 },
                    TileInventoryQuantity { quantity: 8 },
        )).id() );
    game_state.inventory_by_number.insert(9,
        commands.spawn(( Tile { tile_number: 9 },
                    TileInventoryQuantity { quantity: 7 },
        )).id() );
    game_state.inventory_by_number.insert(14,
        commands.spawn(( Tile { tile_number: 14 },
                    TileInventoryQuantity { quantity: 3 },
        )).id() );
    game_state.inventory_by_number.insert(15,
        commands.spawn(( Tile { tile_number: 15 },
                    TileInventoryQuantity { quantity: 2 },
        )).id() );
    game_state.inventory_by_number.insert(16,
        commands.spawn(( Tile { tile_number: 16 },
                    TileInventoryQuantity { quantity: 1 },
        )).id() );
    game_state.inventory_by_number.insert(18,
        commands.spawn(( Tile { tile_number: 18 },
                    TileInventoryQuantity { quantity: 1 },
        )).id() );
    game_state.inventory_by_number.insert(19,
        commands.spawn(( Tile { tile_number: 19 },
                    TileInventoryQuantity { quantity: 1 },
        )).id() );
    game_state.inventory_by_number.insert(20,
        commands.spawn(( Tile { tile_number: 20 },
                    TileInventoryQuantity { quantity: 1 },
        )).id() );
    game_state.inventory_by_number.insert(23,
        commands.spawn(( Tile { tile_number: 23 },
                    TileInventoryQuantity { quantity: 3 },
        )).id() );
    game_state.inventory_by_number.insert(24,
        commands.spawn(( Tile { tile_number: 24 },
                    TileInventoryQuantity { quantity: 3 },
        )).id() );
    game_state.inventory_by_number.insert(25,
        commands.spawn(( Tile { tile_number: 25 },
                    TileInventoryQuantity { quantity: 1 },
        )).id() );
    game_state.inventory_by_number.insert(26,
        commands.spawn(( Tile { tile_number: 26 },
                    TileInventoryQuantity { quantity: 1 },
        )).id() );
    game_state.inventory_by_number.insert(27,
        commands.spawn(( Tile { tile_number: 27 },
                    TileInventoryQuantity { quantity: 1 },
        )).id() );
    game_state.inventory_by_number.insert(28,
        commands.spawn(( Tile { tile_number: 28 },
                    TileInventoryQuantity { quantity: 1 },
        )).id() );
    game_state.inventory_by_number.insert(29,
        commands.spawn(( Tile { tile_number: 29 },
                    TileInventoryQuantity { quantity: 1 },
        )).id() );
    game_state.inventory_by_number.insert(39,
        commands.spawn(( Tile { tile_number: 39 },
                    TileInventoryQuantity { quantity: 1 },
        )).id() );
    game_state.inventory_by_number.insert(40,
        commands.spawn(( Tile { tile_number: 40 },
                    TileInventoryQuantity { quantity: 1 },
        )).id() );
    game_state.inventory_by_number.insert(41,
        commands.spawn(( Tile { tile_number: 41 },
                    TileInventoryQuantity { quantity: 2 },
        )).id() );
    game_state.inventory_by_number.insert(42,
        commands.spawn(( Tile { tile_number: 42 },
                    TileInventoryQuantity { quantity: 2 },
        )).id() );
    game_state.inventory_by_number.insert(43,
        commands.spawn(( Tile { tile_number: 43 },
                    TileInventoryQuantity { quantity: 2 },
        )).id() );
    game_state.inventory_by_number.insert(44,
        commands.spawn(( Tile { tile_number: 44 },
                    TileInventoryQuantity { quantity: 1 },
        )).id() );
    game_state.inventory_by_number.insert(45,
        commands.spawn(( Tile { tile_number: 45 },
                    TileInventoryQuantity { quantity: 2 },
        )).id() );
    game_state.inventory_by_number.insert(46,
        commands.spawn(( Tile { tile_number: 46 },
                    TileInventoryQuantity { quantity: 2 },
        )).id() );
    game_state.inventory_by_number.insert(47,
        commands.spawn(( Tile { tile_number: 47 },
                    TileInventoryQuantity { quantity: 1 },
        )).id() );
    game_state.inventory_by_number.insert(53,
        commands.spawn(( Tile { tile_number: 53 },
                    TileInventoryQuantity { quantity: 2 },
        )).id() );
    game_state.inventory_by_number.insert(54,
        commands.spawn(( Tile { tile_number: 54 },
                    TileInventoryQuantity { quantity: 1 },
        )).id() );
    game_state.inventory_by_number.insert(55,
        commands.spawn(( Tile { tile_number: 55 },
                    TileInventoryQuantity { quantity: 1 },
        )).id() );
    game_state.inventory_by_number.insert(56,
        commands.spawn(( Tile { tile_number: 56 },
                    TileInventoryQuantity { quantity: 1 },
        )).id() );
    game_state.inventory_by_number.insert(57,
        commands.spawn(( Tile { tile_number: 57 },
                    TileInventoryQuantity { quantity: 4 },
        )).id() );
    game_state.inventory_by_number.insert(58,
        commands.spawn(( Tile { tile_number: 58 },
                    TileInventoryQuantity { quantity: 2 },
        )).id() );
    game_state.inventory_by_number.insert(59,
        commands.spawn(( Tile { tile_number: 59 },
                    TileInventoryQuantity { quantity: 2 },
        )).id() );
    game_state.inventory_by_number.insert(61,
        commands.spawn(( Tile { tile_number: 61 },
                    TileInventoryQuantity { quantity: 2 },
        )).id() );
    game_state.inventory_by_number.insert(62,
        commands.spawn(( Tile { tile_number: 62 },
                    TileInventoryQuantity { quantity: 1 },
        )).id() );
    game_state.inventory_by_number.insert(63,
        commands.spawn(( Tile { tile_number: 63 },
                    TileInventoryQuantity { quantity: 3 },
        )).id() );
    game_state.inventory_by_number.insert(64,
        commands.spawn(( Tile { tile_number: 64 },
                    TileInventoryQuantity { quantity: 1 },
        )).id() );
    game_state.inventory_by_number.insert(65,
        commands.spawn(( Tile { tile_number: 65 },
                    TileInventoryQuantity { quantity: 1 },
        )).id() );
    game_state.inventory_by_number.insert(66,
        commands.spawn(( Tile { tile_number: 66 },
                    TileInventoryQuantity { quantity: 1 },
        )).id() );
    game_state.inventory_by_number.insert(67,
        commands.spawn(( Tile { tile_number: 67 },
                    TileInventoryQuantity { quantity: 1 },
        )).id() );
    game_state.inventory_by_number.insert(68,
        commands.spawn(( Tile { tile_number: 68 },
                    TileInventoryQuantity { quantity: 1 },
        )).id() );
    game_state.inventory_by_number.insert(69,
        commands.spawn(( Tile { tile_number: 69 },
                    TileInventoryQuantity { quantity: 1 },
        )).id() );
    game_state.inventory_by_number.insert(70,
        commands.spawn(( Tile { tile_number: 70 },
                    TileInventoryQuantity { quantity: 1 },
        )).id() );

    info!("loaded inventory_by_number with {} entities", 
            game_state.inventory_by_number.len());
}

fn load_cities(game_state: & mut GameState)
{
    game_state.cities.insert("A9".to_string(),
        City {
            city_type: CityType::RedHex,
            hex: "A9".to_string(),
            name: "Canadian West".to_string(),
            value: 30,
            value_phase_five: 50,
        });
    game_state.cities.insert("A11".to_string(),
        City {
            city_type: CityType::RedHex,
            hex: "A11".to_string(),
            name: "Canadian West".to_string(),
            value: 30,
            value_phase_five: 50,
        });
    game_state.cities.insert("A19".to_string(),
        City {
            city_type: CityType::LargeCity,
            hex: "A19".to_string(),
            name: "Montreal".to_string(),
            value: 40,
            value_phase_five: 40,
        });
    game_state.cities.insert("B10".to_string(),
        City {
            city_type: CityType::LargeCity,
            hex: "B10".to_string(),
            name: "B10".to_string(),
            value: 20,
            value_phase_five: 20,
        });
    game_state.cities.insert("B16".to_string(),
        City {
            city_type: CityType::LargeCity,
            hex: "B16".to_string(),
            name: "B16".to_string(),
            value: 20,
            value_phase_five: 20,
        });
    game_state.cities.insert("B20".to_string(),
        City {
            city_type: CityType::SmallCity,
            hex: "B20".to_string(),
            name: "B20".to_string(),
            value: 10,
            value_phase_five: 10,
        });
    game_state.cities.insert("B24".to_string(),
        City {
            city_type: CityType::RedHex,
            hex: "B24".to_string(),
            name: "Maritime Province".to_string(),
            value: 20,
            value_phase_five: 30,
        });
    game_state.cities.insert("C15".to_string(),
        City {
            city_type: CityType::SmallCity,
            hex: "C15".to_string(),
            name: "C15".to_string(),
            value: 10,
            value_phase_five: 10,
        });
    game_state.cities.insert("D2".to_string(),
        City {
            city_type: CityType::LargeCity,
            hex: "D2".to_string(),
            name: "D2".to_string(),
            value: 20,
            value_phase_five: 20,
        });
    game_state.cities.insert("D4".to_string(),
        City {
            city_type: CityType::SmallCity,
            hex: "D4".to_string(),
            name: "D4".to_string(),
            value: 10,
            value_phase_five: 10,
        });
    game_state.cities.insert("D10".to_string(),
        City {
            city_type: CityType::LargeCity, // This is a O-O map hex
            hex: "D10".to_string(),
            name: "D10".to_string(),
            value: 20,
            value_phase_five: 20,
        });
    game_state.cities.insert("D14".to_string(),
        City {
            city_type: CityType::LargeCity,
            hex: "D14".to_string(),
            name: "D14".to_string(),
            value: 20,
            value_phase_five: 20,
        });
    game_state.cities.insert("E5".to_string(),
        City {
            city_type: CityType::LargeCity, // This is a O-O map hex
            hex: "E5".to_string(),
            name: "E5".to_string(),
            value: 20,
            value_phase_five: 20,
        });
    game_state.cities.insert("E7".to_string(),
        City {
            city_type: CityType::SmallCity,
            hex: "E7".to_string(),
            name: "E7".to_string(),
            value: 10,
            value_phase_five: 10,
        });
    game_state.cities.insert("E11".to_string(),
        City {
            city_type: CityType::LargeCity, // This is a O-O map hex
            hex: "E11".to_string(),
            name: "Erie".to_string(),
            value: 20,
            value_phase_five: 20,
        });
    game_state.cities.insert("E19".to_string(),
        City {
            city_type: CityType::LargeCity,
            hex: "E19".to_string(),
            name: "Albany".to_string(),
            value: 20,
            value_phase_five: 20,
        });
    game_state.cities.insert("E23".to_string(),
        City {
            city_type: CityType::LargeCity,
            hex: "E23".to_string(),
            name: "Boston".to_string(),
            value: 30,
            value_phase_five: 30,
        });
    game_state.cities.insert("F2".to_string(),
        City {
            city_type: CityType::RedHex,
            hex: "F2".to_string(),
            name: "Chicago".to_string(),
            value: 40,
            value_phase_five: 70,
        });
    game_state.cities.insert("F4".to_string(),
        City {
            city_type: CityType::LargeCity,
            hex: "F4".to_string(),
            name: "F4".to_string(),
            value: 20,
            value_phase_five: 20,
        });
    game_state.cities.insert("F6".to_string(),
        City {
            city_type: CityType::LargeCity,
            hex: "F6".to_string(),
            name: "Cleveland".to_string(),
            value: 30,
            value_phase_five: 30,
        });
    game_state.cities.insert("F10".to_string(),
        City {
            city_type: CityType::SmallCity,
            hex: "F10".to_string(),
            name: "F10".to_string(),
            value: 10,
            value_phase_five: 10,
        });
    game_state.cities.insert("F16".to_string(),
        City {
            city_type: CityType::LargeCity,
            hex: "F16".to_string(),
            name: "F16".to_string(),
            value: 20,
            value_phase_five: 20,
        });
    game_state.cities.insert("F20".to_string(),
        City {
            city_type: CityType::SmallCity,
            hex: "F20".to_string(),
            name: "F20".to_string(),
            value: 10,
            value_phase_five: 10,
        });
    game_state.cities.insert("F22".to_string(),
        City {
            city_type: CityType::LargeCity,
            hex: "F22".to_string(),
            name: "F22".to_string(),
            value: 20,
            value_phase_five: 20,
        });
    game_state.cities.insert("F24".to_string(),
        City {
            city_type: CityType::SmallCity,
            hex: "F24".to_string(),
            name: "F24".to_string(),
            value: 10,
            value_phase_five: 10,
        });
    game_state.cities.insert("G7".to_string(),
        City {
            city_type: CityType::SmallCity,
            hex: "G7".to_string(),
            name: "G7".to_string(),
            value: 10,
            value_phase_five: 10,
        });
    game_state.cities.insert("G17".to_string(),
        City {
            city_type: CityType::SmallCity,
            hex: "G17".to_string(),
            name: "G17".to_string(),
            value: 10,
            value_phase_five: 10,
        });
    game_state.cities.insert("G19".to_string(),
        City {
            city_type: CityType::LargeCity, // This is New York City special
            hex: "G19".to_string(),
            name: "New York City".to_string(),
            value: 40,
            value_phase_five: 40,
        });
    game_state.cities.insert("H4".to_string(),
        City {
            city_type: CityType::LargeCity,
            hex: "H4".to_string(),
            name: "H4".to_string(),
            value: 20,
            value_phase_five: 20,
        });
    game_state.cities.insert("H10".to_string(),
        City {
            city_type: CityType::LargeCity,
            hex: "H10".to_string(),
            name: "H10".to_string(),
            value: 20,
            value_phase_five: 20,
        });
    game_state.cities.insert("H12".to_string(),
        City {
            city_type: CityType::LargeCity,
            hex: "H12".to_string(),
            name: "Altoona".to_string(),
            value: 10,
            value_phase_five: 10,
        });
    game_state.cities.insert("H16".to_string(),
        City {
            city_type: CityType::LargeCity,
            hex: "H16".to_string(),
            name: "H16".to_string(),
            value: 20,
            value_phase_five: 20,
        });
    game_state.cities.insert("H18".to_string(),
        City {
            city_type: CityType::LargeCity, // This is a O-O map hex
            hex: "H18".to_string(),
            name: "H18".to_string(),
            value: 20,
            value_phase_five: 20,
        });
    game_state.cities.insert("I1".to_string(),
        City {
            city_type: CityType::RedHex,
            hex: "I1".to_string(),
            name: "Gulf".to_string(),
            value: 30,
            value_phase_five: 60,
        });
    game_state.cities.insert("I15".to_string(),
        City {
            city_type: CityType::LargeCity,
            hex: "I15".to_string(),
            name: "Baltimore".to_string(),
            value: 30,
            value_phase_five: 30,
        });
    game_state.cities.insert("I19".to_string(),
        City {
            city_type: CityType::SmallCity,
            hex: "I19".to_string(),
            name: "I19".to_string(),
            value: 10,
            value_phase_five: 10,
        });
    game_state.cities.insert("J2".to_string(),
        City {
            city_type: CityType::RedHex,
            hex: "J2".to_string(),
            name: "Gulf".to_string(),
            value: 30,
            value_phase_five: 60,
        });
    game_state.cities.insert("J14".to_string(),
        City {
            city_type: CityType::LargeCity,
            hex: "J14".to_string(),
            name: "J14".to_string(),
            value: 20,
            value_phase_five: 20,
        });
    game_state.cities.insert("K13".to_string(),
        City {
            city_type: CityType::RedHex,
            hex: "K13".to_string(),
            name: "Deep South".to_string(),
            value: 30,
            value_phase_five: 40,
        });
    game_state.cities.insert("K15".to_string(),
        City {
            city_type: CityType::LargeCity,
            hex: "K15".to_string(),
            name: "K15".to_string(),
            value: 20,
            value_phase_five: 20,
        });



    info!("loaded cities with {} entities", game_state.cities.len());
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

    // info!("location of worldPos {},{} is {:?}", pos.x,pos.y,hex);

        hex
    }
}

impl TileTrack
{
    pub fn rotate_tile_track(&self, rotation: usize) -> [usize;6] {
        // the output position for each input position:
        let o0 : usize = ( ( 0 ) + rotation) % 6;
        let o1 : usize = ( ( 1 ) + rotation) % 6;
        let o2 : usize = ( ( 2 ) + rotation) % 6;
        let o3 : usize = ( ( 3 ) + rotation) % 6;
        let o4 : usize = ( ( 4 ) + rotation) % 6;
        let o5 : usize = ( ( 5 ) + rotation) % 6;

        let mut result = [0;6];

        result[o0] = self.track[0];
        result[o1] = self.track[1];
        result[o2] = self.track[2];
        result[o3] = self.track[3];
        result[o4] = self.track[4];
        result[o5] = self.track[5];

        result
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
        (HexName::new("A9"),  Hex::new( 1, -5),  "A9", 
            HexColor::Red, [0,0,1,0,0,0], Vec::new()),
        (HexName::new("A11"), Hex::new( 2, -5), "A11", 
            HexColor::Red, [0,0,1,1,0,0], Vec::new()),
        (HexName::new("A17"), Hex::new( 5, -5), "A17", 
            HexColor::Gray, [0,0,1,1,0,0], Vec::new()),
        (HexName::new("A19"), Hex::new( 6, -5), "A19", 
            HexColor::Gray, [0,0,1,1,0,0], Vec::new()),

        (HexName::new("B10"), Hex::new( 1, -4), "blank_1large", 
            HexColor::Tan, [0;6], vec![57]),
        (HexName::new("B12"), Hex::new( 2, -4), "blank", 
            HexColor::Tan, [0;6], vec![7,8,9]),
        (HexName::new("B14"), Hex::new( 3, -4), "blank", 
            HexColor::Tan, [0;6], vec![7,8,9]),
        (HexName::new("B16"), Hex::new( 4, -4), "blank_1large", 
            HexColor::Tan, [0;6], vec![57]),
        (HexName::new("B18"), Hex::new( 5, -4), "B18", 
            HexColor::Tan, [0;6], vec![7,8,9]),
        (HexName::new("B20"), Hex::new( 6, -4), "B20", 
            HexColor::Tan, [0;6], vec![3,4,58]),
        (HexName::new("B22"), Hex::new( 7, -4), "blank", 
            HexColor::Tan, [0;6], vec![7,8,9]),
        (HexName::new("B24"), Hex::new( 8, -4), "B24", 
            HexColor::Red, [0,0,0,1,1,0], Vec::new()),

        (HexName::new("C7"),  Hex::new(-1, -3), "blank", 
            HexColor::Tan, [0;6], vec![7,8,9]),
        (HexName::new("C9"),  Hex::new( 0, -3), "blank", 
            HexColor::Tan, [0;6], vec![7,8,9]),
        (HexName::new("C11"), Hex::new( 1, -3), "blank", 
            HexColor::Tan, [0;6], vec![7,8,9]),
        (HexName::new("C13"), Hex::new( 2, -3), "blank", 
            HexColor::Tan, [0;6], vec![7,8,9]),
        (HexName::new("C15"), Hex::new( 3, -3), "C15", 
            HexColor::Gray, [1,0,0,0,1,0], Vec::new()),
        (HexName::new("C17"), Hex::new( 4, -3), "blank_mountain", 
            HexColor::Tan, [0;6], vec![7,8,9]),
        (HexName::new("C19"), Hex::new( 5, -3), "C19", 
            HexColor::Tan, [0;6], vec![7,8,9]),
        (HexName::new("C21"), Hex::new( 6, -3), "blank_mountain", 
            HexColor::Tan, [0;6], vec![7,8,9]),
        (HexName::new("C23"), Hex::new( 7, -3), "blank", 
            HexColor::Tan, [0;6], vec![7,8,9]),

        (HexName::new("D2"),  Hex::new(-4, -2), "D2", 
            HexColor::Gray, [0,1,1,0,0,0], Vec::new()),
        (HexName::new("D4"),  Hex::new(-3, -2), "blank_1small", 
            HexColor::Tan, [0;6], vec![3,4,58]),
        (HexName::new("D6"),  Hex::new(-2, -2), "D6", 
            HexColor::Tan, [0;6], vec![7,8,9]),
        (HexName::new("D8"),  Hex::new(-1, -2), "blank", 
            HexColor::Tan, [0;6], vec![7,8,9]),
        (HexName::new("D10"), Hex::new( 0, -2), "D10", 
            HexColor::Yellow, [0;6], vec![59]),
        (HexName::new("D12"), Hex::new( 1, -2), "blank", 
            HexColor::Tan, [0;6], vec![7,8,9]),
        (HexName::new("D14"), Hex::new( 2, -2), "D14", 
            HexColor::Gray, [0,1,0,1,1,0], Vec::new()),
        (HexName::new("D16"), Hex::new( 3, -2), "blank", 
            HexColor::Tan, [0;6], vec![7,8,9]),
        (HexName::new("D18"), Hex::new( 4, -2), "D18", 
            HexColor::Tan, [0;6], vec![7,8,9]),
        (HexName::new("D20"), Hex::new( 5, -2), "blank", 
            HexColor::Tan, [0;6], vec![7,8,9]),
        (HexName::new("D22"), Hex::new( 6, -2), "blank_mountain", 
            HexColor::Tan, [0;6], vec![7,8,9]),
        (HexName::new("D24"), Hex::new( 7, -2), "D24", 
            HexColor::Gray, [0,0,0,1,1,0], Vec::new()),

        (HexName::new("E3"),  Hex::new(-4, -1), "blank", 
            HexColor::Tan, [0;6], vec![7,8,9]),
        (HexName::new("E5"),  Hex::new(-3, -1), "E5", 
            HexColor::Yellow, [0;6], vec![59]),
        (HexName::new("E7"),  Hex::new(-2, -1), "blank_1small", 
            HexColor::Tan, [0;6], vec![3,4,58]),
        (HexName::new("E9"),  Hex::new(-1, -1), "E9", 
            HexColor::Gray, [1,0,0,0,0,1], Vec::new()),
        (HexName::new("E11"), Hex::new( 0, -1), "E11", 
            HexColor::Yellow, [0;6], vec![59]),
        (HexName::new("E13"), Hex::new( 1, -1), "blank", 
            HexColor::Tan, [0;6], vec![7,8,9]),
        (HexName::new("E15"), Hex::new( 2, -1), "blank", 
            HexColor::Tan, [0;6], vec![7,8,9]),
        (HexName::new("E17"), Hex::new( 3, -1), "blank_mountain", 
            HexColor::Tan, [0;6], vec![7,8,9]),
        (HexName::new("E19"), Hex::new( 4, -1), "E19", 
            HexColor::Tan, [0;6], vec![57]),
        (HexName::new("E21"), Hex::new( 5, -1), "blank_mountain", 
            HexColor::Tan, [0;6], vec![7,8,9]),
        (HexName::new("E23"), Hex::new( 6, -1), "E23", 
            HexColor::Yellow, [1,0,1,0,0,0], vec![53]),

        (HexName::new("F2"),  Hex::new(-5,  0), "F2", 
            HexColor::Red, [1,1,1,0,0,0], Vec::new()),
        (HexName::new("F4"),  Hex::new(-4,  0), "F4", 
            HexColor::Tan, [0;6], vec![57]),
        (HexName::new("F6"),  Hex::new(-3,  0), "F6", 
            HexColor::Gray, [0,0,1,1,0,0], Vec::new()),
        (HexName::new("F8"),  Hex::new(-2,  0), "blank", 
            HexColor::Tan, [0;6], vec![7,8,9]),
        (HexName::new("F10"), Hex::new(-1,  0), "blank_1small", 
            HexColor::Tan, [0;6], vec![3,4,58]),
                // F12 -- CENTER OF THE SCREEN
        (HexName::new("F12"), Hex::new( 0,  0), "blank", 
            HexColor::Tan, [0;6], vec![7,8,9]),
                // F12 -- CENTER OF THE SCREEN
        (HexName::new("F14"), Hex::new( 1,  0), "blank", 
            HexColor::Tan, [0;6], vec![7,8,9]),
        (HexName::new("F16"), Hex::new( 2,  0), "F16", 
            HexColor::Tan, [0;6], vec![57]),
        (HexName::new("F18"), Hex::new( 3,  0), "blank", 
            HexColor::Tan, [0;6], vec![7,8,9]),
        (HexName::new("F20"), Hex::new( 4,  0), "blank_2small", 
            HexColor::Tan, [0;6], vec![1,2,55,56,69]),
        (HexName::new("F22"), Hex::new( 5,  0), "F22", 
            HexColor::Tan, [0;6], vec![57]),
        (HexName::new("F24"), Hex::new( 6,  0), "F24", 
            HexColor::Gray, [0,0,0,0,1,1], Vec::new()),

        (HexName::new("G3"),  Hex::new(-5,  1), "blank", 
            HexColor::Tan, [0;6], vec![7,8,9]),
        (HexName::new("G5"),  Hex::new(-4,  1), "blank", 
            HexColor::Tan, [0;6], vec![7,8,9]),
        (HexName::new("G7"),  Hex::new(-3,  1), "blank_2small", 
            HexColor::Tan, [0;6], vec![1,2,55,56,69]),
        (HexName::new("G9"),  Hex::new(-2,  1), "blank", 
            HexColor::Tan, [0;6], vec![7,8,9]),
        (HexName::new("G11"), Hex::new(-1,  1), "blank", 
            HexColor::Tan, [0;6], vec![7,8,9]),
        (HexName::new("G13"), Hex::new( 0,  1), "blank_mountain", 
            HexColor::Tan, [0;6], vec![7,8,9]),
        (HexName::new("G15"), Hex::new( 1,  1), "G15", 
            HexColor::Tan, [0;6], vec![7,8,9]),
        (HexName::new("G17"), Hex::new( 2,  1), "blank_2small", 
            HexColor::Tan, [0;6], vec![1,2,55,56,69]),
        (HexName::new("G19"), Hex::new( 3,  1), "G19", 
            HexColor::Yellow, [1,0,0,1,0,0], vec![54]),

        (HexName::new("H2"),  Hex::new(-6,  2), "blank", 
            HexColor::Tan, [0;6], vec![7,8,9]),
        (HexName::new("H4"),  Hex::new(-5,  2), "blank_1large", 
            HexColor::Tan, [0;6], vec![57]),
        (HexName::new("H6"),  Hex::new(-4,  2), "blank", 
            HexColor::Tan, [0;6], vec![7,8,9]),
        (HexName::new("H8"),  Hex::new(-3,  2), "blank", 
            HexColor::Tan, [0;6], vec![7,8,9]),
        (HexName::new("H10"), Hex::new(-2,  2), "blank_1large", 
            HexColor::Tan, [0;6], vec![57]),
        (HexName::new("H12"), Hex::new(-1,  2), "H12", 
            HexColor::Gray, [0,1,0,0,1,0], Vec::new()),
        (HexName::new("H14"), Hex::new( 0,  2), "blank", 
            HexColor::Tan, [0;6], vec![7,8,9]),
        (HexName::new("H16"), Hex::new( 1,  2), "blank_1large", 
            HexColor::Tan, [0;6], vec![57]),
        (HexName::new("H18"), Hex::new( 2,  2), "H18", 
            HexColor::Yellow, [0;6], vec![59]),
        (HexName::new("I1"),  Hex::new(-7,  3), "blank", 
            HexColor::Red, [0,1,0,0,0,0], Vec::new()),
        (HexName::new("I3"),  Hex::new(-6,  3), "blank", 
            HexColor::Tan, [0;6], vec![7,8,9]),
        (HexName::new("I5"),  Hex::new(-5,  3), "blank", 
            HexColor::Tan, [0;6], vec![7,8,9]),
        (HexName::new("I7"),  Hex::new(-4,  3), "blank", 
            HexColor::Tan, [0;6], vec![7,8,9]),
        (HexName::new("I9"),  Hex::new(-3,  3), "blank", 
            HexColor::Tan, [0;6], vec![7,8,9]),
        (HexName::new("I11"), Hex::new(-2,  3), "blank_mountain", 
            HexColor::Tan, [0;6], vec![7,8,9]),
        (HexName::new("I13"), Hex::new(-1,  3), "blank", 
            HexColor::Tan, [0;6], vec![7,8,9]),
        (HexName::new("I15"), Hex::new( 0,  3), "I15", 
            HexColor::Yellow, [0,1,0,1,0,0], vec![53]),
        (HexName::new("I17"), Hex::new( 1,  3), "I17", 
            HexColor::Tan, [0;6], vec![7,8,9]),
        (HexName::new("I19"), Hex::new( 2,  3), "I19", 
            HexColor::Gray, [0,0,0,0,1,1], Vec::new()),

        (HexName::new("J2"),  Hex::new(-7,  4), "J2", 
            HexColor::Red, [1,1,0,0,0,0], Vec::new()),
        (HexName::new("J4"),  Hex::new(-6,  4), "blank", 
            HexColor::Tan, [0;6], vec![7,8,9]),
        (HexName::new("J6"),  Hex::new(-5,  4), "blank", 
            HexColor::Tan, [0;6], vec![7,8,9]),
        (HexName::new("J8"),  Hex::new(-4,  4), "blank", 
            HexColor::Tan, [0;6], vec![7,8,9]),
        (HexName::new("J10"), Hex::new(-3,  4), "blank_mountain", 
            HexColor::Tan, [0;6], vec![7,8,9]),
        (HexName::new("J12"), Hex::new(-2,  4), "blank_mountain", 
            HexColor::Tan, [0;6], vec![7,8,9]),
        (HexName::new("J14"), Hex::new(-1,  4), "J14", 
            HexColor::Tan, [0;6], vec![57]),

        (HexName::new("K13"), Hex::new(-2,  5), "K13", 
            HexColor::Red, [1,0,0,0,0,1], Vec::new()),
        (HexName::new("K15"), Hex::new(-1,  5), "K15", 
            HexColor::Gray, [0,0,0,0,0,1], Vec::new()),
    ];

    // For each hex in our game map, a MapTile component holds
    // various metadata about that hex's current state. The
    // MapTile entities may be rapidly located by using global
    // indices in the GameState resource.
    //
    // GameState keeps only the indices, so any system can resolve
    // a hex -- by coordinate or by name -- to its entity,
    // then read/mutate the `MapTile` through a query.

    for (hex_name, coord, tile_name, hcolor, tpattern, tile_list) in hexes
    {
        let world_pos = settings.hex_to_world_pos(coord);

        let entity = commands.spawn((
            Sprite::from_image(asset_server.load(
                                format!("Map/{}.png", tile_name))),
            Transform::from_xyz(world_pos.x, world_pos.y, 0.0),
            MapTile {
                coord,
                hex_name: hex_name.clone(),
                hex_color: hcolor,
                market: HashMap::new(),
                tile_name: tile_name.to_string(),
                placed_tile: 0, // 0 = no track tile placed yet
                track: tpattern,
                tile_choices: tile_list,
            },
        )).id();

        game_state.tile_by_coord.insert(coord, entity);
        game_state.tile_by_name.insert(hex_name.name, entity);
    }

    load_cities(&mut game_state);
}

// If no tile has yet been placed on this hex, use the MapTile data
// to find out the available tile choices and set them in the track_state.
// If there is already a tile on this hex, use the tile data
// to find out the available tile choices and set them in the track_state.
//
// set rotation to zero in the track_state.
//
// set the image for the hex to show how it would look with this tile

pub fn set_tile_choices_for_hex( game_state: &mut GameState,
                    map_tile: & MapTile,
                    sprite: &mut Sprite,
                    placement: &Query<(&TilePlacementData, &TileTrack)>,
                    asset_server: &AssetServer,
) {
    if map_tile.placed_tile == 0
    {
        // no tile has yet been placed on this hex

        game_state.track_state.tile_choices = map_tile.tile_choices.clone();
        let num_choices = game_state.track_state.tile_choices.len();
        if num_choices != 0
        {
            game_state.track_state.tile_chosen = map_tile.tile_choices[0];
            game_state.track_state.tile_rotation = 0;
        }
        else
        {
            info!("No tile can be placed on this hex");
            return;
        }
    }
    else
    {
        // a tile is already present on this hex

        let new_number = map_tile.placed_tile;

        let Some(&placement_entity) =
            game_state.tile_placement_data_by_number.get( & new_number) else
        {   
            info!("No placement data for tile number {}", new_number);
            return;
        };
        if let Ok(tile_data) = placement.get(placement_entity)  {
            // info!("loaded placement data for tile number {}", new_number);
            // tile_data.0 is a TilePlacementData, tile_data.1 is TileTrack
 
            game_state.track_state.tile_choices =
                                        tile_data.0.upgrade_list.clone();
            let num_choices = game_state.track_state.tile_choices.len();
            if num_choices != 0
            {
                game_state.track_state.tile_chosen =
                                game_state.track_state.tile_choices[0];
                game_state.track_state.tile_rotation = 0;
            }
            else
            {
                info!("This tile cannot be further upgraded");
                return;
            }
        }
    }

    update_map_for_new_track(game_state, sprite, placement, asset_server);
}

pub fn update_map_for_new_track( game_state: &mut GameState,
                    sprite: &mut Sprite,
                    placement: &Query<(&TilePlacementData, &TileTrack)>,
                    asset_server: &AssetServer,
) {
    let new_number = game_state.track_state.tile_chosen;
    let rotation = game_state.track_state.tile_rotation;

    let Some(&placement_entity) =
        game_state.tile_placement_data_by_number.get( & new_number) else
    {   
        info!("No placement data for tile number {}", new_number);
        return;
    };
    if let Ok(tile_data) = placement.get(placement_entity)  {
        // info!("loaded placement data for tile number {}", new_number);
        // tile_data.0 is a TilePlacementData, tile_data.1 is TileTrack

        let image_path = format!("{}/T{}_{}.png",
            tile_data.0.color.name(),
            new_number,
            tile_data.0.facings.image_name(rotation));

        sprite.image = asset_server.load(image_path.clone());
    }
}

pub fn handle_tile_clicks(
    mut contexts: EguiContexts,
    mut game_state: ResMut<GameState>,
    buttons: Res<ButtonInput<MouseButton>>,
    camera_query: Query<(&Camera, &GlobalTransform), With<super::MainCamera>>,
    windows: Query<&Window>,
    mut maptile_query: Query<(&MapTile, &mut Sprite, &Transform)>,
    placement: Query<(&TilePlacementData, &TileTrack)>,
    settings: Res<HexSettings>,
    asset_server: Res<AssetServer>,
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

            // info!("Is there a tile at {:?}", hex_coord);
            // Check if there's a tile at this coordinate

            for (tile, mut sprite, _transform) in maptile_query.iter_mut() {
                if tile.coord == hex_coord {
                    game_state.track_state.hex_name =
                                            tile.hex_name.name.clone();
                    game_state.track_state.tile_name = tile.tile_name.clone();
                    game_state.track_state.hex_coord = Some(hex_coord);
                    game_state.track_state.hex_chosen = true;
                    set_tile_choices_for_hex(&mut game_state,
                                            &tile,
                                            &mut sprite,
                                            &placement,
                                            &asset_server);
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
   
    Over time, the track starts to form connections and routes emerge.
    Note that this computation is also where we ensure legal tile
    upgrades as they must maintain existing connectivity.
   
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

/// Startup system that builds the tile placement data
///
/// For each distinct tile in the 1830 set, this should:
///
///   1. spawn one entity with the tile_number and information
///      that is needed when that tile is placed on a map hex.
///      In particular this includes the tile's track, but also
///      some other details such as whether the tile has a junction
///      or crossover, etc.
///      Note that these entities track the base (unrotate) track
///      pattern. When an actual tile is placed on the map, the
///      user's chosen rotation is applied to this base track
///      pattern and the rotated result is stored in the MapTile track.
///
///   2. record it in `game_state.inventory_by_number`, keyed by `tile_number`,
///      so `place_tile` can look it up directly.
pub fn spawn_tile_placement_data(
    mut commands: Commands,
    mut game_state: ResMut<GameState>,
) {
    game_state.tile_placement_data_by_number.insert(1,
        commands.spawn(( Tile { tile_number: 1 },
                    TilePlacementData {
                        color: TileColor::Yellow,
                        upgrade_list: Vec::new(),
                        facings: TileFacings::Three,
                    },
                    TileTrack { track: [0,1,1,0,1,1] },
        )).id() );
    game_state.tile_placement_data_by_number.insert(2,
        commands.spawn(( Tile { tile_number: 2 },
                    TilePlacementData {
                        color: TileColor::Yellow,
                        upgrade_list: Vec::new(),
                        facings: TileFacings::Six,
                    },
                    TileTrack { track: [1,1,1,0,0,1] },
        )).id() );
    game_state.tile_placement_data_by_number.insert(3,
        commands.spawn(( Tile { tile_number: 3 },
                    TilePlacementData {
                        color: TileColor::Yellow,
                        upgrade_list: Vec::new(),
                        facings: TileFacings::Six,
                    },
                    TileTrack { track: [1,0,0,0,0,1] },
        )).id() );
    game_state.tile_placement_data_by_number.insert(4,
        commands.spawn(( Tile { tile_number: 4 },
                    TilePlacementData {
                        color: TileColor::Yellow,
                        upgrade_list: Vec::new(),
                        facings: TileFacings::Three,
                    },
                    TileTrack { track: [0,0,1,0,0,1] },
        )).id() );
    game_state.tile_placement_data_by_number.insert(7,
        commands.spawn(( Tile { tile_number: 7 },
                    TilePlacementData {
                        color: TileColor::Yellow,
                        upgrade_list: vec![18,26,27,28,29],
                        facings: TileFacings::Six,
                    },
                    TileTrack { track: [1,0,0,0,0,1] },
        )).id() );
    game_state.tile_placement_data_by_number.insert(8,
        commands.spawn(( Tile { tile_number: 8 },
                    TilePlacementData {
                        color: TileColor::Yellow,
                        upgrade_list: vec![16,19,23,24,25,28,29],
                        facings: TileFacings::Six,
                    },
                    TileTrack { track: [0,1,0,0,0,1] },
        )).id() );
    game_state.tile_placement_data_by_number.insert(9,
        commands.spawn(( Tile { tile_number: 9 },
                    TilePlacementData {
                        color: TileColor::Yellow,
                        upgrade_list: vec![18,19,20,23,24,26,27],
                        facings: TileFacings::Three,
                    },
                    TileTrack { track: [0,0,1,0,0,1] },
        )).id() );
    game_state.tile_placement_data_by_number.insert(14,
        commands.spawn(( Tile { tile_number: 14 },
                    TilePlacementData {
                        color: TileColor::Green,
                        upgrade_list: vec![63],
                        facings: TileFacings::Three,
                    },
                    TileTrack { track: [0,1,1,0,1,1] },
        )).id() );
    game_state.tile_placement_data_by_number.insert(15,
        commands.spawn(( Tile { tile_number: 15 },
                    TilePlacementData {
                        color: TileColor::Green,
                        upgrade_list: vec![63],
                        facings: TileFacings::Six,
                    },
                    TileTrack { track: [1,1,1,0,0,1] },
        )).id() );
    game_state.tile_placement_data_by_number.insert(16,
        commands.spawn(( Tile { tile_number: 16 },
                    TilePlacementData {
                        color: TileColor::Green,
                        upgrade_list: vec![43,70],
                        facings: TileFacings::Six,
                    },
                    TileTrack { track: [1,1,1,0,0,1] },
                    TileHasCrossover,
        )).id() );
    game_state.tile_placement_data_by_number.insert(18,
        commands.spawn(( Tile { tile_number: 18 },
                    TilePlacementData {
                        color: TileColor::Green,
                        upgrade_list: vec![43],
                        facings: TileFacings::Six,
                    },
                    TileTrack { track: [1,1,1,0,0,1] },
        )).id() );
    game_state.tile_placement_data_by_number.insert(19,
        commands.spawn(( Tile { tile_number: 19 },
                    TilePlacementData {
                        color: TileColor::Green,
                        upgrade_list: vec![45,46],
                        facings: TileFacings::Six,
                    },
                    TileTrack { track: [1,1,0,1,0,1] },
                    TileHasCrossover,
        )).id() );
    game_state.tile_placement_data_by_number.insert(20,
        commands.spawn(( Tile { tile_number: 20 },
                    TilePlacementData {
                        color: TileColor::Green,
                        upgrade_list: vec![44,47],
                        facings: TileFacings::Three,
                    },
                    TileTrack { track: [0,1,1,0,1,1] },
                    TileHasCrossover,
        )).id() );
    game_state.tile_placement_data_by_number.insert(23,
        commands.spawn(( Tile { tile_number: 23 },
                    TilePlacementData {
                        color: TileColor::Green,
                        upgrade_list: vec![41,43,45,47],
                        facings: TileFacings::Six,
                    },
                    TileTrack { track: [0,0,1,1,0,1] },
                    TileHasJunction,
        )).id() );
    game_state.tile_placement_data_by_number.insert(24,
        commands.spawn(( Tile { tile_number: 24 },
                    TilePlacementData {
                        color: TileColor::Green,
                        upgrade_list: vec![42,43,46,47],
                        facings: TileFacings::Six,
                    },
                    TileTrack { track: [0,1,1,0,0,1] },
                    TileHasJunction,
        )).id() );
    game_state.tile_placement_data_by_number.insert(25,
        commands.spawn(( Tile { tile_number: 25 },
                    TilePlacementData {
                        color: TileColor::Green,
                        upgrade_list: vec![40,45,46],
                        facings: TileFacings::Six,
                    },
                    TileTrack { track: [0,1,0,1,0,1] },
                    TileHasJunction,
        )).id() );
    game_state.tile_placement_data_by_number.insert(26,
        commands.spawn(( Tile { tile_number: 26 },
                    TilePlacementData {
                        color: TileColor::Green,
                        upgrade_list: vec![42,44,45],
                        facings: TileFacings::Six,
                    },
                    TileTrack { track: [0,0,1,0,1,1] },
                    TileHasJunction,
        )).id() );
    game_state.tile_placement_data_by_number.insert(27,
        commands.spawn(( Tile { tile_number: 27 },
                    TilePlacementData {
                        color: TileColor::Green,
                        upgrade_list: vec![42,44,46],
                        facings: TileFacings::Six,
                    },
                    TileTrack { track: [1,0,1,0,0,1] },
                    TileHasJunction,
        )).id() );
    game_state.tile_placement_data_by_number.insert(28,
        commands.spawn(( Tile { tile_number: 28 },
                    TilePlacementData {
                        color: TileColor::Green,
                        upgrade_list: vec![39,43,46,70],
                        facings: TileFacings::Six,
                    },
                    TileTrack { track: [0,0,0,1,1,1] },
                    TileHasJunction,
        )).id() );
    game_state.tile_placement_data_by_number.insert(29,
        commands.spawn(( Tile { tile_number: 29 },
                    TilePlacementData {
                        color: TileColor::Green,
                        upgrade_list: vec![39,43,45,70],
                        facings: TileFacings::Six,
                    },
                    TileTrack { track: [1,1,0,0,0,1] },
                    TileHasJunction,
        )).id() );
    game_state.tile_placement_data_by_number.insert(39,
        commands.spawn(( Tile { tile_number: 39 },
                    TilePlacementData {
                        color: TileColor::Orange,
                        upgrade_list: Vec::new(),
                        facings: TileFacings::Six,
                    },
                    TileTrack { track: [1,1,0,0,0,1] },
                    TileHasJunction,
        )).id() );
    game_state.tile_placement_data_by_number.insert(40,
        commands.spawn(( Tile { tile_number: 40 },
                    TilePlacementData {
                        color: TileColor::Orange,
                        upgrade_list: Vec::new(),
                        facings: TileFacings::Two,
                    },
                    TileTrack { track: [0,1,0,1,0,1] },
                    TileHasJunction,
        )).id() );
    game_state.tile_placement_data_by_number.insert(41,
        commands.spawn(( Tile { tile_number: 41 },
                    TilePlacementData {
                        color: TileColor::Orange,
                        upgrade_list: Vec::new(),
                        facings: TileFacings::Six,
                    },
                    TileTrack { track: [0,0,1,1,0,1] },
                    TileHasJunction,
        )).id() );
    game_state.tile_placement_data_by_number.insert(42,
        commands.spawn(( Tile { tile_number: 42 },
                    TilePlacementData {
                        color: TileColor::Orange,
                        upgrade_list: Vec::new(),
                        facings: TileFacings::Six,
                    },
                    TileTrack { track: [0,1,1,0,0,1] },
                    TileHasJunction,
        )).id() );
    game_state.tile_placement_data_by_number.insert(43,
        commands.spawn(( Tile { tile_number: 43 },
                    TilePlacementData {
                        color: TileColor::Orange,
                        upgrade_list: Vec::new(),
                        facings: TileFacings::Six,
                    },
                    TileTrack { track: [1,1,1,0,0,1] },
                    TileHasJunction,
                    TileHasCrossover,
        )).id() );
    game_state.tile_placement_data_by_number.insert(44,
        commands.spawn(( Tile { tile_number: 44 },
                    TilePlacementData {
                        color: TileColor::Orange,
                        upgrade_list: Vec::new(),
                        facings: TileFacings::Three,
                    },
                    TileTrack { track: [0,1,1,0,1,1] },
                    TileHasJunction,
                    TileInventoryQuantity { quantity: 1 },
        )).id() );
    game_state.tile_placement_data_by_number.insert(45,
        commands.spawn(( Tile { tile_number: 45 },
                    TilePlacementData {
                        color: TileColor::Orange,
                        upgrade_list: Vec::new(),
                        facings: TileFacings::Six,
                    },
                    TileTrack { track: [0,1,1,1,0,1] },
                    TileHasJunction,
                    TileInventoryQuantity { quantity: 1 },
        )).id() );
    game_state.tile_placement_data_by_number.insert(46,
        commands.spawn(( Tile { tile_number: 46 },
                    TilePlacementData {
                        color: TileColor::Orange,
                        upgrade_list: Vec::new(),
                        facings: TileFacings::Six,
                    },
                    TileTrack { track: [0,1,1,1,0,1] },
                    TileHasJunction,
                    TileInventoryQuantity { quantity: 1 },
        )).id() );
    game_state.tile_placement_data_by_number.insert(47,
        commands.spawn(( Tile { tile_number: 47 },
                    TilePlacementData {
                        color: TileColor::Orange,
                        upgrade_list: Vec::new(),
                        facings: TileFacings::Three,
                    },
                    TileTrack { track: [0,1,1,0,1,1] },
                    TileHasJunction,
                    TileInventoryQuantity { quantity: 1 },
        )).id() );
    game_state.tile_placement_data_by_number.insert(53,
        commands.spawn(( Tile { tile_number: 53 },
                    TilePlacementData {
                        color: TileColor::Green,
                        upgrade_list: vec![61],
                        facings: TileFacings::Two,
                    },
                    TileTrack { track: [0,1,0,1,0,1] },
        )).id() );
    game_state.tile_placement_data_by_number.insert(54,
        commands.spawn(( Tile { tile_number: 54 },
                    TilePlacementData {
                        color: TileColor::Green,
                        upgrade_list: vec![62],
                        facings: TileFacings::One,
                    },
                    TileTrack { track: [1,0,0,1,1,1] },
        )).id() );
    game_state.tile_placement_data_by_number.insert(55,
        commands.spawn(( Tile { tile_number: 55 },
                    TilePlacementData {
                        color: TileColor::Yellow,
                        upgrade_list: Vec::new(),
                        facings: TileFacings::Three,
                    },
                    TileTrack { track: [0,1,1,0,1,1] },
                    TileHasCrossover,
        )).id() );
    game_state.tile_placement_data_by_number.insert(56,
        commands.spawn(( Tile { tile_number: 56 },
                    TilePlacementData {
                        color: TileColor::Yellow,
                        upgrade_list: Vec::new(),
                        facings: TileFacings::Six,
                    },
                    TileTrack { track: [1,1,1,0,0,1] },
                    TileHasCrossover,
        )).id() );
    game_state.tile_placement_data_by_number.insert(57,
        commands.spawn(( Tile { tile_number: 57 },
                    TilePlacementData {
                        color: TileColor::Yellow,
                        upgrade_list: vec![14,15],
                        facings: TileFacings::Three,
                    },
                    TileTrack { track: [0,0,1,0,0,1] },
        )).id() );
    game_state.tile_placement_data_by_number.insert(58,
        commands.spawn(( Tile { tile_number: 58 },
                    TilePlacementData {
                        color: TileColor::Yellow,
                        upgrade_list: Vec::new(),
                        facings: TileFacings::Six,
                    },
                    TileTrack { track: [0,1,0,0,0,1] },
        )).id() );
    game_state.tile_placement_data_by_number.insert(59,
        commands.spawn(( Tile { tile_number: 59 },
                    TilePlacementData {
                        color: TileColor::Green,
                        upgrade_list: vec![64,65,66,67,68],
                        facings: TileFacings::Six,
                    },
                    TileTrack { track: [0,1,0,0,0,1] },
        )).id() );
    game_state.tile_placement_data_by_number.insert(61,
        commands.spawn(( Tile { tile_number: 61 },
                    TilePlacementData {
                        color: TileColor::Orange,
                        upgrade_list: Vec::new(),
                        facings: TileFacings::Six, // Not right, special case
                    },
                    TileTrack { track: [1,1,0,1,0,1] },
        )).id() );
    game_state.tile_placement_data_by_number.insert(62,
        commands.spawn(( Tile { tile_number: 62 },
                    TilePlacementData {
                        color: TileColor::Orange,
                        upgrade_list: Vec::new(),
                        facings: TileFacings::One,
                    },
                    TileTrack { track: [1,0,0,1,1,1] },
        )).id() );
    game_state.tile_placement_data_by_number.insert(63,
        commands.spawn(( Tile { tile_number: 63 },
                    TilePlacementData {
                        color: TileColor::Orange,
                        upgrade_list: Vec::new(),
                        facings: TileFacings::One,
                    },
                    TileTrack { track: [1,1,1,1,1,1] },
        )).id() );
    game_state.tile_placement_data_by_number.insert(64,
        commands.spawn(( Tile { tile_number: 64 },
                    TilePlacementData {
                        color: TileColor::Orange,
                        upgrade_list: Vec::new(),
                        facings: TileFacings::Six,
                    },
                    TileTrack { track: [1,1,1,1,1,1] },
        )).id() );
    game_state.tile_placement_data_by_number.insert(65,
        commands.spawn(( Tile { tile_number: 65 },
                    TilePlacementData {
                        color: TileColor::Orange,
                        upgrade_list: Vec::new(),
                        facings: TileFacings::Six,
                    },
                    TileTrack { track: [0,1,0,1,1,1] },
        )).id() );
    game_state.tile_placement_data_by_number.insert(66,
        commands.spawn(( Tile { tile_number: 66 },
                    TilePlacementData {
                        color: TileColor::Orange,
                        upgrade_list: Vec::new(),
                        facings: TileFacings::Six,
                    },
                    TileTrack { track: [1,1,1,0,0,1] },
        )).id() );
    game_state.tile_placement_data_by_number.insert(67,
        commands.spawn(( Tile { tile_number: 67 },
                    TilePlacementData {
                        color: TileColor::Orange,
                        upgrade_list: Vec::new(),
                        facings: TileFacings::Six,
                    },
                    TileTrack { track: [1,1,0,1,0,1] },
        )).id() );
    game_state.tile_placement_data_by_number.insert(68,
        commands.spawn(( Tile { tile_number: 68 },
                    TilePlacementData {
                        color: TileColor::Orange,
                        upgrade_list: Vec::new(),
                        facings: TileFacings::Three,
                    },
                    TileTrack { track: [0,1,1,0,1,1] },
        )).id() );
    game_state.tile_placement_data_by_number.insert(69,
        commands.spawn(( Tile { tile_number: 69 },
                    TilePlacementData {
                        color: TileColor::Yellow,
                        upgrade_list: Vec::new(),
                        facings: TileFacings::Six,
                    },
                    TileTrack { track: [1,1,0,1,0,1] },
                    TileHasCrossover,
        )).id() );
    game_state.tile_placement_data_by_number.insert(70,
        commands.spawn(( Tile { tile_number: 70 },
                    TilePlacementData {
                        color: TileColor::Orange,
                        upgrade_list: Vec::new(),
                        facings: TileFacings::Six,
                    },
                    TileTrack { track: [1,1,1,0,0,1] },
                    TileHasJunction,
                    TileHasCrossover,
        )).id() );
}

//
// Some terminology that is useful in this section of the code:
//
// - A "path" is a continuous segment of track that connects two
//   map hexes. The most common use of paths is for building "routes"
//   but sometimes we have other paths, for example when deciding
//   if a possible new tile connects to a railroad's station marker.
//
//   There is not necessarily a path between any two arbitrary
//   hexes on the map. We use find_path() to search the map and
//   return a path if there is one, otherwise it returns None.
//
// - A "route" is a path which satisfies a number of additional
//   rules and constraints. A route is a path which a particular
//   railroad can use for running a train and collecting revenue.
//
//   The number of cities on a route includes all of the cities that
//   the route runs through or to. A route may not skip a city that
//   it runs through.
//
//   There may be multiple paths between two hexes. The a_star
//   algorithm returns the shortest path, but if we are trying
//   to form a route, we may need to tell the a_star algorithm
//   about some of the rules and constraints so that it returns
//   a longer (in hexes) path which is better for making a route.
//
//   Specifically, a path which includes a higher number of cities
//   will produce greater revenue, if the railroad has a train
//   with that greater range (a 4-city path is useless if a railroad
//   only has range-2 and range-3 trains).
//
//   Also, a particular path may be blocked by another
//   railroad's station marker but a longer path may be available.
//

pub struct Route
{
    pub path: Vec<Hex>, // the path this route takes
    pub num_cities: u32, // the length of this route in cities.
    pub revenue: u32, // the sum of all the revenues of its cities
}

pub fn form_route(
    game_state: & GameState,
    hexes: &mut Query<(&mut Sprite, &mut MapTile)>,
    path: Vec<Hex>, 
) -> Option<Route>
{
    return None; // stub.

    // Check that the path includes at least one city containing
    // one of the railroad’s stations. If so, compute the num_cities
    // and revenue of the path and return a Route.

    let mut num_cities : u32 = 0;
    let mut revenue : u32 = 0;

    for hex in path
    {
        let hex_entity = *game_state.tile_by_coord.get(&hex)?;

        let Ok((_sprite, map_tile)) = hexes.get(hex_entity) else {
            info!("Map tile entity for {:?} missing", hex);
            return None;
        }; 

        if let Some(city) = game_state.cities.get(&map_tile.hex_name.name) {
            num_cities += 1;
            revenue += city.value; // FIXME value_phase_five

            // FIXME station markers
        }
    }

    return Some(Route {
                path: path,
                num_cities: num_cities,
                revenue: revenue,
    });
}

enum RouteAssignment
{
    // Assigned means that the highest valued Route was successfully
    // given to the shortest-range train that could use it.
    //
    Assigned { revenue: u32, train: Train },

    // No MoreTrains means that the railroad has used all of its
    // trains, and so it can't take advantage of any of the
    // remaining routes.
    //
    NoMoreTrains,

    // NoUsableTrain means that the highest valued Route was skipped
    // because the railroad didn't have a locomotive powerful enough
    // to use it.
    //
    NoUsableTrain { route_length: u32 },
}

fn smallest_train_of_at_least(trains: & mut Vec<Train>, range: u32)
            -> Option<Train>
{
    let mut t_idx : usize = 0;
    let mut found : bool = false;
    while t_idx < trains.len()
    {
        if trains[t_idx].range() >= range
        {
            found = true;
            break;
        }
        t_idx += 1;
    }
    if found
    {
        let train = trains.remove(t_idx);
        return Some(train);
    }
    return None;
}

// Look at the first route in the routes vector and remove it from
// the vector. If the railroad has a train that can run that route,
// find the smallest of the railroad's trains that can be used,
// remove that train from the trains vector, and return which train
// was used and what revenue was earned. If the railroad can't run
// that route, return the reason why. The caller can then decide
// whether to continue trying with the next route in the vector or not.
//
fn assign_route_to_best_train(routes: &mut Vec<Route>,
                                    trains: & mut Vec<Train>,
        ) -> RouteAssignment
{
    let route = routes.remove(0);

    if trains.len() == 0
    {
        return RouteAssignment::NoMoreTrains;
    }

    if let Some(train) = smallest_train_of_at_least(trains, route.num_cities)
    {
        // RR has a train that can run this route
        return RouteAssignment::Assigned {
            revenue: route.revenue,
            train: train,
        }
    }
    return RouteAssignment::NoUsableTrain {
            route_length: route.num_cities,
    };
}

pub fn find_path(
    game_state: & GameState,
    hexes: &mut Query<(&mut Sprite, &mut MapTile)>,
    start: &Hex,
    end: &Hex,
) -> Option<Vec<Hex>>
{
    // The a_star cost function receives raw hex coordinates.
    // We resolve each to its tile entity through the GameState
    // indices, read the `MapTile` components via the query,
    // and defer to `MapTile::route_cost`.
    //
    // a_star probes `cost(start, start)` and `cost(end, end)`
    // to decide whether the endpoints participate at all,
    // so a tile must report a finite cost to itself
    // `a == b` to `Some(0)`; all other pairs consult the
    // connectivity table.
    //
    // Connectivity is a pair-wise concept: a must have connectivity
    // to b and vice versa, otherwise the pair have no connectivity.
    //
    let cost = |a: Hex, b: Hex| -> Option<u32> {
        if a == b {
            return Some(0);
        }
        let start_entity = *game_state.tile_by_coord.get(&a)?;
        let end_entity = *game_state.tile_by_coord.get(&b)?;
        let start_hex = hexes.get(start_entity).ok()?;
        let end_hex = hexes.get(end_entity).ok()?;

        start_hex.1.route_cost(end_hex.1)
    };

    a_star(*start, *end, cost)
}

// Determine the best (highest revenue) train routes that the
// currently-operating railroad can provide on the current map,
// and record the resulting revenue in the GameState.

pub fn run_trains(game_state: & mut GameState,
            mut hexes: &mut Query<(&mut Sprite, &mut MapTile)> )
{
    let Some(mut current_railroad) =
        game_state.operating_state.order.get(game_state.operating_state.cur_rr)
    else {      
        return; 
    };      
            
    let rr_idx = current_railroad.rr_idx;
    let railroad = &game_state.railroads[rr_idx];

    // a special case: the railroad has no trains at this time
    if railroad.assets.trains.len() == 0
    {
        game_state.operating_state.operating_revenue = 0;
        info!("This railroad has no trains, so it produces no revenue");
        return;
    }

    //
    // I think I want this to be something like:
    // - generate all possible routes on the current map
    // - discard any route which violates the various rules and constraints,
    //   so we only have the routes which this railroad could run
    // - compute the revenue of each route,
    // - pick the combination of routes which gives the highest revenue.
    //

    let mut routes : Vec<Route> = Vec::new();
    let mut trains : Vec<Train> = railroad.assets.trains.clone();

    trains.sort_by(|a, b| { a.range().cmp(&b.range()) });

    let num_cities : usize = game_state.cities.len();
    let city_hexes: Vec<&String> = game_state.cities.keys().collect();

    let mut start_city : usize = 0;
    while start_city < num_cities
    {
        let start_hex_name = city_hexes[start_city].clone();
        let Some(&start_hex_entity) =
                    game_state.tile_by_name.get(start_hex_name.as_str()) else {
            info!("No tile named {}", start_hex_name);
            return;
        };  
        let Ok((_, start_map_tile)) = hexes.get(start_hex_entity) else {
            info!("Map tile entity for {} missing", start_hex_name);
            return;
        };      
        let start_coord = start_map_tile.coord.clone();

        let mut end_city : usize = start_city + 1;

        while end_city < num_cities
        {
            let end_hex_name = city_hexes[end_city].clone();
            let Some(&end_hex_entity) =
                    game_state.tile_by_name.get(end_hex_name.as_str()) else {
                info!("No tile named {}", end_hex_name);
                return;
            };  
            let Ok((_, end_map_tile)) = hexes.get(end_hex_entity) else {
                info!("Map tile entity for {} missing", end_hex_name);
                return;
            };      
            let end_coord = end_map_tile.coord.clone();

            if let Some(path) = find_path( game_state, &mut hexes,
                            &start_coord, &end_coord)
            {
                info!("path from {} to {} is {:?}",
                            start_hex_name, end_hex_name,
                            path);

                if let Some(route) = form_route( game_state, &mut hexes, path)
                {
                    routes.push(route);
                }
            }
            end_city += 1;
        }
        start_city += 1;
    }

    info!("Found {} routes", routes.len());
 
    if routes.len() == 0
    {
        game_state.operating_state.operating_revenue = 0;
        info!("This railroad has no routes, so it produces no revenue");
        return;
    }
    // Sort the routes in descending order by revenue
    routes.sort_unstable_by_key(|r| std::cmp::Reverse(r.revenue));

    // Choose the highest-valued routes, until we run out of trains or routes
    let mut route_selection_complete: bool = false;
    while routes.len() > 0
    {
        if route_selection_complete
        {
            break;
        }
        let result = assign_route_to_best_train(& mut routes, &mut trains);

        match result
        {
            RouteAssignment::Assigned { revenue, train } => {
                game_state.operating_state.operating_revenue += revenue;
                info!("Successfully used route of revenue {} with train {:?}",
                        revenue, train);
            },
            RouteAssignment::NoMoreTrains => {
                info!("All trains have been used for routes");
                route_selection_complete = true;
            },
            RouteAssignment::NoUsableTrain { route_length } => {
                info!("No train available for route of length {}",
                        route_length);
            },
        }
    }

    if route_selection_complete
    {
        info!("run_trains complete, successfully used all trains, revenue {}",
                game_state.operating_state.operating_revenue);
    }
    else
    {
        info!("run_trains complete, some trains could not be used");
    }
}

/// System to perform some simple routefinding tests on the empty
/// map present at startup. Since this leaves the game map modified,
/// it will be removed once the routefinding module is stabilized.
pub fn do_simple_routefinding_tests(
    mut game_state: ResMut<GameState>,
    mut hexes: Query<(&mut Sprite, &mut MapTile)>,
    mut inventory: Query<&mut TileInventoryQuantity>,
    placement: Query<(&TilePlacementData,&TileTrack)>,
    asset_server: Res<AssetServer>,
) {
    // On an empty map, there should be no path from A9 to B10.
    let A9 = Hex::new(1, -5);
    let B10 = Hex::new(1, -4);
    let C15 = Hex::new(3, -3);
    let D10 = Hex::new(0, -2);
    let D14 = Hex::new(2, -2);
    let E9 = Hex::new(1, -1);
    let E11 = Hex::new(0, -1);

    let path = find_path( &mut game_state, &mut hexes, &A9, &B10);

    info!("Empty map path from A9 to B10 is: {:?}", path);

    // Placing a tile on B10 should now allow a_star to find a path
    // from A9 to B10

    game_state.track_state.hex_chosen = true;
    game_state.track_state.hex_name = "B10".to_string();
    game_state.track_state.tile_chosen = 57;
    game_state.track_state.tile_rotation = 0;

    place_tile_impl( &mut game_state, &mut hexes, &mut inventory,
        &placement, &asset_server,
    );

    let path_with_tile = find_path( &mut game_state, &mut hexes, &A9, &B10);

    info!("With tile on B10, path from A9 to B10 is: {:?}", path_with_tile);

    let incomplete_path = find_path( &mut game_state, &mut hexes, &A9, &D14);

    info!("Can we get from A9 to D14 (no): {:?}", incomplete_path);

    // Now let's lay/upgrade tile all the way from B10 to D14:
    // B10 => C9 => D10 => D12.
    // Then we'll see what paths we find.

    game_state.track_state.hex_chosen = true;
    game_state.track_state.hex_name = "B10".to_string();
    game_state.track_state.tile_chosen = 14;
    game_state.track_state.tile_rotation = 1;

    place_tile_impl( &mut game_state, &mut hexes, &mut inventory,
        &placement, &asset_server,
    );

    game_state.track_state.hex_chosen = true;
    game_state.track_state.hex_name = "C9".to_string();
    game_state.track_state.tile_chosen = 8;
    game_state.track_state.tile_rotation = 1;

    place_tile_impl( &mut game_state, &mut hexes, &mut inventory,
        &placement, &asset_server,
    );

    game_state.track_state.hex_chosen = true;
    game_state.track_state.hex_name = "D10".to_string();
    game_state.track_state.tile_chosen = 65;
    game_state.track_state.tile_rotation = 0;

    place_tile_impl( &mut game_state, &mut hexes, &mut inventory,
        &placement, &asset_server,
    );

    game_state.track_state.hex_chosen = true;
    game_state.track_state.hex_name = "D12".to_string();
    game_state.track_state.tile_chosen = 9;
    game_state.track_state.tile_rotation = 2;

    place_tile_impl( &mut game_state, &mut hexes, &mut inventory,
        &placement, &asset_server,
    );

    // We should have valid paths from A9 to: B10, D10, D14, and E9 but NOT
    // to E11 or C15.
    //
    // The paths to E9 are incorrect because the current algorithm doesn't
    // understand the intricacies of tile 65, but that's not a problem
    // with the a_star costing, that's a problem with the 1830-specific
    // special cases.
    //
    // And it shouldn't matter whether we start at A9 or at D14, we
    // should see the same paths in either direction.

    info!("Path from A9 to B10: {:?}",
            find_path( &mut game_state, &mut hexes, &A9, &B10));
    info!("Path from A9 to D10: {:?}",
            find_path( &mut game_state, &mut hexes, &A9, &D10));
    info!("Path from A9 to D14: {:?}",
            find_path( &mut game_state, &mut hexes, &A9, &D14));
    info!("Path from A9 to E9: {:?}",
            find_path( &mut game_state, &mut hexes, &A9, &E9));

    info!("Path from A9 to E11 (expect None): {:?}",
            find_path( &mut game_state, &mut hexes, &A9, &E11));
    info!("Path from A9 to C15 (expect None): {:?}",
            find_path( &mut game_state, &mut hexes, &A9, &C15));

    info!("Path from D14 to D10: {:?}",
            find_path( &mut game_state, &mut hexes, &D14, &D10));
    info!("Path from D14 to B10: {:?}",
            find_path( &mut game_state, &mut hexes, &D14, &B10));
    info!("Path from D14 to E9: {:?}",
            find_path( &mut game_state, &mut hexes, &D14, &E9));
    info!("Path from D14 to A9: {:?}",
            find_path( &mut game_state, &mut hexes, &D14, &A9));

    info!("Path from D14 to E11 (expect None): {:?}",
            find_path( &mut game_state, &mut hexes, &A9, &E11));
    info!("Path from D14 to C15 (expect None): {:?}",
            find_path( &mut game_state, &mut hexes, &A9, &C15));
}
