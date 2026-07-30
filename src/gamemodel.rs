// Game Model Module for 1830 Game
// This module contains all the components, resources, and systems for the 1830 game model

use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPrimaryContextPass, egui};

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

impl GameState {
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
// SYSTEMS - Game logic functions
// ============================================================================

/// System to advance the game through different phases
pub fn advance_game_phase(
    mut game_state: ResMut<GameState>,
) {
    let msg = game_state.advance_phase();
    info!("{}", msg);
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

            ui.separator();

            if ui.button("Advance Phase").clicked() {
                let msg = game_state.advance_phase();
                info!("{}", msg);
            }
        });

    Ok(())
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
            .add_systems(Startup, setup_game)

            // egui UI systems must run in the EguiPrimaryContextPass schedule
            // so the primary context is available.
            .add_systems(EguiPrimaryContextPass, game_state_panel);

        // Update systems run every frame
        // TODO: Add update systems when needed
        // .add_systems(Update, (advance_game_phase, determine_winner))
    }
}
