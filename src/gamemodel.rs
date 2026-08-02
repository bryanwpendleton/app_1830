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
    pub tile_string : String,
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

/*
    pub fn place_tile(&mut self, hex: &str, color:&str, tile:&str, facing:&str) 
    {
        info!("You asked to place {} tile {} facing {} on hex {}",
                color, tile, facing, hex);
    }
*/

    pub fn place_tile(&mut self) 
    {
        info!("You asked to place a tile : {}", self.tile_string);
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

            if ui.button("Advance Phase").clicked() {
                let msg = game_state.advance_phase();
                info!("{}", msg);
            }

            ui.separator();

            let response =
                ui.add(egui::TextEdit::singleline(&mut game_state.tile_string));
            if response.lost_focus()
            {
                game_state.place_tile();
            }
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
            let mut selected_hex = 0;

            bevy_egui::egui::ComboBox::from_label("Map Hex")
                .selected_text(pickable_map_hexes[selected_hex])
                .show_ui(ui, |ui| {
                    for (index, option) in
                            pickable_map_hexes.iter().enumerate()
                    {
                        ui.selectable_value(&mut selected_hex,
                                            index, *option);
                    }
                });

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

            let mut selected_tile = 0;

            bevy_egui::egui::ComboBox::from_label("Tile")
                .selected_text(pickable_tiles[selected_tile])
                .show_ui(ui, |ui| {
                    for (index, option) in
                            pickable_tiles.iter().enumerate()
                    {
                        ui.selectable_value(&mut selected_tile,
                                            index, *option);
                    }
                });

            let pickable_facings =
                vec!["none", "f2", "f3", "f4", "f5", "f6"];
            let mut selected_facing = 0;

            bevy_egui::egui::ComboBox::from_label("Facing")
                .selected_text(pickable_facings[selected_facing])
                .show_ui(ui, |ui| {
                    for (index, option) in
                            pickable_facings.iter().enumerate()
                    {
                        ui.selectable_value(&mut selected_facing,
                                            index, *option);
                    }
                });

            if ui.button("Place tile").clicked() {
                let mut color : &str = "unknown";
                if selected_tile < pickable_yellow_tiles.len()
                {
                    color = "yellow";
                }
                else if selected_tile <
                    pickable_green_tiles.len() + pickable_yellow_tiles.len()
                {
                    color = "green";
                }
                else
                {
                    color = "orange";
                }

                game_state.place_tile(
                            pickable_map_hexes[selected_hex],
                            color,
                            pickable_tiles[selected_tile],
                            pickable_facings[selected_facing]);
            }
*/

            ui.separator();
        });

    Ok(())
}

/// System to initialize game resources
pub fn setup_game(mut commands: Commands) {
    commands.insert_resource(GameState {
        phase: GamePhase::PurchasePrivateCompanies,
        bank: 12000,
        tile_string : String::new(),
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
