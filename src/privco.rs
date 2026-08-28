use bevy::prelude::*;
use bevy_egui::EguiContexts;

use crate::gamemodel::GameState;
use crate::gamemodel::Player;
use crate::gamemodel::PrivateCompany;

use crate::gamemodel::create_players;

// Private Companies behaviors for 1830.
//
// Private companies may be owned by either a player or a railroad.
// Initially they are sold to the players by auction, and subsequently
// they can be resold. All the private companies must be sold at the
// start of the game before anything else happens. This is the special
// phase GamePhase::PurchasePrivateCompanies.
//
// While they are open, they pay their own some revenue for each
// operating round. Once closed, certain events may be triggered.
//
// The initial auction is a complete and separate mini-game that runs
// once at the start of 1830, and its implementation is here.
//
// During GamePhase::ThreeTrains and GamePhase::FourTrains, a railroad
// may buy a private company at any time during its turn in an
// operating round. In addition, ownership of private companies
// may make possible activities otherwise not allowed. They may
// also be sold back and forth between players. Private companies
// are closed when GamePhase::FiveTrains starts.
//
// Each private company that you own counts against your total
// certificate limit.
//
// At the beginning of each operating round, each not-yet-closed
// private comany pays revenue to its own.
//
// Each private company has a special effect on game play (until it
// is closed).

pub struct PlayerBid
{
    pub order: u32, // who placed this bid, identified by play order
    pub private_company: PrivateCompany,
    pub bid_amount: u32,
}

// During the PurchasePrivateCompanies phase, there are two separate
// sub-phases that occur, depending on whether the private company
// with the lowest face value has a bid on it.
//
// Here, I give these two phases the names:
//
// - AllowBids
// - ResolveBids
//
// To be able to participate in the ResolveBids phase for a particular
// private company, you must first have made a bid for that private
// company during the AllowBids phase. 
//

pub enum AllowBidsAction
{
    Pass,
    BuyLowest,
    Bid,
}

pub enum ResolveBidsAction
{
    AutoPass, // this player didn't have a bid on this private company
    RaiseBid,
    Pass,
}

// A bid for a private company must exceed the face value of the
// company (or of any other bid already made for it) by at least $5
// and a multiple of $1. The player must place the bid money in
// front of him on the table and not use it for any other purpose
// until ownership of the company is resolved. 

pub fn place_bid_impl(
    commands: &mut Commands,
    game_state: &mut GameState,
    players: &mut Query<&mut Player>,
    player_id: u32,
    pc: PrivateCompany,
    amount: u32 )
{
    // todo: make sure the new bid exceeds any other for this PC

    // todo: make sure the new bid exceeds the face value of the PC

    // reduce player.personal_money by amount
    for mut player in players.iter_mut()
    {
        if player.order == player_id
        {
            player.assets.personal_money -= amount;
        }
    }

    game_state.auction_bids.push(
        PlayerBid {
            order: player_id,
            private_company: pc,
            bid_amount: amount,
        }
    );
}

// During the final auction for a PC, you can raise your bid.

pub fn raise_bid_impl(
    commands: &mut Commands,
    game_state: &mut GameState,
    players: &mut Query<&mut Player>,
    player_id: u32,
    pc: PrivateCompany,
    amount: u32 )
{
    // todo: make sure the new bid exceeds any other for this PC

    for mut bid in game_state.auction_bids.iter_mut()
    {
        if bid.order == player_id
        {
            let increase = amount - bid.bid_amount;
            bid.bid_amount = amount;

            for mut player in players.iter_mut()
            {
                if player.order == player_id
                {
                    player.assets.personal_money -= increase;
                }
            }
            info!("Adjusted existing bid to {}, an increase of {}",
                amount, increase);
        }
    }

    game_state.auction_bids.push(
        PlayerBid {
            order: player_id,
            private_company: pc,
            bid_amount: amount,
        }
    );
}

// Pay face value to buy the unsold private company that has
// the lowest face value. The player to your left gets the
// priority deal card.

pub fn buy_pc_impl(
    commands: &mut Commands,
    game_state: &mut GameState,
    players: &mut Query<&mut Player>,
    player_id: u32,
    pc: PrivateCompany,
    amount: u32 )
{
    for mut player in players.iter_mut()
    {
        if player.order == player_id
        {
            player.assets.personal_money -= amount;
            player.assets.private_companies[pc as usize] = 1;
        }
        if player.order == (player_id + 1) % game_state.num_players
        {
            info!("Don't know how to re-assign Priority Deal Card");
        }
    }
}

pub fn auction_pass_impl(
    commands: &mut Commands,
    game_state: &mut GameState,
    players: &mut Query<&mut Player>,
    player_id: u32,
) {
    info!("Auction pass not implemented yet:");
}

pub fn resolve_pass_impl(
    commands: &mut Commands,
    game_state: &mut GameState,
    players: &mut Query<&mut Player>,
    player_id: u32,
) {
    info!("Resolve pass not implemented yet:");
}

/// System to perform a simple private company auction test at startup.
///
/// It will be removed once the privco module is stabilized.
pub fn do_simple_auction_tests(
    mut commands: Commands,
    mut game_state: ResMut<GameState>,
    mut players: &mut Query<&mut Player>,
) {   

    // Gerald has the priority at the start of a 4 player game
    // with Dave, Bruce, and Alex to his left in that order.

    create_players( &mut commands,
            &mut game_state,
            vec!["Gerald".into(), "Dave".into(),
                "Bruce".into(), "Alex".into()]);

    // Gerald places a bid of $165 on the CA. Dave bids $225 for the
    // BO, Bruce bids $75 for the DH, and Alex bids $170 for the CA. 

    place_bid_impl(&mut commands, &mut game_state, &mut players,
                    0, PrivateCompany::CamdenAndAmboy, 165);
    place_bid_impl(&mut commands, &mut game_state, &mut players,
                    1, PrivateCompany::BaltimoreAndOhio, 225);
    place_bid_impl(&mut commands, &mut game_state, &mut players,
                    2, PrivateCompany::DelawareAndHudson, 75);
    place_bid_impl(&mut commands, &mut game_state, &mut players,
                    3, PrivateCompany::CamdenAndAmboy, 170);

    // Coming back to Gerald, he bids $80 for the
    // DH. Dave then buys the SV for $20 and the
    // priority deal card goes to Bruce.

    place_bid_impl(&mut commands, &mut game_state, &mut players,
                    0, PrivateCompany::DelawareAndHudson, 80);

    buy_pc_impl(&mut commands, &mut game_state, &mut players,
                    1, PrivateCompany::SchuykillValley, 20);

    // Bruce passes.
    auction_pass_impl(&mut commands, &mut game_state, &mut players, 2);

    // Alex buys the CL for $40 and the priority deal card goes
    // back to Gerald
    buy_pc_impl(&mut commands, &mut game_state, &mut players,
                    3, PrivateCompany::ChamplainAndStLawrence, 40);

    // The bid-buying pauses so that Bruce and Gerald’s bids
    // on the DH can be resolved.  Bruce’s original bid of $75
    // is the lowest, so he bids first. He bids $85. 

    raise_bid_impl(&mut commands, &mut game_state, &mut players,
                    2, PrivateCompany::DelawareAndHudson, 85);

    // Gerald bids $90.  Bruce bids $95

    raise_bid_impl(&mut commands, &mut game_state, &mut players,
                    0, PrivateCompany::DelawareAndHudson, 90);
    raise_bid_impl(&mut commands, &mut game_state, &mut players,
                    2, PrivateCompany::DelawareAndHudson, 95);

    // Gerald decides that is too rich for him and passes.
    // Bruce pays his $95 to the bank and takes the DH certificate.

    resolve_pass_impl(&mut commands, &mut game_state, &mut players, 0);

    // Since Alex was the last player to bid-buy, Gerald is
    // the next to bid-buy. He buys the MH for $110 and the
    // priority deal card goes to Dave. 
    buy_pc_impl(&mut commands, &mut game_state, &mut players,
                    0, PrivateCompany::MohawkAndHudson, 110);

    // The bid-buying pauses so that Gerald and Alex’s bids on the
    // CA can be resolved. Gerald bids $175, and Alex jumps to $200
    // to speed things up. Gerald calculates and goes to $207.
    // Alex bids $212. Gerald counts his money and passes. Alex pays
    // his $212 and takes the CA and the free PRR certifiate
    // that goes with it—a bargain.
    raise_bid_impl(&mut commands, &mut game_state, &mut players,
                    0, PrivateCompany::CamdenAndAmboy, 175);
    raise_bid_impl(&mut commands, &mut game_state, &mut players,
                    3, PrivateCompany::CamdenAndAmboy, 200);
    raise_bid_impl(&mut commands, &mut game_state, &mut players,
                    0, PrivateCompany::CamdenAndAmboy, 207);
    raise_bid_impl(&mut commands, &mut game_state, &mut players,
                    3, PrivateCompany::CamdenAndAmboy, 212);

    resolve_pass_impl(&mut commands, &mut game_state, &mut players, 0);

    // There is only one bid on the BO, so Dave pays his $225
    // and takes the BO. 

    // The end results are:
    // - Gerald: $490, MH
    // - Dave: $355, BO, SV, Priority Deal card, B&O President’s certificate
    // - Bruce: $505, DH,
    // - Alex: $348, CA, CL, 1 PRR share
}
