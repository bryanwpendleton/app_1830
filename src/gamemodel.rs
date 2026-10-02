// Game Model Module for 1830 Game
// This module contains components, resources, and systems for the 1830 game

use std::collections::HashMap;

use egui::Ui;

use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPrimaryContextPass, egui};

use hexx::Hex;

use crate::routemap::MapTile;
use crate::routemap::HexName;
use crate::routemap::TileInventoryQuantity;
use crate::routemap::TilePlacementData;
use crate::routemap::TileTrack;
use crate::routemap::TileHasCrossover;
use crate::routemap::TileHasJunction;
use crate::stockmarket::GridBox;
use crate::stockmarket::StockMarketCell;
use crate::stockmarket::ShareValueToken;
use crate::stockmarket::check_if_floated;
use crate::stockmarket::railroad_purchase_options;
use crate::stockmarket::stock_round_pass_impl;
use crate::stockmarket::buy_railroad_impl;
use crate::stockmarket::PurchaseDecision;
use crate::stockmarket::set_operating_order;
use crate::privco::NUM_PRIVATE_COMPANIES;
use crate::privco::PlayerBid;
use crate::privco::PrivateCompanyAuctionSubphase;
use crate::privco::PrivateCompanyState;
use crate::privco::pay_privco_revenue;
use crate::privco::auction_pass_impl;
use crate::privco::buy_pc_impl;
use crate::privco::place_bid_impl;
use crate::privco::minimum_bid_for_pc;

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
//
// We track each player's certificates individually, and
// we also track their ownership of each of the companies.
// So for example if you own:
// - The Delaware & Hudson
// - 40% of the Canadian Pacific, including the presidency
// - 20% of the Pennsylvania
// we track 5 certificates: C_DH,C_PC_CPR,C_CPR,C_CPR,C_PRR,C_PRR
// we track a 4 in corporations[CanadianPacific]
// we track a 2 in corporations[Pennsylvania]
// we track a 1 in private_companies[DelawareAndHudson]
// 

pub struct PlayerAssets {
    pub personal_money: u32,
    pub certificates: Vec<Certificate>,
    pub corporations: [u32;8], // Indexed by Corporation enum
    pub private_companies: [u32;6], // Indexed by PrivateCompany enum
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
/// - the player immediately after the last player that bought or
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

// As with any stock exchange, certificates are used in 1830 to
// represent each player’s ownership in the private companies
// and public railroad corporations. For a private company, a
// single certificate represents 100% ownership in the company.
// For a corporation, a single certificate represents 10% or 20%
// ownership. The president’s certificate in a corporation is two
// shares (20%), but counts as a single certificate.

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Certificate {

    C_SV, // SchuykillValley
    C_CS, // ChamplainAndStLawrence
    C_DH, // DelawareAndHudson
    C_MH, // MohawkAndHudson
    C_CA, // CamdenAndAmboy
    C_BO, // BaltimoreAndOhio

    C_PC_PRR, // Pennsylvania
    C_PRR,
    C_PC_NYC, // NewYorkCentral
    C_NYC,
    C_PC_CPR, // CanadianPacific
    C_CPR,
    C_PC_BnO, // BaltimoreAndOhio
    C_BnO,
    C_PC_CnO, // ChesapeakeAndOhio
    C_CnO,
    C_PC_ERIE, // Erie
    C_ERIE,
    C_PC_NNH, // NewYorkNewHavenAndHartford
    C_NNH,
    C_PC_BnM, // BostonAndMaine
    C_BnM,

    Unknown,
}
impl Certificate
{
    pub fn certificate_limit(num_players: u32) -> usize
    {
        match num_players
        {
            2 => 28,
            3 => 20,
            4 => 16,
            5 => 13,
            6 => 11,
            _ => 0,
        }
    }

    pub fn presidents_certificate(rr_idx: usize) -> Certificate
    {
        const PRR  : usize = Railroad::Pennsylvania as usize;
        const NYC  : usize = Railroad::NewYorkCentral as usize;
        const CPR  : usize = Railroad::CanadianPacific as usize;
        const BNO  : usize = Railroad::BaltimoreAndOhio as usize;
        const CNO  : usize = Railroad::ChesapeakeAndOhio as usize;
        const ERIE : usize = Railroad::Erie as usize;
        const NNH  : usize = Railroad::NewYorkNewHavenAndHartford as usize;
        const BNM  : usize = Railroad::BostonAndMaine as usize;

        match rr_idx
        {
            PRR => Certificate::C_PC_PRR,
            NYC => Certificate::C_PC_NYC,
            CPR => Certificate::C_PC_CPR,
            BNO => Certificate::C_PC_BnO,
            CNO => Certificate::C_PC_CnO,
            ERIE => Certificate::C_PC_ERIE,
            NNH => Certificate::C_PC_NNH,
            BNM => Certificate::C_PC_BnM,
            _ => Certificate::Unknown,
        }
    }

    pub fn certificate(rr_idx: usize) -> Certificate
    {
        const PRR  : usize = Railroad::Pennsylvania as usize;
        const NYC  : usize = Railroad::NewYorkCentral as usize;
        const CPR  : usize = Railroad::CanadianPacific as usize;
        const BNO  : usize = Railroad::BaltimoreAndOhio as usize;
        const CNO  : usize = Railroad::ChesapeakeAndOhio as usize;
        const ERIE : usize = Railroad::Erie as usize;
        const NNH  : usize = Railroad::NewYorkNewHavenAndHartford as usize;
        const BNM  : usize = Railroad::BostonAndMaine as usize;

        match rr_idx
        {
            PRR => Certificate::C_PRR,
            NYC => Certificate::C_NYC,
            CPR => Certificate::C_CPR,
            BNO => Certificate::C_BnO,
            CNO => Certificate::C_CnO,
            ERIE => Certificate::C_ERIE,
            NNH => Certificate::C_NNH,
            BNM => Certificate::C_BnM,
            _ => Certificate::Unknown,
        }
    }

    pub fn private_company_certificate(pc_idx: usize) -> Certificate
    {
        const SV : usize = PrivateCompany::SchuykillValley as usize;
        const CSL : usize = PrivateCompany::ChamplainAndStLawrence as usize;
        const DNH : usize = PrivateCompany::DelawareAndHudson as usize;
        const MNH : usize = PrivateCompany::MohawkAndHudson as usize;
        const CNA : usize = PrivateCompany::CamdenAndAmboy as usize;
        const BNO : usize = PrivateCompany::BaltimoreAndOhio as usize;

        match pc_idx
        {
            SV => Certificate::C_SV,
            CSL => Certificate::C_CS,
            DNH => Certificate::C_DH,
            MNH => Certificate::C_MH,
            CNA => Certificate::C_CA,
            BNO => Certificate::C_BO,
            _ => Certificate::Unknown,
        }
    }
}

// who holds the president's certificate for cur_rr?

pub fn president_of(
            game_state: & GameState,
            players: &mut Query<& mut Player>,
            cur_rr: usize) -> usize
{
    let cert_we_want = match cur_rr
    {
        0 => Certificate::C_PC_PRR,
        1 => Certificate::C_PC_NYC,
        2 => Certificate::C_PC_CPR,
        3 => Certificate::C_PC_BnO,
        4 => Certificate::C_PC_CnO,
        5 => Certificate::C_PC_ERIE,
        6 => Certificate::C_PC_NNH,
        7 => Certificate::C_PC_BnM,
        _ => Certificate::Unknown,
    };

    for player in players
    {
        for cert in player.assets.certificates.clone()
        {
            if cert == cert_we_want
            {
                return player.order as usize;
            }
        }
    }
    info!("Failed to find the present of railroad {}", cur_rr);
    return 0;
}

/// Tracking when a Stock Round is over:
/// - when a player passes, passes is incremented
///   and if passes is num == GameState.num_players, round is over
/// - otherwise last_buy_sell is set to this player
///   and passes is set to zero.
pub struct MarketState {
    pub passes: u32,
    pub last_buy_sell: u32,
    pub current_player_has_bought_stock: bool,
    pub current_player_has_sold_stock: bool,
}

pub struct OperatingRailroad
{
    pub rr_idx: usize,
    pub grid_position: GridBox,
    pub z_order: u32,
}

pub struct OperatingState {
    pub order: Vec<OperatingRailroad>, // floated railroads in operating order
    pub cur_rr: usize, // current position in the 'order' vector
}

// The Railroad Corporations don't have any natural order,
// here we just list them in the order they occur in the rules.

pub const NUM_RAILROADS: usize = 8;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Railroad {
    Pennsylvania = 0,
    NewYorkCentral = 1,
    CanadianPacific = 2,
    BaltimoreAndOhio = 3,
    ChesapeakeAndOhio = 4,
    Erie = 5,
    NewYorkNewHavenAndHartford = 6,
    BostonAndMaine = 7,
}

// Railroad Corporations have assets:
// - station tokens, or station markers
// - money
// - trains
pub struct RailroadAssets
{
    pub stations: u32, // number of not-yet-placed station markers
    pub corporation_money: u32,
    pub trains: Vec<Train>,
    pub share_value_token: ShareValueToken,
}

/// Marks an entity as a Railroad Corporation
#[derive(Component)]
pub struct RailroadCorporation {
    pub name: String,
    pub short_name: String,
    pub par_value: u32,
    pub certificates_remaining: u32,
    pub num_stations: u32,
    pub starting_city: String,
    pub starting_hex: String,
    pub floated: bool,
    pub assets: RailroadAssets,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParValue {
    SixtySeven = 67,
    SeventyOne = 71,
    SeventySix = 76,
    EightyTwo = 82,
    Ninety = 90,
    OneHundred = 100,
} 
impl ParValue
{
    pub fn name(&self) -> &str
    {
        match self
        {
            ParValue::SixtySeven => "67",
            ParValue::SeventyOne => "71",
            ParValue::SeventySix => "76",
            ParValue::EightyTwo => "82",
            ParValue::Ninety => "90",
            ParValue::OneHundred => "100",
        }
    }
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

    UnknownPrivateCompany = 254,
}
impl PrivateCompany
{
    pub fn fromInteger(i: usize) -> PrivateCompany
    {
        match i
        {
            0 => PrivateCompany::SchuykillValley,
            1 => PrivateCompany::ChamplainAndStLawrence,
            2 => PrivateCompany::DelawareAndHudson,
            3 => PrivateCompany::MohawkAndHudson,
            4 => PrivateCompany::CamdenAndAmboy,
            5 => PrivateCompany::BaltimoreAndOhio,
            _ => PrivateCompany::UnknownPrivateCompany,
        }
    }
    pub fn faceValue(pc : PrivateCompany) -> u32
    {
        match pc
        {
            PrivateCompany::SchuykillValley => 20,
            PrivateCompany::ChamplainAndStLawrence => 40,
            PrivateCompany::DelawareAndHudson => 70,
            PrivateCompany::MohawkAndHudson => 110,
            PrivateCompany::CamdenAndAmboy => 160,
            PrivateCompany::BaltimoreAndOhio => 220,
            _ => 999,
        }
    }
    pub fn revenue(i: usize) -> u32
    {
        match i
        {
            0 => 5,  // PrivateCompany::SchuykillValley,
            1 => 10, // PrivateCompany::ChamplainAndStLawrence,
            2 => 15, // PrivateCompany::DelawareAndHudson,
            3 => 20, // PrivateCompany::MohawkAndHudson,
            4 => 25, // PrivateCompany::CamdenAndAmboy,
            5 => 30, // PrivateCompany::BaltimoreAndOhio,
            _ => 0,  //PrivateCompany::UnknownPrivateCompany,
        }
    }

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

// CurrentPCAuction holds the state for resolving an auction for
// a PrivateCompany:
//
// If all of the bidders pass consecutively, the auction ends and
// the high bidder buys the private company using the money
// he originally bid and additional money if necessary
pub struct CurrentPCAuction
{
    pub pc: PrivateCompany,
    pub num_bidders: u32,
    pub num_passes: u32,
    pub current_bidder: u32,
}

// ============================================================================
// RESOURCES - Global game state
// ============================================================================

/// The current state and phase of the game
#[derive(Resource)]
pub struct GameState {
    pub phase: GamePhase,
    pub bank: u32,
    pub num_players: u32, // 2-6
    pub certificate_limit: usize, // varies depending on num_players

    pub round_state: RoundState,

    pub priority_deal_card_holder : Entity,
    pub current_player : Entity,
    // player entity by player_id; there may be fewer than 6 players
    pub player_by_player_id: [Entity;6],

    pub market: HashMap<String, GridBox>,
    pub market_state: MarketState,

    pub private_company_states: [u32;6], // for values see PrivateCompanyState
    pub auction_bids: Vec<PlayerBid>,
    pub auction_state: CurrentPCAuction,
    pub auction_subphase : PrivateCompanyAuctionSubphase,

    pub current_par_value: ParValue ,
    pub railroads: Vec<RailroadCorporation>,

    pub operating_state: OperatingState,

    // tile_by_coord and tile_by_name provide search indices to the MapTile
    // entities.  Each `MapTile` lives in its own entity spawned by
    // `spawn_routemap`. These maps let any system that has `GameState`
    // resolve a hex to its `Entity` -- by axial coordinate or by
    // its `HexName` string.
    //
    pub tile_by_coord: HashMap<Hex, Entity>,
    pub tile_by_name: HashMap<String, Entity>,

    // Index into the track-tile inventory, keyed by `tile_number`.
    // Each entry's entity is a `TileInventoryQuantity` recording
    // how many copies of that tile are available to be placed.
    pub inventory_by_number: HashMap<u32, Entity>,

    pub tile_placement_data_by_number: HashMap<u32, Entity>,

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
            certificate_limit: 0,
            round_state: RoundState::StockRound,
            priority_deal_card_holder : Entity::PLACEHOLDER,
            current_player : Entity::PLACEHOLDER,
            player_by_player_id : [Entity::PLACEHOLDER;6],
            market: HashMap::new(),
            market_state: MarketState {
                passes: 0,
                last_buy_sell: 0,
                current_player_has_bought_stock: false,
                current_player_has_sold_stock: false,
            },
            operating_state: OperatingState {
                order: Vec::new(),
                cur_rr: 0,
            },

            private_company_states: [0;6],
            auction_bids: Vec::new(),
            auction_state: CurrentPCAuction {
                pc: PrivateCompany::UnknownPrivateCompany,
                num_bidders: 0,
                num_passes: 0,
                current_bidder: 0
            },
            auction_subphase: PrivateCompanyAuctionSubphase::AllowBids,

            current_par_value: ParValue::SixtySeven,
            railroads: vec![
                RailroadCorporation {
                    name: "Pennsylvania".to_string(),
                    short_name: "PRR".to_string(),
                    par_value: 0,
                    certificates_remaining: 9,
                    num_stations: 4,
                    starting_city: "Altoona".to_string(),
                    starting_hex: "H12".to_string(),
                    floated: false,
                    assets: RailroadAssets {
                        stations: 0,
                        corporation_money: 0,
                        trains: Vec::new(),
                        share_value_token: ShareValueToken
                            {
                                grid_box: "".to_string(),
                                z_order: 0
                            },
                    },
                },
                RailroadCorporation {
                    name: "New York Central".to_string(),
                    short_name: "NYC".to_string(),
                    par_value: 0,
                    certificates_remaining: 9,
                    num_stations: 4,
                    starting_city: "Albany".to_string(),
                    starting_hex: "E19".to_string(),
                    floated: false,
                    assets: RailroadAssets {
                        stations: 0,
                        corporation_money: 0,
                        trains: Vec::new(),
                        share_value_token: ShareValueToken
                            {
                                grid_box: "".to_string(),
                                z_order: 0
                            },
                    },
                },
                RailroadCorporation {
                    name: "Canadian Pacific".to_string(),
                    short_name: "CPR".to_string(),
                    par_value: 0,
                    certificates_remaining: 9,
                    num_stations: 4,
                    starting_city: "Montreal".to_string(),
                    starting_hex: "A19".to_string(),
                    floated: false,
                    assets: RailroadAssets {
                        stations: 0,
                        corporation_money: 0,
                        trains: Vec::new(),
                        share_value_token: ShareValueToken
                            {
                                grid_box: "".to_string(),
                                z_order: 0
                            },
                    },
                },
                RailroadCorporation {
                    name: "Baltimore & Ohio".to_string(),
                    short_name: "B&O".to_string(),
                    par_value: 0,
                    certificates_remaining: 9,
                    num_stations: 3,
                    starting_city: "Baltimore".to_string(),
                    starting_hex: "I15".to_string(),
                    floated: false,
                    assets: RailroadAssets {
                        stations: 0,
                        corporation_money: 0,
                        trains: Vec::new(),
                        share_value_token: ShareValueToken
                            {
                                grid_box: "".to_string(),
                                z_order: 0
                            },
                    },
                },
                RailroadCorporation {
                    name: "Chesapeake & Ohio".to_string(),
                    short_name: "C&O".to_string(),
                    par_value: 0,
                    certificates_remaining: 9,
                    num_stations: 3,
                    starting_city: "Cleveland".to_string(),
                    starting_hex: "F6".to_string(),
                    floated: false,
                    assets: RailroadAssets {
                        stations: 0,
                        corporation_money: 0,
                        trains: Vec::new(),
                        share_value_token: ShareValueToken
                            {
                                grid_box: "".to_string(),
                                z_order: 0
                            },
                    },
                },
                RailroadCorporation {
                    name: "Erie".to_string(),
                    short_name: "Erie".to_string(),
                    par_value: 0,
                    certificates_remaining: 9,
                    num_stations: 3,
                    starting_city: "Buffalo".to_string(),
                    starting_hex: "E11".to_string(),
                    floated: false,
                    assets: RailroadAssets {
                        stations: 0,
                        corporation_money: 0,
                        trains: Vec::new(),
                        share_value_token: ShareValueToken
                            {
                                grid_box: "".to_string(),
                                z_order: 0
                            },
                    },
                },
                RailroadCorporation {
                    name: "New York, New Haven, & Hartford".to_string(),
                    short_name: "NNH".to_string(),
                    par_value: 0,
                    certificates_remaining: 9,
                    num_stations: 2,
                    starting_city: "New York".to_string(),
                    starting_hex: "G19".to_string(),
                    floated: false,
                    assets: RailroadAssets {
                        stations: 0,
                        corporation_money: 0,
                        trains: Vec::new(),
                        share_value_token: ShareValueToken
                            {
                                grid_box: "".to_string(),
                                z_order: 0
                            },
                    },
                },
                RailroadCorporation {
                    name: "Boston & Maine".to_string(),
                    short_name: "B&M".to_string(),
                    par_value: 0,
                    certificates_remaining: 9,
                    num_stations: 2,
                    starting_city: "Boston".to_string(),
                    starting_hex: "E23".to_string(),
                    floated: false,
                    assets: RailroadAssets {
                        stations: 0,
                        corporation_money: 0,
                        trains: Vec::new(),
                        share_value_token: ShareValueToken
                            {
                                grid_box: "".to_string(),
                                z_order: 0
                            },
                    },
                },
            ],

            tile_by_coord: HashMap::new(),
            tile_by_name: HashMap::new(),
            inventory_by_number: HashMap::new(),
            tile_placement_data_by_number: HashMap::new(),
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
pub fn start_stock_round(mut game_state: ResMut<GameState>,
                    priority_deal: Query<Entity, With<PriorityDealCard>>,
) {
    info!("=== Stock Round starting ===");

    game_state.market_state.passes = 0;
    game_state.market_state.last_buy_sell = 0;

    let player_entity = priority_deal.single();
    if let Ok(current_player) = player_entity
    {
        game_state.current_player = current_player;
    }
    game_state.round_state = RoundState::StockRound;
}

/// Runs once each time an Operating Round begins (on entering
/// [`RoundState::OperatingRound`]).
pub fn start_operating_round(mut game_state: ResMut<GameState>,
    mut players: Query<&mut Player>, )
{
    pay_privco_revenue(& mut game_state, & mut players);

    set_operating_order(& mut game_state, & mut players);

    game_state.round_state = RoundState::OperatingRound;

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

/// Places a track tile on a map hex, maintaining the tile inventory.
///
/// The request is carried in `game_state.tile_string`, formatted as
/// `hex_name:image_path:tile_number` (e.g. `"F12:Map/T57.png:57"`).
///
/// Placement rules enforced here:
///   - The tile to place must have non-zero inventory; otherwise the placement
///     is rejected.
///   - If the target hex already holds a track tile, that tile is lifted and
///     returned to inventory (its quantity is incremented) before the new tile
///     is placed.
///   - Placing the new tile decrements its inventory and records its
///     `tile_number` in the hex's `MapTile::placed_tile`.
pub fn place_tile(
    mut game_state: ResMut<GameState>,
    mut hexes: Query<(&mut Sprite, &mut MapTile)>,
    mut inventory: Query<&mut TileInventoryQuantity>,
    placement: Query<(&TilePlacementData,&TileTrack)>,
    asset_server: Res<AssetServer>,
) {
    // Thin system wrapper: unwrap the system params to plain references and
    // defer to `place_tile_impl`, which holds the actual placement logic so it
    // can be driven directly from tests

    place_tile_impl(
        &mut game_state,
        &mut hexes,
        &mut inventory,
        &placement,
        &asset_server,
    );
}

/// The body of the [`place_tile`] system, factored out so it can be called
/// outside a running schedule -- e.g. from internal tests that build the
/// queries via a `SystemState` over a hand-constructed `World`.
///
/// The parameters mirror the system's, but as plain (mutable) references:
/// `ResMut`/`Res`/`Query` all deref to these, so the system wrapper just passes
/// them through.
///
/// See [`place_tile`] for the request format and placement rules.
pub fn place_tile_impl(
    game_state: &mut GameState,
    hexes: &mut Query<(&mut Sprite, &mut MapTile)>,
    inventory: &mut Query<&mut TileInventoryQuantity>,
    placement: &Query<(&TilePlacementData, &TileTrack)>,
    asset_server: &AssetServer,
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

    // Always consume the request, whether or not it turns out to be valid.
    game_state.tile_string.clear();

    if v.len() != 4 {
        info!("Malformed tile request, expected hex_name:image:tile_number:rotation");
        return;
    }
    let (hex_name, image_path) = (&v[0], &v[1]);
    let Ok(new_number) = v[2].parse::<u32>() else {
        info!("Bad tile_number in tile request: {}", v[2]);
        return;
    };
    let Ok(rotation) = v[3].parse::<i32>() else {
        info!("Bad rotation in tile request: {}", v[3]);
        return;
    };

    // Resolve the target hex and the incoming tile's inventory via the indices.
    let Some(&hex_entity) = game_state.tile_by_name.get(hex_name) else {
        info!("No tile named {}", hex_name);
        return;
    };
    let Some(&new_inv_entity) = game_state.inventory_by_number.get(&new_number) else {
        info!("No inventory for tile number {}", new_number);
        return;
    };
    let Some(&placement_entity) =
                    game_state.tile_placement_data_by_number.get(&new_number) else {
        info!("No placement data for tile number {}", new_number);
        return;
    };

    // Rule 1: the tile to place must be available.
    match inventory.get(new_inv_entity) {
        Ok(q) if q.quantity > 0 => {}
        Ok(_) => {
            info!("Tile {} is out of stock", new_number);
            return;
        }
        Err(_) => {
            info!("Inventory entity for tile {} missing", new_number);
            return;
        }
    }

    let Ok((mut sprite, mut map_tile)) = hexes.get_mut(hex_entity) else {
        info!("Map tile entity for {} missing", hex_name);
        return;
    };

    // Rule 2: if a tile is already here, lift it back into inventory. Its
    // inventory entity is a different one than the incoming tile's (unless it's
    // the same number, in which case the net effect is a no-op we still perform
    // step by step), so resolve and update it independently.
    let old_number = map_tile.placed_tile;
    if old_number != 0 {
        if let Some(&old_inv_entity) = game_state.inventory_by_number.get(&old_number) {
            if let Ok(mut old_q) = inventory.get_mut(old_inv_entity) {
                old_q.quantity += 1;
                info!("Returned tile {} to inventory ({} available)",
                    old_number, old_q.quantity);
            }
        }
    }

// FIXME -- there are other placement rules to enforce

    // Rule 3: take the new tile from inventory and place it.
    if let Ok(mut new_q) = inventory.get_mut(new_inv_entity) {
        new_q.quantity -= 1;
        info!("Took tile {} from inventory ({} remaining)",
            new_number, new_q.quantity);
    }

    // placement data with track pattern and other placement details:
    if let Ok(tile_data) = placement.get(placement_entity)  {
        // info!("loaded placement data for tile number {}", new_number);
        map_tile.track = tile_data.1.rotate_tile_track(rotation);
    } else {
        info!("Unable to find placement data for tile number {}", new_number);
        return;
    };

    map_tile.placed_tile = new_number;
    sprite.image = asset_server.load(image_path.clone());
    info!("Placed tile {} on {} (image {})", new_number, hex_name, image_path);
}

/// Renders the game info panel bound to [`GameState`].
///
/// Runs in the [`EguiPrimaryContextPass`] schedule and draws a right-hand
/// side panel as an overlay on top of the hex map, leaving the map rendering
/// untouched.
pub fn game_state_panel_right(
    mut commands: Commands,
    mut contexts: EguiContexts,
    mut players: Query<&mut Player>,
    mut game_state: ResMut<GameState>,
) -> Result {
    let ctx = contexts.ctx_mut()?.clone();

    // Panels render into a Ui built over the viewport background layer.
    let mut viewport_ui = egui::Ui::new(
        ctx.clone(),
        "game_info_viewport_right".into(),
        egui::UiBuilder::new()
            .layer_id(egui::LayerId::background())
            .max_rect(ctx.viewport_rect()),
    );

    egui::Panel::right("game_info_panel_right")
        .resizable(false)
        .default_size(220.0)
        .show(&mut viewport_ui, |ui| {
            ui.heading("1830");
            ui.separator();

            if game_state.phase == GamePhase::PurchasePrivateCompanies
            {
                build_right_side_privatecompany_ui(
                        &mut commands, ui, &mut players, &mut game_state);
            }
            else if game_state.round_state == RoundState::StockRound
            {
                build_right_side_stock_ui(
                        &mut commands, ui, &mut players, &mut game_state);
            }
            else
            {
                build_right_side_operating_ui(
                        &mut commands, ui, &mut players, &mut game_state);
            }
            // ui.add(egui::TextEdit::singleline(&mut game_state.tile_string));

            ui.separator();
        });

    Ok(())
}

// During this intermediate time where we're using egui rather than a
// more integrated Bevy gui, the left panel is read-only, and contains
// important information that the current player needs to be able to
// make their current game decision.

pub fn build_right_side_privatecompany_ui(
            commands: &mut Commands,
            ui: &mut Ui,
            players: &mut Query<& mut Player>,
            game_state: & mut GameState)
{
    // CURRENT_PLAYER: you can:
    // Button("Buy {PrivateCompany} for NN")
    // Bid on (private company droplist) for (amount) (>= min bid)
    // Button("Pass")

    // Read the current player's display values up front so the shared
    // borrow of `players` ends before we pass `players` mutably to the
    // auction/bid helpers below.
    let Ok(current_player) = players.get(game_state.current_player) else {
        return;
    };
    let current_order = current_player.order;
    let current_name = current_player.name.clone();
    let current_money = current_player.assets.personal_money;

    {
        ui.label(format!("{}: you have {} and may:",
            current_name,
            current_money));

        if ui.button("Pass").clicked()
        {
            auction_pass_impl( commands, game_state,
                                players, current_order);
            return;
        }
        else
        {
            let mut first_unsold: bool = true;
            let mut pc_idx: usize = 0;

            while pc_idx < NUM_PRIVATE_COMPANIES
            {       
                let pc = PrivateCompany::fromInteger(pc_idx);
                let price = PrivateCompany::faceValue(pc);
                let min_bid = minimum_bid_for_pc(game_state, &pc);

                if game_state.private_company_states[pc_idx] ==
                    PrivateCompanyState::Unsold as u32 &&
                    first_unsold
                {   
                    if ui.button(format!("Buy {:?} for {}",
                        pc, price)).clicked()
                    {
                        buy_pc_impl(commands, game_state, players,
                                current_order, pc, price);
                        return;
                    }
                    first_unsold = false;
                }
                if game_state.private_company_states[pc_idx] ==
                    PrivateCompanyState::HasBids as u32 ||
                   ( game_state.private_company_states[pc_idx] ==
                    PrivateCompanyState::Unsold as u32 &&
                        ! first_unsold )
                {
                    if ui.button(format!("Bid at least {} on {:?}",
                                min_bid, pc)).clicked()
                    {
                        place_bid_impl(commands, game_state, players,
                                    current_order, pc, min_bid);
                        return;
                    }           
                }           
                pc_idx += 1;
            }    
        }
    }

}

pub fn build_left_side_privatecompany_ui( ui: &mut Ui,
                                        mut players: Query<&Player>,
                                        game_state: & GameState)
{
    // for the purchase private companies phase, we need to show:
    //
    // - how much money the current player has available.
    //
    // - the unpurchased private companies
    // - what bids have already been made on them, how much, by whom
    // 
    // The details of the above further depend on whether we are
    // currently allowing new bids (AllowBids), or whether we are currently
    // resolving the final bids for a particular PC (ResolveBids)

    if let Ok(current_player) = players.get(game_state.current_player)
    {
        ui.label(format!("{}: you have {}",
            current_player.name,
            current_player.assets.personal_money));
    }

    let mut pc_idx : usize = 0;
    while pc_idx < NUM_PRIVATE_COMPANIES
    {
        ui.label(format!("{:?}: {}",
                    PrivateCompany::fromInteger(pc_idx),
                    PrivateCompanyState::formatState(
                            game_state.private_company_states[pc_idx])));
        pc_idx += 1;
    }

    if game_state.auction_subphase == PrivateCompanyAuctionSubphase::AllowBids
    {
    }
    else
    {
    }
}

pub fn build_left_side_stock_ui( ui: &mut Ui,
                                        players: Query<&Player>,
                                        game_state: & GameState)
{
    let Ok(current_player) = players.get(game_state.current_player) else {
        return;
    };
    let current_order = current_player.order;
    let current_name = current_player.name.clone();
    let current_money = current_player.assets.personal_money;
    let current_num_certs = current_player.assets.certificates.len();

    ui.label(format!("{}: you have {} and {} certificates:",
            current_name,
            current_money,
            current_num_certs));

    for c in current_player.assets.certificates.clone()
    {
        ui.label(format!("{:?}", c));
    }
}


pub fn build_right_side_stock_ui(
            commands: &mut Commands,
            ui: &mut Ui,
            players: &mut Query<& mut Player>,
            game_state: & mut GameState)
{
    // CURRENT_PLAYER: you can:
    // ... (lots of stuff)

    // During your turn in a stock round, you may buy an available
    // certificate from either the initial offering or the bank pool. 

    let Ok(mut current_player) = players.get_mut(game_state.current_player) else {
        return;
    };
    let current_order = current_player.order;
    let current_name = current_player.name.clone();
    let current_money = current_player.assets.personal_money;

    ui.label(format!("{}: you have {} and may:",
            current_name,
            current_money));

    egui::ComboBox::from_label("Set Par Value")
        .selected_text(game_state.current_par_value.name())
        .show_ui(ui, |ui| {
            ui.selectable_value(&mut game_state.current_par_value,
                ParValue::SixtySeven, ParValue::SixtySeven.name());
            ui.selectable_value(&mut game_state.current_par_value,
                ParValue::SeventyOne, ParValue::SeventyOne.name());
            ui.selectable_value(&mut game_state.current_par_value,
                ParValue::SeventySix, ParValue::SeventySix.name());
            ui.selectable_value(&mut game_state.current_par_value,
                ParValue::EightyTwo, ParValue::EightyTwo.name());
            ui.selectable_value(&mut game_state.current_par_value,
                ParValue::Ninety, ParValue::Ninety.name());
            ui.selectable_value(&mut game_state.current_par_value,
                ParValue::OneHundred, ParValue::OneHundred.name());
        });

    let mut rr_idx : usize = 0;
    while rr_idx < NUM_RAILROADS
    {
        let railroad = &game_state.railroads[rr_idx];
        let short_name = railroad.short_name.clone();

        let purchase_options = railroad_purchase_options(
                game_state, &current_player, railroad);

        if purchase_options.canBuyPresCert()
        {
            let price = 2 * game_state.current_par_value as u32;

            if ui.button(format!("Buy {:?} pres cert for {}",
                        short_name, price)).clicked()
            {
                buy_railroad_impl(commands, game_state,
                        &mut current_player,
                        rr_idx,
                        PurchaseDecision::BuyPresCert);
                return;
            }
        }
        if purchase_options.canBuyCert()
        {
            let price = game_state.current_par_value as u32;

            if ui.button(format!("Buy {:?} cert for {}",
                        short_name, price)).clicked()
            {
                buy_railroad_impl(commands, game_state,
                        &mut current_player,
                        rr_idx,
                        PurchaseDecision::BuyCert);
                check_if_floated(commands, game_state, players, rr_idx);
                return;
            }
        }
        rr_idx += 1;
    }
    if ui.button("Pass").clicked()
    {
        stock_round_pass_impl( commands, game_state,
                            players, current_order);
        return;
    }
}

// Each railroad is operated by the player that owns its president’s
// certificate. Each railroad may lay track, earn revenue, and
// purchase trains. In addition, each railroad must decide each
// operating round whether to distribute its earnings as dividends
// among the stockholders (raising the railroad’s share value) or to
// retain its earnings to finance further corporate activities (thus
// lowering its share value).

pub fn build_right_side_operating_ui(
            commands: &mut Commands,
            ui: &mut Ui,
            players: &mut Query<& mut Player>,
            game_state: & mut GameState)
{
    let Ok(mut current_railroad) =
        game_state.operating_state.order.get(game_state.operating_state.cur_rr)
    else {
        return;
    };
    let current_player_idx =
        president_of(game_state, players, game_state.operating_state.cur_rr);
    let current_player_ety = game_state.player_by_player_id[current_player_idx];
    let Ok(current_player) = players.get(current_player_ety) else {
        return;
    };

    let current_order = current_player.order;
    let current_name = current_player.name.clone();
    let current_money = current_player.assets.personal_money;

    ui.label(format!("{}: you have {} and may:",
            current_name,
            current_money));

    egui::ComboBox::from_label("Set Par Value")
        .selected_text(game_state.current_par_value.name())
        .show_ui(ui, |ui| {
            ui.selectable_value(&mut game_state.current_par_value,
                ParValue::SixtySeven, ParValue::SixtySeven.name());
            ui.selectable_value(&mut game_state.current_par_value,
                ParValue::SeventyOne, ParValue::SeventyOne.name());
            ui.selectable_value(&mut game_state.current_par_value,
                ParValue::SeventySix, ParValue::SeventySix.name());
            ui.selectable_value(&mut game_state.current_par_value,
                ParValue::EightyTwo, ParValue::EightyTwo.name());
            ui.selectable_value(&mut game_state.current_par_value,
                ParValue::Ninety, ParValue::Ninety.name());
            ui.selectable_value(&mut game_state.current_par_value,
                ParValue::OneHundred, ParValue::OneHundred.name());
        });

    let mut rr_idx : usize = 0;
    while rr_idx < NUM_RAILROADS
    {
        let railroad = &game_state.railroads[rr_idx];
        let short_name = railroad.short_name.clone();

        let purchase_options = railroad_purchase_options(
                game_state, &current_player, railroad);

        if purchase_options.canBuyPresCert()
        {
            let price = 2 * game_state.current_par_value as u32;

            if ui.button(format!("Buy {:?} pres cert for {}",
                        short_name, price)).clicked()
            {
                buy_railroad_impl(commands, game_state,
                        &mut current_player,
                        rr_idx,
                        PurchaseDecision::BuyPresCert);
                return;
            }
        }
        if purchase_options.canBuyCert()
        {
            let price = game_state.current_par_value as u32;

            if ui.button(format!("Buy {:?} cert for {}",
                        short_name, price)).clicked()
            {
                buy_railroad_impl(commands, game_state,
                        &mut current_player,
                        rr_idx,
                        PurchaseDecision::BuyCert);
                check_if_floated(commands, game_state, players, rr_idx);
                return;
            }
        }
        rr_idx += 1;
    }
    if ui.button("Pass").clicked()
    {
        stock_round_pass_impl( commands, game_state,
                            players, current_order);
        return;
    }
}


pub fn game_state_panel_left(
    mut contexts: EguiContexts,
    players: Query<&Player>,
    mut game_state: ResMut<GameState>,
) -> Result {
    let ctx = contexts.ctx_mut()?.clone();

    // Panels render into a Ui built over the viewport background layer.
    let mut viewport_ui = egui::Ui::new(
        ctx.clone(),
        "game_info_viewport_left".into(),
        egui::UiBuilder::new()
            .layer_id(egui::LayerId::background())
            .max_rect(ctx.viewport_rect()),
    );

    egui::Panel::left("game_info_panel_left")
        .resizable(false)
        .default_size(220.0)
        .show(&mut viewport_ui, |ui| {
            ui.heading("1830");
            ui.separator();

            ui.label(format!("Phase: {}", game_state.phase_label()));
            ui.label(format!("Bank: ${}", game_state.bank));
            ui.separator();

            if game_state.phase == GamePhase::PurchasePrivateCompanies
            {
                build_left_side_privatecompany_ui( ui, players, &game_state);
            }
            else
            {
                // assume we're in a stock round for now.
                build_left_side_stock_ui(
                        ui, players, & game_state);
            }

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
    game_state.certificate_limit = Certificate::certificate_limit(num_players);

    for name in names {
        let nmclone = name.clone();
        let player_entity =
            commands.spawn(Player {
                name,
                order: player_order,
                assets: PlayerAssets {
                    personal_money: starting_money,
                    certificates : Vec::new(),
                    corporations: [0; 8],
                    private_companies: [0; 6],
                },
            }).id();
        game_state.player_by_player_id[player_order as usize] = player_entity;
        if player_order == 0
        {
            game_state.priority_deal_card_holder = player_entity;
            commands.entity(player_entity).insert(PriorityDealCard);
            game_state.current_player = player_entity;
            info!("Player {} will have the PriorityDealCard to start",
                    nmclone);
        }
        player_order += 1;
    }
}

/// System to initialize a dummy game with some players
pub fn setup_dummy_players(mut commands: Commands,
                    mut game_state: ResMut<GameState>)
{
    info!("Perhaps we don't need this anymore?");

    create_players( &mut commands,
            &mut game_state,
            vec!["Bryan".into(), "Dan".into()]);
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
            .add_systems(Startup, setup_game)

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
            .add_systems(EguiPrimaryContextPass, 
                (game_state_panel_left, game_state_panel_right) )

        // Update systems run every frame
        // TODO: Add update systems when needed
        // .add_systems(Update, (advance_game_phase, determine_winner))

            .add_systems(Update, place_tile) ;
    }
}
