use bevy::prelude::*;
use bevy_egui::EguiContexts;

use crate::gamemodel::GameState;
use crate::gamemodel::Player;
use crate::gamemodel::PrivateCompany;
use crate::gamemodel::PriorityDealCard;

use crate::gamemodel::create_players;

// Private Companies behaviors for 1830.
//
// Private companies may be owned by either a player or a railroad.
// Initially they are sold to the players by auction, and subsequently
// they can be resold. All the private companies must be sold at the
// start of the game before anything else happens. This is the special
// phase GamePhase::PurchasePrivateCompanies.

pub const NUM_PRIVATE_COMPANIES: usize = 6;

/// The OwnedByPlayer and OwnedByRR states also encode *which* player
/// or Railroad owns the PrivateCompany
pub enum PrivateCompanyState
{
    Unsold = 0,
    HasBids = 1,
    OwnedByPlayer = 10, // 10 through 10+(numPlayers-1)
    OwnedByRR = 20,     // 20-27 (numRRs = 8)
    Closed = 30,

    UnknownPCState = 100,
}

impl PrivateCompanyState
{
    pub fn formatState(encoded_state: u32 ) -> String
    {
        if encoded_state == PrivateCompanyState::Unsold as u32
        {
            return "Unsold".to_owned();
        }
        if encoded_state == PrivateCompanyState::HasBids as u32
        {
            return "Has Bids".to_owned();
        }
        if encoded_state >= PrivateCompanyState::OwnedByPlayer as u32 &&
           encoded_state <  PrivateCompanyState::OwnedByPlayer as u32 + 5
        {
            return format!("Owned by player {}", 
               encoded_state - PrivateCompanyState::OwnedByPlayer as u32);
        }
        if encoded_state >= PrivateCompanyState::OwnedByRR as u32 &&
           encoded_state <  PrivateCompanyState::OwnedByRR as u32 + 7
        {
            return format!("Owned by RailRoad {}", 
               encoded_state - PrivateCompanyState::OwnedByRR as u32);
        }
        if encoded_state == PrivateCompanyState::Closed as u32
        {
            return "Closed".to_owned();
        }
        return format!("Unknown private company state {}", encoded_state);
    }
}

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

#[derive(PartialEq)]
pub enum PrivateCompanyAuctionSubphase
{
    AllowBids,
    ResolveBids,
}

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
    game_state.private_company_states[pc as usize] = 
                            PrivateCompanyState::HasBids as u32;
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
        if bid.order == player_id && bid.private_company == pc
        {
            // info!("You are bidding {} on {}, existing bid is {}",
            //             amount, pc as usize, bid.bid_amount);

            let increase = amount - bid.bid_amount;
            bid.bid_amount = amount;

            for mut player in players.iter_mut()
            {
                if player.order == player_id
                {
                    player.assets.personal_money -= increase;
                }
            }
            // info!("Adjusted existing bid to {}, an increase of {}",
            //     amount, increase);
        }
    }
    game_state.auction_state.num_passes = 0;

}

pub fn lowest_unsold_pc( game_state: &mut GameState ) -> PrivateCompany
{
    let mut pc_idx: usize = 0;
    while pc_idx < NUM_PRIVATE_COMPANIES
    {
        if game_state.private_company_states[pc_idx] ==
            PrivateCompanyState::Unsold as u32
        {
            return PrivateCompany::fromInteger(pc_idx);
        }
        pc_idx += 1;
    }
    return PrivateCompany::UnknownPrivateCompany;
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

            game_state.private_company_states[pc as usize] =
                PrivateCompanyState::OwnedByPlayer as u32 + player_id;

            info!("Player {} buys private company {:?} for {}, money now {}",
                player_id, pc, amount, player.assets.personal_money);

            move_priority_deal_card(commands, game_state,
                            (player_id + 1) % game_state.num_players);
        }
    }
}

// Moves the CurrentPlayer component directly from the current holder to
// the specified player_id.
//
pub fn move_priority_deal_card(
    commands: &mut Commands,
    game_state: &mut GameState,
    player_id: u32,
) {
    
    commands.entity(game_state.priority_deal_card_holder)
                    .remove::<PriorityDealCard>();
    commands.entity(game_state.player_by_player_id[player_id as usize])
                    .insert(PriorityDealCard);

    game_state.priority_deal_card_holder =
                    game_state.player_by_player_id[player_id as usize];
}

pub fn auction_pass_impl(
    commands: &mut Commands,
    game_state: &mut GameState,
    players: &mut Query<&mut Player>,
    player_id: u32,
) {
    info!("Auction pass not implemented yet:");
}

// This code assumes there was at least one such bid, and returns
// the player_id of the winning bid and the bid index in the
// master array of bids.

pub fn highest_bid_for_pc(game_state: &GameState, pc: &PrivateCompany)
        -> (u32,usize)
{

    let mut who_won: u32 = 0;
    let mut which_bid: usize = 0;
    let mut result: u32 = 0;

    let mut index : usize = 0;
    while index < game_state.auction_bids.len()
    {
        if game_state.auction_bids[index].private_company == *pc &&
            game_state.auction_bids[index].bid_amount > result
        {
            result = game_state.auction_bids[index].bid_amount;
            who_won = game_state.auction_bids[index].order;
            which_bid = index;
        }
        index = index + 1;
    }
    info!("Player {} won the final auction", who_won);

    (who_won, which_bid)
}

// If the unsold private company with the lowest face value has at least
// one bid on it, the buy-bid-turn sequence is paused. If only one
// player has a bid on the private company, that player buys it
// for the amount of the bid. If multiple players have bid on the
// private company, an auction is held.

pub fn resolve_pc_bids(
    commands: &mut Commands,
    game_state: &mut GameState,
    players: &mut Query<&mut Player>,
    pc: PrivateCompany,
    bidders: u32,
    lowest_bidder: u32,
) {
    if bidders == 1
    {
        let mut idx = 0;
        while idx < game_state.auction_bids.len()
        {
            if game_state.auction_bids[idx].private_company == pc &&
               game_state.auction_bids[idx].order == lowest_bidder
            {
                info!("The {:?} is bought by player {} for {}",
                    pc, lowest_bidder,
                    game_state.auction_bids[idx].bid_amount);

                // We buy it for $0 because we're applying the bid, so
                // the money was already accounted for.

                buy_pc_impl( commands, game_state, players, lowest_bidder,
                    pc, 0);

                game_state.auction_bids.remove(idx);
                return;
            }
            idx += 1;
        }
    }

    info!("Finalize auction for {:?}", pc);

    game_state.auction_state.pc = pc;
    game_state.auction_state.num_bidders = bidders;
    game_state.auction_state.num_passes = 0;
    game_state.auction_state.current_bidder = lowest_bidder;
}

pub fn resolve_pass_impl(
    commands: &mut Commands,
    game_state: &mut GameState,
    players: &mut Query<&mut Player>,
    player_id: u32,
) {
    info!("Resolve pass");

    // Once all the bidders have passed consecutively, the auction ends:
    // - the high bidder buys the private company for the cost of their bid
    // - the other bids on the private company are terminated and the
    //   bid money returns to those players

    let pc = game_state.auction_state.pc;

    game_state.auction_state.num_passes += 1;

    info!("For PC {:?} there are {} bidders and there have been {} passes",
            game_state.auction_state.pc,
            game_state.auction_state.num_bidders,
            game_state.auction_state.num_passes);

    if game_state.auction_state.num_passes ==
       game_state.auction_state.num_bidders
    {
        let (who_won,which_bid) = highest_bid_for_pc(&game_state, &pc);
        
        for mut player in players.iter_mut()
        {
            if player.order == who_won
            {
                player.assets.private_companies[
                        game_state.auction_state.pc as usize] = 1;
                game_state.auction_bids.remove(which_bid);
            }
        }

        let mut i_rev = game_state.auction_bids.len() - 1;
        while i_rev >= 0
        {
            if game_state.auction_bids[i_rev].private_company == pc
            {
                let amt = game_state.auction_bids[i_rev].bid_amount;

                info!("Found a non-winning bid of {} for {:?}", amt, pc);
                
                for mut player in players.iter_mut()
                {
                    if player.order == game_state.auction_bids[i_rev].order
                    {
                        info!("Returning the money from the losing bid to {}",
                                player.order);
                        player.assets.personal_money += amt;
                    }
                }
                game_state.auction_bids.remove(i_rev);
            }
            else
            {
                info!("Bid of {} from player {} was not for {:?} but for {:?}",
                    game_state.auction_bids[i_rev].bid_amount,
                    game_state.auction_bids[i_rev].order,
                    pc,
                    game_state.auction_bids[i_rev].private_company);
            }
            if i_rev == 0
            {
                break;
            }
            i_rev -= 1;
        }
    }
}

pub fn create_auction_test_players(mut commands: Commands,
                    mut game_state: ResMut<GameState>)
{
    create_players( &mut commands,
            &mut game_state,
            vec!["Gerald".into(), "Dave".into(),
                "Bruce".into(), "Alex".into()]);
}

/// System to perform a simple private company auction test at startup.
///
/// It will be removed once the privco module is stabilized.
pub fn do_simple_auction_tests(
    mut commands: Commands,
    mut game_state: ResMut<GameState>,
    mut players: Query<&mut Player>,
) {
    info!("do_simple_auction_tests");

    // Gerald has the priority at the start of a 4 player game
    // with Dave, Bruce, and Alex to his left in that order.

    for player in players.iter()
    {
        info!("Player {} (id:{}) starts with money {}",
            player.name, player.order, player.assets.personal_money);
    }

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

    resolve_pc_bids( &mut commands, &mut game_state, &mut players,
                    PrivateCompany::DelawareAndHudson,
                    2, 2);

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
    resolve_pass_impl(&mut commands, &mut game_state, &mut players, 2);

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

    resolve_pc_bids( &mut commands, &mut game_state, &mut players,
                    PrivateCompany::CamdenAndAmboy,
                    2, 0);

    raise_bid_impl(&mut commands, &mut game_state, &mut players,
                    0, PrivateCompany::CamdenAndAmboy, 175);
    raise_bid_impl(&mut commands, &mut game_state, &mut players,
                    3, PrivateCompany::CamdenAndAmboy, 200);
    raise_bid_impl(&mut commands, &mut game_state, &mut players,
                    0, PrivateCompany::CamdenAndAmboy, 207);
    raise_bid_impl(&mut commands, &mut game_state, &mut players,
                    3, PrivateCompany::CamdenAndAmboy, 212);

    resolve_pass_impl(&mut commands, &mut game_state, &mut players, 0);
    resolve_pass_impl(&mut commands, &mut game_state, &mut players, 3);

    // There is only one bid on the BO, so Dave pays his $225
    // and takes the BO. 

    resolve_pc_bids( &mut commands, &mut game_state, &mut players,
                    PrivateCompany::BaltimoreAndOhio,
                    1, 1);

    // The end results are:
    // - Gerald: $490, MH
    // - Dave: $355, BO, SV, Priority Deal card, B&O President’s certificate
    // - Bruce: $505, DH,
    // - Alex: $348, CA, CL, 1 PRR share

    // all the bids should have been resolved
    for bid in &game_state.auction_bids
    {
        info!("Unresolved bid at end of test: player {} bid {} on pc {:?}",
                bid.order,
                bid.bid_amount,
                bid.private_company);
    }
    
    for player in players.iter()
    {
        info!("Player {} (id:{}) ends with money {}",
            player.name, player.order, player.assets.personal_money);
        info!("   private_companies owned: {:?}",
            player.assets.private_companies);
        info!("   railroad certificates owned: {:?}",
            player.assets.corporations);
    }

}
