// Game Model Module for 1830 Game
// This module contains all the components, resources, and systems for the 1830 game model

use bevy::prelude::*;

// ============================================================================
// COMPONENTS - Data attached to entities
// ============================================================================

// 1830 is a railroad investment and building game. You and the
// other players are the stockholders of railroad corporations. Each
// corporation is controlled by its leading stock holder—its president.
// You expand your railroads and generate revenue by building track
// on the map, buying trains, and operating those trains. 

/// Marks an entity as a player in the game
#[derive(Component)]
pub struct Player {
    pub name: String,
}

// 1830 uses a stock market. You and the other players buy and
// sell shares in the railroad corporations. If you own the most
// shares in a corporation, you are its president and control its
// operations. You earn dividends if you own shares in flourishing
// corporations. If you sell shares in a corporation, the value of the
// shares in that corporation drops. Like the real stock market, you
// try to buy shares in corporations that are rising in value, earn
// dividends while you can, and sell first when your money could
// be better used elsewhere. 

/// Marks an entity as a Railroad Corporation
#[derive(Component)]
pub struct RailroadCorporation {
    pub name: String,
}

// ============================================================================
// MARKER COMPONENTS - Used to tag/identify entities
// ============================================================================

/// Marks the currently active player (whose turn it is)
#[derive(Component)]
pub struct ActivePlayer;

/// Marks a RailroadPresident
#[derive(Component)]
pub struct RailroadPresident;

// ============================================================================
// DATA STRUCTURES
// ============================================================================

/// Represents a PrivateCompany
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PrivateCompany {
    SchuykillValue,
    ChamplainAndStLawrence,
    DelawareAndHudson,
    MohawkAndHudson,
    CamdenAndAmboy,
    BaltimoreAndOhio,
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

// ============================================================================
// RESOURCES - Global game state
// ============================================================================

/// The current state and phase of the game
#[derive(Resource)]
pub struct GameState {
    pub phase: GamePhase,
    pub bank: u32,
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
// SYSTEMS - Game logic functions
// ============================================================================

/// System to advance the game through different phases
pub fn advance_game_phase(
    mut game_state: ResMut<GameState>,
) {
    // TODO: Implement phase advancement logic
    match game_state.phase {
        GamePhase::PurchasePrivateCompanies => {
            game_state.phase = GamePhase::TwoTrains;
            info!("Advanced to TwoTrains phase");
        }
        GamePhase::TwoTrains => {
            game_state.phase = GamePhase::ThreeTrains;
            info!("Advanced to ThreeTrains phase");
        }
        GamePhase::ThreeTrains => {
            game_state.phase = GamePhase::FourTrains;
            info!("Advanced to FourTrains phase");
        }
        GamePhase::FourTrains => {
            game_state.phase = GamePhase::FiveTrains;
            info!("Advanced to FiveTrains phase");
        }
        GamePhase::FiveTrains => {
            game_state.phase = GamePhase::SixTrains;
            info!("Advanced to SixTrains phase");
        }
        GamePhase::SixTrains => {
            game_state.phase = GamePhase::DieselTrains;
            info!("Advanced to DieselTrains phase");
        }
        GamePhase::DieselTrains => {
            game_state.phase = GamePhase::EndGame;
            info!("Advanced to EndGame phase");
        }
        GamePhase::EndGame => {
            info!("Who won?");
        }
    }
}

/// System to initialize game resources
pub fn setup_game(mut commands: Commands) {
    commands.insert_resource(GameState {
        phase: GamePhase::PurchasePrivateCompanies,
        bank: 12000,
    });

    info!("Game initialized");
}

// ============================================================================
// PLUGIN - Bundles all 1830 game functionality
// ============================================================================

/// Plugin that adds all 1830 game systems and resources
pub struct Game1830Plugin;

impl Plugin for Game1830Plugin {
    fn build(&self, app: &mut App) {
        app
            // Setup systems run once at startup
            //.add_systems(Startup, (setup_game, setup_players).chain())

            // Update systems run every frame
            // TODO: Add update systems when needed
            // .add_systems(Update, (advance_game_phase, determine_winner))
            ;
    }
}
