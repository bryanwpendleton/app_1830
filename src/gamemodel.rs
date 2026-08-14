// Game Model Module for 1830 Game
// This module contains components, resources, and systems for the 1830 game

use std::collections::HashMap;

use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPrimaryContextPass, egui};

use hexx::Hex;

use crate::routemap::MapTile;
use crate::routemap::HexName;
use crate::stockmarket::GridBox;
use crate::stockmarket::StockMarketCell;

// ============================================================================
// COMPONENTS - Data attached to entities
// ============================================================================

// 1830 is a railroad investment and building game. You and the
// other players are the stockholders of railroad corporations. Each
// corporation is controlled by its leading stock holder—its president.
// You expand your railroads and generate revenue by building track
// on the map, buying trains, and operating those trains. 
//
// Each player has assets:
// - personal money
// - shares of railroad corporations
// - private companies (usually closed by end of game)

pub struct PlayerAssets {
    personal_money: u32,
    corporations: [u32;8], // Indexed by Corporation enum
    private_companies: [u32;6], // Indexed by PrivateCompany enum
}

/// Marks an entity as a player in the game
///
/// For the proper operation of the Stock Rounds, we have to
/// keep track of the Players so that we satisfy the following:
///
/// - players take turns in order, the order is decided when
///   when the game starts and doesn't change. Each player
///   knows his/her order number, and the total number of
///   players is global GameState.
/// - the current player is the one taking a turn, then the
///   next player in order gets to take a turn.
/// - when all players have consecutively passed, the current
///   round ends. The number of consecutive passes is global
///   GameState, incremented when the current player passes
///   and reset to zero when a play buys or sells
/// - the player immediatel after the last player that bought or
///   sold a certificate is given the priority deal card, indicating
///   that player takes the first turn in the next stock round.
///
#[derive(Component)]
pub struct Player {
    pub name: String,
    pub order: u32, // next player is (order + 1) modulo num_players
    pub assets: PlayerAssets,
}

/// Marks the currently active player (whose turn it is)
#[derive(Component)]
pub struct CurrentPlayer;

/// Marks the Player who starts the next Stock Round
#[derive(Component)]
pub struct PriorityDealCard;

// 1830 uses a stock market. You and the other players buy and
// sell shares in the railroad corporations. If you own the most
// shares in a corporation, you are its president and control its
// operations. You earn dividends if you own shares in flourishing
// corporations. If you sell shares in a corporation, the value of the
// shares in that corporation drops. Like the real stock market, you
// try to buy shares in corporations that are rising in value, earn
// dividends while you can, and sell first when your money could
// be better used elsewhere. 

/// Tracking when a Stock Round is over:
/// - when a player passes, passes is incremented
///   and if passes is num == GameState.num_players, round is over
/// - otherwise last_buy_sell is set to this player
///   and passes is set to zero.
pub struct MarketState {
    pub passes: u32,
    pub last_buy_sell: u32,
}

/// Marks an entity as a Railroad Corporation
#[derive(Component)]
pub struct RailroadCorporation {
    pub name: String,
}

/// Marks a RailroadPresident
#[derive(Component)]
pub struct RailroadPresident;

/// Represents a PrivateCompany
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PrivateCompany {
    SchuykillValley = 0,
    ChamplainAndStLawrence = 1,
    DelawareAndHudson = 2,
    MohawkAndHudson = 3,
    CamdenAndAmboy = 4,
    BaltimoreAndOhio = 5,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Train {
    Two = 2,
    Three = 3,
    Four = 4,
    Five = 5,
    Six = 6,
    Diesel = 7,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Corporation {
    Pennsylvania = 0,
    NewYorkCentral = 1,
    CanadianPacific = 2,
    BaltimoreAndOhio = 3,
    ChesapeakeAndOhio = 4,
    Erie = 5,
    NewYorkNewHavenAndHartford = 6,
    BostonAndMaine = 7,
}

// ============================================================================
// RESOURCES - Global game state
// ============================================================================

/// The current state and phase of the game
#[derive(Resource)]
pub struct GameState {
    pub phase: GamePhase,
    pub bank: u32,
    pub num_players: u32,

    pub market: HashMap<String, GridBox>,
    pub market_state: MarketState,

    // pub route_map: HashMap<String, MapTile>,
    // pub route_tiles: HashMap<Hex, MapTile>,

    pub tile_string : String,
}

impl GameState {
    /// The initial game state for a fresh game.
    ///
    /// Inserted at plugin-build time (see [`Game1830Plugin`]) so the resource
    /// exists before the first `RoundState` transition, which Bevy runs once
    /// prior to `PreStartup` -- earlier than any `Startup` system.
    pub fn new() -> Self {
        GameState {
            phase: GamePhase::PurchasePrivateCompanies,
            bank: 12000 - 2400, // 2400 is the initial money for the players.
            num_players: 0,
            market: HashMap::new(),
            market_state: MarketState {
                passes: 0,
                last_buy_sell: 0,
            },
            route_map: HashMap::new(),
            route_tiles: HashMap::new(),
            tile_string: String::new(),
        }
    }

    /// Advance the game to the next phase, returning a log message.
    /// Shared by the `advance_game_phase` system and the UI panel button.
    pub fn advance_phase(&mut self) -> &'static str {
        match self.phase {
            GamePhase::PurchasePrivateCompanies => {
                self.phase = GamePhase::TwoTrains;
                "Advanced to TwoTrains phase"
            }
            GamePhase::TwoTrains => {
                self.phase = GamePhase::ThreeTrains;
                "Advanced to ThreeTrains phase"
            }
            GamePhase::ThreeTrains => {
                self.phase = GamePhase::FourTrains;
                "Advanced to FourTrains phase"
            }
            GamePhase::FourTrains => {
                self.phase = GamePhase::FiveTrains;
                "Advanced to FiveTrains phase"
            }
            GamePhase::FiveTrains => {
                self.phase = GamePhase::SixTrains;
                "Advanced to SixTrains phase"
            }
            GamePhase::SixTrains => {
                self.phase = GamePhase::DieselTrains;
                "Advanced to DieselTrains phase"
            }
            GamePhase::DieselTrains => {
                self.phase = GamePhase::EndGame;
                "Advanced to EndGame phase"
            }
            GamePhase::EndGame => "Who won?",
        }
    }

    /// Human-readable label for the current phase, for display in the UI.
    pub fn phase_label(&self) -> &'static str {
        match self.phase {
            GamePhase::PurchasePrivateCompanies => "Purchase Private Companies",
            GamePhase::TwoTrains => "2-Trains",
            GamePhase::ThreeTrains => "3-Trains",
            GamePhase::FourTrains => "4-Trains",
            GamePhase::FiveTrains => "5-Trains",
            GamePhase::SixTrains => "6-Trains",
            GamePhase::DieselTrains => "Diesel Trains",
            GamePhase::EndGame => "End Game",
        }
    }
}

// The game progresses through seven phases. The start of each
// new phase is triggered by the purchase of a new train type:
// 2-train, 3-train, 4-train, 5-train, 6-train, diesel. Each phase has
// limitations and addtions as follows:

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum GamePhase {
    PurchasePrivateCompanies,
    TwoTrains,
    ThreeTrains,
    FourTrains,
    FiveTrains,
    SixTrains,
    DieselTrains,
    EndGame,
}

// ============================================================================
// ROUNDS - State + SystemSets for the Stock/Operating round cycle
// ============================================================================

// The game runs as a sequence of rounds that alternate between a Stock Round
// (players buy and sell shares) and an Operating Round (corporations lay track,
// run trains, and pay out). Certain one-shot actions happen at the *start* of
// each round.
//
// We model "which round we're in" as a Bevy `States` type.
// On every transition Bevy runs the `OnEnter`/`OnExit` schedules
// for the state, which is where the start-of-round setup belongs.
//
// The `RoundSet` SystemSets then group the *recurring* (per-frame) systems that
// run throughout each round, so they can be ordered and gated together.

/// Which kind of round is currently active. Transitions between the two are
/// driven through `NextState<RoundState>` (see [`advance_round`]).
#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RoundState {
    /// Players buy and sell shares. 1830 always opens with a Stock Round.
    #[default]
    StockRound,
    /// Corporations lay track, run trains, and pay dividends.
    OperatingRound,
}

/// SystemSets for the recurring, per-frame systems belonging to each round.
///
/// Group a round's ongoing systems into the matching set; the set is gated to
/// run only while that round is active (configured in [`Game1830Plugin`]). The
/// one-shot start-of-round work is scheduled in `OnEnter(RoundState::...)`
/// instead, not in these sets.
#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RoundSet {
    StockRound,
    OperatingRound,
}

/// Runs once each time a Stock Round begins (on entering
/// [`RoundState::StockRound`]).
///
/// Resets the pass tracking that determines when the round ends. The priority
/// deal and current-player selection will be wired in here as those systems
/// come online.
pub fn start_stock_round(mut game_state: ResMut<GameState>) {
    game_state.market_state.passes = 0;
    game_state.market_state.last_buy_sell = 0;
    info!("=== Stock Round starting ===");
}

/// Runs once each time an Operating Round begins (on entering
/// [`RoundState::OperatingRound`]).
pub fn start_operating_round() {
    info!("=== Operating Round starting ===");
}

/// Advances to the other round type, triggering that round's `OnEnter` systems.
///
/// This is intentionally *not* scheduled to run every frame (doing so would
/// flip the round each tick). Call it, or schedule it behind a run condition,
/// when the current round has ended.
pub fn advance_round(
    current: Res<State<RoundState>>,
    mut next: ResMut<NextState<RoundState>>,
) {
    let upcoming = match current.get() {
        RoundState::StockRound => RoundState::OperatingRound,
        RoundState::OperatingRound => RoundState::StockRound,
    };
    next.set(upcoming);
}

// ============================================================================
// SYSTEMS - Game logic functions
// ============================================================================

/// System to advance the game through different phases
pub fn advance_game_phase(
    mut game_state: ResMut<GameState>,
) {
    let msg = game_state.advance_phase();
    info!("{}", msg);
}

pub fn place_tile(
    mut game_state: ResMut<GameState>,
    mut hexes: Query<(&mut Sprite, &MapTile)>,
    asset_server: Res<AssetServer>,
) {
    if game_state.tile_string.is_empty()
    {
        return;
    }

    info!("You asked to place a tile : {}", game_state.tile_string);

    let v: Vec<String> = game_state
        .tile_string
        .split(":")
        .map(|s| s.to_string())
        .collect();

    for (mut sprite, map_hex) in &mut hexes {
        if map_hex.hex_name.name == v[0]
        {
            sprite.image = asset_server.load(v[1].clone());
            info!("Updated the image for {} to {}", v[0], v[1]);
        }
    }
    game_state.tile_string.clear();
}

/// Renders the game info panel bound to [`GameState`].
///
/// Runs in the [`EguiPrimaryContextPass`] schedule and draws a right-hand
/// side panel as an overlay on top of the hex map, leaving the map rendering
/// untouched.
pub fn game_state_panel(
    mut contexts: EguiContexts,
    mut game_state: ResMut<GameState>,
) -> Result {
    let ctx = contexts.ctx_mut()?.clone();

    // Panels render into a Ui built over the viewport background layer.
    let mut viewport_ui = egui::Ui::new(
        ctx.clone(),
        "game_info_viewport".into(),
        egui::UiBuilder::new()
            .layer_id(egui::LayerId::background())
            .max_rect(ctx.viewport_rect()),
    );

    egui::Panel::right("game_info_panel")
        .resizable(false)
        .default_size(220.0)
        .show(&mut viewport_ui, |ui| {
            ui.heading("1830");
            ui.separator();

            ui.label(format!("Phase: {}", game_state.phase_label()));
            ui.label(format!("Bank: ${}", game_state.bank));

            if ui.button("Advance Phase").clicked() {
                let msg = game_state.advance_phase();
                info!("{}", msg);
            }

            ui.separator();

            ui.add(egui::TextEdit::singleline(&mut game_state.tile_string));
/*
            let pickable_map_hexes = vec!["B10", "B12", "B14",
                "B16", "B18", "B20", "B22", "C7", "C9", "C11",
                "C13", "C17", "C19", "C21", "C23", "D4", "D6",
                "D8", "D10", "D12", "D16", "D18", "D10", "D22",
                "E3", "E5", "E7", "E11", "E13", "E15", "E17",
                "E19", "E21", "E23", "F4", "F8", "F10", "F12",
                "F14", "F16", "F18", "F20", "F22", "G3", "G5",
                "G7", "G9", "G11", "G13", "G15", "G17", "G19",
                "H2", "H4", "H6", "H8", "H10", "H14", "F16",
                "H18", "H20", "I3", "I5", "I7", "I9", "I11",
                "I13", "I15", "I17", "J4", "J6", "J8", "J10",
                "J12", "J14"];

            let pickable_yellow_tiles =
                vec!["T1", "T2", "T3", "T4", "T5", "T6", "T7",
                    "T8", "T9", "T55", "T56", "T57", "T58", "T69"];
            let pickable_green_tiles =
                vec!["T14", "T15", "T16", "T18", "T19", "T20",
                    "T23", "T24", "T25", "T26", "T27", "T28",
                    "T29", "T53", "T54", "T59"];
            let pickable_orange_tiles =
                vec!["T39", "T40", "T41", "T42", "T43", "T44",
                    "T45", "T46", "T47", "T61", "T62", "T63",
                    "T64", "T65", "T66", "T67", "T68", "T70"];
            let pickable_tiles : Vec<&str> = 
                vec![pickable_yellow_tiles.clone(),
                    pickable_green_tiles.clone(),
                    pickable_orange_tiles.clone()]
                .into_iter().flatten().collect();

            let pickable_facings =
                vec!["none", "f2", "f3", "f4", "f5", "f6"];
            let mut selected_facing = 0;

*/

            ui.separator();
        });

    Ok(())
}

/// System to initialize game resources
pub fn setup_game() {

    info!("Game initialized");
}

/// Spawns a [`Player`] entity for each name in `names`, initializing their
/// [`PlayerAssets`] for the start of a new game.
///
/// The starting bank is split evenly among the players: each begins with
/// `2400 / players` in personal money and holds no corporation shares or
/// private companies yet.
pub fn create_players(commands: &mut Commands,
                    game_state: &mut GameState,
                    names: Vec<String>)
{
    let num_players: u32 = names.len() as u32;
    let starting_money = 2400 / num_players;
    let mut player_order = 0;

    game_state.num_players = num_players;

    for name in names {
        commands.spawn(Player {
            name,
            order: player_order,
            assets: PlayerAssets {
                personal_money: starting_money,
                corporations: [0; 8],
                private_companies: [0; 6],
            },
        });
        player_order += 1;
    }
}

/// System to initialize a dummy game with 3 players
pub fn setup_dummy_players(mut commands: Commands,
                    mut game_state: ResMut<GameState>)
{
    create_players( &mut commands,
            &mut game_state,
            vec!["Bryan".into(), "Dan".into(), "Tay".into()]);
}

// ============================================================================
// PLUGIN - Bundles all 1830 game functionality
// ============================================================================

/// Plugin that adds all 1830 game systems and resources
pub struct Game1830Plugin;

impl Plugin for Game1830Plugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(GameState::new())

            .init_state::<RoundState>()

            // Setup systems run once at startup
            .add_systems(Startup, (setup_game, setup_dummy_players).chain())

            // Start-of-round actions: run once on each transition into the
            // corresponding round, via the OnEnter schedules.
            .add_systems(
                OnEnter(RoundState::StockRound), start_stock_round)
            .add_systems(
                OnEnter(RoundState::OperatingRound), start_operating_round)

            // Gate each round's recurring SystemSet so its members only run
            // while that round is the active state.
            .configure_sets(
                Update,
                (
                    RoundSet::StockRound.run_if(
                        in_state(RoundState::StockRound)),
                    RoundSet::OperatingRound.run_if(
                        in_state(RoundState::OperatingRound)),
                ),
            )

            // egui UI systems must run in the EguiPrimaryContextPass schedule
            // so the primary context is available.
            .add_systems(EguiPrimaryContextPass, game_state_panel)

        // Update systems run every frame
        // TODO: Add update systems when needed
        // .add_systems(Update, (advance_game_phase, determine_winner))

            .add_systems(Update, place_tile) ;
    }
}
