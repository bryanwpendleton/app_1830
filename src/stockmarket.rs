use bevy::prelude::*;
use bevy_egui::EguiContexts;

use crate::gamemodel::GameState;

/// The stock market records and governs the value of the railroad
/// corporations’ shares. This stock market is represented by a
/// large grid of colored sections and a share value token for each
/// corporation. Each corporation’s share value is equal to the number
/// in the grid box occupied by that corporation’s share value token. 

/// If a share value token is in a yellow, orange, or brown grid box,
/// there are some special effects.
///
/// Certificates of any corporation whose share value token is in
/// a yellow, orange, or brown grid box do not count against a
/// player’s overall certificate limit.
///
/// The shares of any corporation whose share value token is in
/// an orange or brown grid box may be held in excess of the
/// individual corporation certificate limit (60%).
///
/// During your turn in a stock round, you may purchase
/// any number of certificates from the bank pool of one
/// corporation whose share value token is in a brown grid box.
/// This purchase counts as your one certificate purchase for the
/// turn.
///
#[derive(Component)]
enum GridBoxColor {
    Clear,
    Yellow,
    Orange,
    Brown,
    Red,
}


/// Each GridBox in the stock market has:
/// - a unique name
/// - a share value
/// - a color
/// - the up, down, right, and left destinations upon movement
pub struct GridBox
{
    name: String,
    color: GridBoxColor,
    value: u32,
    up: String,
    down: String,
    right: String,
    left: String,
}

#[derive(Component)]
pub struct StockMarketCell {
    pub grid_box: GridBox,
}

/// System to initialize the Stock Market grid at game start.
/// 
/// To identify grid boxes, we use the "playing remotely" notation of
/// http://www.fwtwr.com/18xx/info/sm-1830.asp
///
pub fn initialize_stock_market(
    mut game_state: ResMut<GameState>,
) {
    /*
        Row A
     */

    game_state.market.insert(
        String::from("60A"),
        GridBox {
            name: String::from("60A"),
            color: GridBoxColor::Yellow,
            value: 60,
            up: String::from("60A"),
            down: String::from("53B"),
            right: String::from("67A"),
            left: String::from("60A"),
        });
    game_state.market.insert(
        String::from("67A"),
        GridBox {
            name: String::from("67A"),
            color: GridBoxColor::Clear,
            value: 67,
            up: String::from("67A"),
            down: String::from("60B"),
            right: String::from("71A"),
            left: String::from("60A"),
        });
    game_state.market.insert(
        String::from("71A"),
        GridBox {
            name: String::from("71A"),
            color: GridBoxColor::Clear,
            value: 71,
            up: String::from("71A"),
            down: String::from("66B"),
            right: String::from("76A"),
            left: String::from("67A"),
        });
    game_state.market.insert(
        String::from("76A"),
        GridBox {
            name: String::from("76A"),
            color: GridBoxColor::Clear,
            value: 76,
            up: String::from("76A"),
            down: String::from("70B"),
            right: String::from("82A"),
            left: String::from("71A"),
        });
    game_state.market.insert(
        String::from("82A"),
        GridBox {
            name: String::from("82A"),
            color: GridBoxColor::Clear,
            value: 82,
            up: String::from("82A"),
            down: String::from("76B"),
            right: String::from("90A"),
            left: String::from("76A"),
        });
    game_state.market.insert(
        String::from("90A"),
        GridBox {
            name: String::from("90A"),
            color: GridBoxColor::Clear,
            value: 90,
            up: String::from("90A"),
            down: String::from("82B"),
            right: String::from("100A"),
            left: String::from("82A"),
        });
    game_state.market.insert(
        String::from("100A"),
        GridBox {
            name: String::from("100A"),
            color: GridBoxColor::Red,
            value: 100,
            up: String::from("100A"),
            down: String::from("90B"),
            right: String::from("112A"),
            left: String::from("90A"),
        });
    game_state.market.insert(
        String::from("112A"),
        GridBox {
            name: String::from("112A"),
            color: GridBoxColor::Clear,
            value: 112,
            up: String::from("112A"),
            down: String::from("100B"),
            right: String::from("126A"),
            left: String::from("100A"),
        });
    game_state.market.insert(
        String::from("126A"),
        GridBox {
            name: String::from("126A"),
            color: GridBoxColor::Clear,
            value: 126,
            up: String::from("126A"),
            down: String::from("112B"),
            right: String::from("142A"),
            left: String::from("112A"),
        });
    game_state.market.insert(
        String::from("142A"),
        GridBox {
            name: String::from("142A"),
            color: GridBoxColor::Clear,
            value: 142,
            up: String::from("142A"),
            down: String::from("126B"),
            right: String::from("160A"),
            left: String::from("112A"),
        });
    game_state.market.insert(
        String::from("160A"),
        GridBox {
            name: String::from("160A"),
            color: GridBoxColor::Clear,
            value: 160,
            up: String::from("160A"),
            down: String::from("142B"),
            right: String::from("180A"),
            left: String::from("142A"),
        });
    game_state.market.insert(
        String::from("180A"),
        GridBox {
            name: String::from("180A"),
            color: GridBoxColor::Clear,
            value: 180,
            up: String::from("180A"),
            down: String::from("160B"),
            right: String::from("200A"),
            left: String::from("160A"),
        });
    game_state.market.insert(
        String::from("200A"),
        GridBox {
            name: String::from("200A"),
            color: GridBoxColor::Clear,
            value: 200,
            up: String::from("200A"),
            down: String::from("180B"),
            right: String::from("225A"),
            left: String::from("180A"),
        });
    game_state.market.insert(
        String::from("225A"),
        GridBox {
            name: String::from("225A"),
            color: GridBoxColor::Clear,
            value: 225,
            up: String::from("225A"),
            down: String::from("200B"),
            right: String::from("250A"),
            left: String::from("200A"),
        });
    game_state.market.insert(
        String::from("250A"),
        GridBox {
            name: String::from("250A"),
            color: GridBoxColor::Clear,
            value: 250,
            up: String::from("250A"),
            down: String::from("220B"),
            right: String::from("275A"),
            left: String::from("225A"),
        });
    game_state.market.insert(
        String::from("275A"),
        GridBox {
            name: String::from("275A"),
            color: GridBoxColor::Clear,
            value: 275,
            up: String::from("275A"),
            down: String::from("240B"),
            right: String::from("300A"),
            left: String::from("250A"),
        });
    game_state.market.insert(
        String::from("300A"),
        GridBox {
            name: String::from("300A"),
            color: GridBoxColor::Clear,
            value: 300,
            up: String::from("300A"),
            down: String::from("260B"),
            right: String::from("325A"),
            left: String::from("275A"),
        });
    game_state.market.insert(
        String::from("325A"),
        GridBox {
            name: String::from("325A"),
            color: GridBoxColor::Clear,
            value: 325,
            up: String::from("325A"),
            down: String::from("280B"),
            right: String::from("350A"),
            left: String::from("300A"),
        });
    game_state.market.insert(
        String::from("350A"),
        GridBox {
            name: String::from("350A"),
            color: GridBoxColor::Clear,
            value: 350,
            up: String::from("350A"),
            down: String::from("300B"),
            right: String::from("350A"),
            left: String::from("325A"),
        });
    /*
        Row B
     */
    game_state.market.insert(
        String::from("53B"),
        GridBox {
            name: String::from("53B"),
            color: GridBoxColor::Yellow,
            value: 53,
            up: String::from("60A"),
            down: String::from("46C"),
            right: String::from("60B"),
            left: String::from("46C"),
        });
    game_state.market.insert(
        String::from("60B"),
        GridBox {
            name: String::from("60B"),
            color: GridBoxColor::Yellow,
            value: 60,
            up: String::from("67A"),
            down: String::from("55C"),
            right: String::from("66B"),
            left: String::from("53B"),
        });
    game_state.market.insert(
        String::from("66B"),
        GridBox {
            name: String::from("66B"),
            color: GridBoxColor::Clear,
            value: 66,
            up: String::from("71A"),
            down: String::from("60C"),
            right: String::from("70B"),
            left: String::from("60B"),
        });
    game_state.market.insert(
        String::from("70B"),
        GridBox {
            name: String::from("70B"),
            color: GridBoxColor::Clear,
            value: 70,
            up: String::from("76A"),
            down: String::from("65C"),
            right: String::from("76B"),
            left: String::from("66B"),
        });
    game_state.market.insert(
        String::from("76B"),
        GridBox {
            name: String::from("76B"),
            color: GridBoxColor::Clear,
            value: 76,
            up: String::from("82A"),
            down: String::from("70C"),
            right: String::from("82B"),
            left: String::from("70B"),
        });
    game_state.market.insert(
        String::from("82B"),
        GridBox {
            name: String::from("82B"),
            color: GridBoxColor::Clear,
            value: 82,
            up: String::from("90A"),
            down: String::from("76C"),
            right: String::from("90B"),
            left: String::from("76B"),
        });
    game_state.market.insert(
        String::from("90B"),
        GridBox {
            name: String::from("90B"),
            color: GridBoxColor::Red,
            value: 90,
            up: String::from("100A"),
            down: String::from("82C"),
            right: String::from("100B"),
            left: String::from("82B"),
        });
    game_state.market.insert(
        String::from("100B"),
        GridBox {
            name: String::from("100B"),
            color: GridBoxColor::Clear,
            value: 100,
            up: String::from("112A"),
            down: String::from("90C"),
            right: String::from("112B"),
            left: String::from("90B"),
        });
    game_state.market.insert(
        String::from("112B"),
        GridBox {
            name: String::from("112B"),
            color: GridBoxColor::Clear,
            value: 112,
            up: String::from("126A"),
            down: String::from("100C"),
            right: String::from("126B"),
            left: String::from("100B"),
        });
    game_state.market.insert(
        String::from("126B"),
        GridBox {
            name: String::from("126B"),
            color: GridBoxColor::Clear,
            value: 126,
            up: String::from("142A"),
            down: String::from("111C"),
            right: String::from("142B"),
            left: String::from("112B"),
        });
    game_state.market.insert(
        String::from("142B"),
        GridBox {
            name: String::from("142B"),
            color: GridBoxColor::Clear,
            value: 142,
            up: String::from("160A"),
            down: String::from("125C"),
            right: String::from("160B"),
            left: String::from("126B"),
        });
    game_state.market.insert(
        String::from("160B"),
        GridBox {
            name: String::from("160B"),
            color: GridBoxColor::Clear,
            value: 160,
            up: String::from("180A"),
            down: String::from("140C"),
            right: String::from("180B"),
            left: String::from("142B"),
        });
    game_state.market.insert(
        String::from("180B"),
        GridBox {
            name: String::from("180B"),
            color: GridBoxColor::Clear,
            value: 180,
            up: String::from("200A"),
            down: String::from("155C"),
            right: String::from("200B"),
            left: String::from("160B"),
        });
    game_state.market.insert(
        String::from("200B"),
        GridBox {
            name: String::from("200B"),
            color: GridBoxColor::Clear,
            value: 200,
            up: String::from("225A"),
            down: String::from("170C"),
            right: String::from("220B"),
            left: String::from("180B"),
        });
    game_state.market.insert(
        String::from("220B"),
        GridBox {
            name: String::from("220B"),
            color: GridBoxColor::Clear,
            value: 220,
            up: String::from("250A"),
            down: String::from("185C"),
            right: String::from("240B"),
            left: String::from("200B"),
        });
    game_state.market.insert(
        String::from("240B"),
        GridBox {
            name: String::from("240B"),
            color: GridBoxColor::Clear,
            value: 240,
            up: String::from("275A"),
            down: String::from("200C"),
            right: String::from("260B"),
            left: String::from("220B"),
        });
    game_state.market.insert(
        String::from("260B"),
        GridBox {
            name: String::from("260B"),
            color: GridBoxColor::Clear,
            value: 260,
            up: String::from("300A"),
            down: String::from("260B"),
            right: String::from("280B"),
            left: String::from("240B"),
        });
    game_state.market.insert(
        String::from("280B"),
        GridBox {
            name: String::from("280B"),
            color: GridBoxColor::Clear,
            value: 280,
            up: String::from("325A"),
            down: String::from("280B"),
            right: String::from("300B"),
            left: String::from("260B"),
        });
    game_state.market.insert(
        String::from("300B"),
        GridBox {
            name: String::from("300B"),
            color: GridBoxColor::Clear,
            value: 300,
            up: String::from("350A"),
            down: String::from("300B"),
            right: String::from("350A"),
            left: String::from("280B"),
        });
    /*
        Row C
     */
    game_state.market.insert(
        String::from("46C"),
        GridBox {
            name: String::from("46C"),
            color: GridBoxColor::Yellow,
            value: 46,
            up: String::from("53B"),
            down: String::from("39D"),
            right: String::from("55C"),
            left: String::from("39D"),
        });
    game_state.market.insert(
        String::from("55C"),
        GridBox {
            name: String::from("55C"),
            color: GridBoxColor::Yellow,
            value: 55,
            up: String::from("60B"),
            down: String::from("48D"),
            right: String::from("60C"),
            left: String::from("46C"),
        });
    game_state.market.insert(
        String::from("60C"),
        GridBox {
            name: String::from("60C"),
            color: GridBoxColor::Yellow,
            value: 60,
            up: String::from("66B"),
            down: String::from("54D"),
            right: String::from("65C"),
            left: String::from("55C"),
        });
    game_state.market.insert(
        String::from("65C"),
        GridBox {
            name: String::from("65C"),
            color: GridBoxColor::Clear,
            value: 65,
            up: String::from("70B"),
            down: String::from("60D"),
            right: String::from("70C"),
            left: String::from("60C"),
        });
    game_state.market.insert(
        String::from("70C"),
        GridBox {
            name: String::from("70C"),
            color: GridBoxColor::Clear,
            value: 70,
            up: String::from("76B"),
            down: String::from("66D"),
            right: String::from("76C"),
            left: String::from("65C"),
        });
    game_state.market.insert(
        String::from("76C"),
        GridBox {
            name: String::from("76C"),
            color: GridBoxColor::Clear,
            value: 76,
            up: String::from("82B"),
            down: String::from("71D"),
            right: String::from("82C"),
            left: String::from("70C"),
        });
    game_state.market.insert(
        String::from("82C"),
        GridBox {
            name: String::from("82C"),
            color: GridBoxColor::Red,
            value: 82,
            up: String::from("90B"),
            down: String::from("76D"),
            right: String::from("90C"),
            left: String::from("76C"),
        });
    game_state.market.insert(
        String::from("90C"),
        GridBox {
            name: String::from("90C"),
            color: GridBoxColor::Clear,
            value: 90,
            up: String::from("100B"),
            down: String::from("82D"),
            right: String::from("100C"),
            left: String::from("82C"),
        });
    game_state.market.insert(
        String::from("100C"),
        GridBox {
            name: String::from("100C"),
            color: GridBoxColor::Clear,
            value: 100,
            up: String::from("112B"),
            down: String::from("90D"),
            right: String::from("111C"),
            left: String::from("90C"),
        });
    game_state.market.insert(
        String::from("111C"),
        GridBox {
            name: String::from("111C"),
            color: GridBoxColor::Clear,
            value: 111,
            up: String::from("126B"),
            down: String::from("100D"),
            right: String::from("125C"),
            left: String::from("100C"),
        });
    game_state.market.insert(
        String::from("125C"),
        GridBox {
            name: String::from("125C"),
            color: GridBoxColor::Clear,
            value: 125,
            up: String::from("142B"),
            down: String::from("110D"),
            right: String::from("140C"),
            left: String::from("111C"),
        });
    game_state.market.insert(
        String::from("140C"),
        GridBox {
            name: String::from("140C"),
            color: GridBoxColor::Clear,
            value: 140,
            up: String::from("160B"),
            down: String::from("120D"),
            right: String::from("155C"),
            left: String::from("125C"),
        });
    game_state.market.insert(
        String::from("155C"),
        GridBox {
            name: String::from("155C"),
            color: GridBoxColor::Clear,
            value: 155,
            up: String::from("180B"),
            down: String::from("130D"),
            right: String::from("170C"),
            left: String::from("140C"),
        });
    game_state.market.insert(
        String::from("170C"),
        GridBox {
            name: String::from("170C"),
            color: GridBoxColor::Clear,
            value: 170,
            up: String::from("200B"),
            down: String::from("170C"),
            right: String::from("185C"),
            left: String::from("155C"),
        });
    game_state.market.insert(
        String::from("185C"),
        GridBox {
            name: String::from("185C"),
            color: GridBoxColor::Clear,
            value: 185,
            up: String::from("220B"),
            down: String::from("185C"),
            right: String::from("200C"),
            left: String::from("170C"),
        });
    game_state.market.insert(
        String::from("200C"),
        GridBox {
            name: String::from("200C"),
            color: GridBoxColor::Clear,
            value: 200,
            up: String::from("240B"),
            down: String::from("200C"),
            right: String::from("240B"),
            left: String::from("185C"),
        });
    /*
        Row D
     */
    game_state.market.insert(
        String::from("39D"),
        GridBox {
            name: String::from("39D"),
            color: GridBoxColor::Orange,
            value: 39,
            up: String::from("46C"),
            down: String::from("32E"),
            right: String::from("48D"),
            left: String::from("32E"),
        });
    game_state.market.insert(
        String::from("48D"),
        GridBox {
            name: String::from("48D"),
            color: GridBoxColor::Yellow,
            value: 48,
            up: String::from("55C"),
            down: String::from("41E"),
            right: String::from("54D"),
            left: String::from("39D"),
        });
    game_state.market.insert(
        String::from("54D"),
        GridBox {
            name: String::from("54D"),
            color: GridBoxColor::Yellow,
            value: 54,
            up: String::from("60C"),
            down: String::from("48E"),
            right: String::from("60D"),
            left: String::from("48D"),
        });
    game_state.market.insert(
        String::from("60D"),
        GridBox {
            name: String::from("60D"),
            color: GridBoxColor::Yellow,
            value: 60,
            up: String::from("65C"),
            down: String::from("55E"),
            right: String::from("66D"),
            left: String::from("54D"),
        });
    game_state.market.insert(
        String::from("66D"),
        GridBox {
            name: String::from("66D"),
            color: GridBoxColor::Clear,
            value: 66,
            up: String::from("70C"),
            down: String::from("62E"),
            right: String::from("71D"),
            left: String::from("60D"),
        });
    game_state.market.insert(
        String::from("71D"),
        GridBox {
            name: String::from("71D"),
            color: GridBoxColor::Clear,
            value: 71,
            up: String::from("76C"),
            down: String::from("67E"),
            right: String::from("76D"),
            left: String::from("66D"),
        });
    game_state.market.insert(
        String::from("76D"),
        GridBox {
            name: String::from("76D"),
            color: GridBoxColor::Red,
            value: 76,
            up: String::from("82C"),
            down: String::from("71E"),
            right: String::from("82D"),
            left: String::from("71D"),
        });
    game_state.market.insert(
        String::from("82D"),
        GridBox {
            name: String::from("82D"),
            color: GridBoxColor::Clear,
            value: 82,
            up: String::from("90C"),
            down: String::from("76E"),
            right: String::from("90D"),
            left: String::from("76D"),
        });
    game_state.market.insert(
        String::from("90D"),
        GridBox {
            name: String::from("90D"),
            color: GridBoxColor::Clear,
            value: 90,
            up: String::from("100C"),
            down: String::from("82E"),
            right: String::from("100D"),
            left: String::from("82D"),
        });
    game_state.market.insert(
        String::from("100D"),
        GridBox {
            name: String::from("100D"),
            color: GridBoxColor::Clear,
            value: 100,
            up: String::from("111C"),
            down: String::from("90E"),
            right: String::from("110D"),
            left: String::from("90D"),
        });
    game_state.market.insert(
        String::from("110D"),
        GridBox {
            name: String::from("110D"),
            color: GridBoxColor::Clear,
            value: 110,
            up: String::from("125C"),
            down: String::from("100E"),
            right: String::from("120D"),
            left: String::from("100D"),
        });
    game_state.market.insert(
        String::from("120D"),
        GridBox {
            name: String::from("120D"),
            color: GridBoxColor::Clear,
            value: 120,
            up: String::from("140C"),
            down: String::from("120D"),
            right: String::from("130D"),
            left: String::from("110D"),
        });
    game_state.market.insert(
        String::from("130D"),
        GridBox {
            name: String::from("130D"),
            color: GridBoxColor::Clear,
            value: 130,
            up: String::from("155C"),
            down: String::from("130D"),
            right: String::from("155C"),
            left: String::from("120D"),
        });
    /*
        Row E
     */
    game_state.market.insert(
        String::from("32E"),
        GridBox {
            name: String::from("32E"),
            color: GridBoxColor::Orange,
            value: 32,
            up: String::from("39D"),
            down: String::from("25F"),
            right: String::from("41E"),
            left: String::from("25F"),
        });
    game_state.market.insert(
        String::from("41E"),
        GridBox {
            name: String::from("41E"),
            color: GridBoxColor::Orange,
            value: 41,
            up: String::from("48D"),
            down: String::from("34F"),
            right: String::from("48E"),
            left: String::from("32E"),
        });
    game_state.market.insert(
        String::from("48E"),
        GridBox {
            name: String::from("48E"),
            color: GridBoxColor::Yellow,
            value: 48,
            up: String::from("54D"),
            down: String::from("42F"),
            right: String::from("55E"),
            left: String::from("41E"),
        });
    game_state.market.insert(
        String::from("55E"),
        GridBox {
            name: String::from("55E"),
            color: GridBoxColor::Yellow,
            value: 55,
            up: String::from("60D"),
            down: String::from("50F"),
            right: String::from("62E"),
            left: String::from("48E"),
        });
    game_state.market.insert(
        String::from("62E"),
        GridBox {
            name: String::from("62E"),
            color: GridBoxColor::Clear,
            value: 62,
            up: String::from("66D"),
            down: String::from("58F"),
            right: String::from("67E"),
            left: String::from("55E"),
        });
    game_state.market.insert(
        String::from("67E"),
        GridBox {
            name: String::from("67E"),
            color: GridBoxColor::Clear,
            value: 67,
            up: String::from("71D"),
            down: String::from("65F"),
            right: String::from("71E"),
            left: String::from("62E"),
        });
    game_state.market.insert(
        String::from("71E"),
        GridBox {
            name: String::from("71E"),
            color: GridBoxColor::Red,
            value: 71,
            up: String::from("76D"),
            down: String::from("67F"),
            right: String::from("76E"),
            left: String::from("67E"),
        });
    game_state.market.insert(
        String::from("76E"),
        GridBox {
            name: String::from("76E"),
            color: GridBoxColor::Clear,
            value: 76,
            up: String::from("82D"),
            down: String::from("71F"),
            right: String::from("82E"),
            left: String::from("71E"),
        });
    game_state.market.insert(
        String::from("82E"),
        GridBox {
            name: String::from("82E"),
            color: GridBoxColor::Clear,
            value: 82,
            up: String::from("90D"),
            down: String::from("75F"),
            right: String::from("90E"),
            left: String::from("76E"),
        });
    game_state.market.insert(
        String::from("90E"),
        GridBox {
            name: String::from("90E"),
            color: GridBoxColor::Clear,
            value: 90,
            up: String::from("100D"),
            down: String::from("80F"),
            right: String::from("100E"),
            left: String::from("82E"),
        });
    game_state.market.insert(
        String::from("100E"),
        GridBox {
            name: String::from("100E"),
            color: GridBoxColor::Clear,
            value: 100,
            up: String::from("110D"),
            down: String::from("100E"),
            right: String::from("110D"),
            left: String::from("90E"),
        });
    /*
        Row F
     */
    game_state.market.insert(
        String::from("25F"),
        GridBox {
            name: String::from("25F"),
            color: GridBoxColor::Brown,
            value: 25,
            up: String::from("32E"),
            down: String::from("18G"),
            right: String::from("34F"),
            left: String::from("18G"),
        });
    game_state.market.insert(
        String::from("34F"),
        GridBox {
            name: String::from("34F"),
            color: GridBoxColor::Orange,
            value: 34,
            up: String::from("41E"),
            down: String::from("27G"),
            right: String::from("42F"),
            left: String::from("25F"),
        });
    game_state.market.insert(
        String::from("42F"),
        GridBox {
            name: String::from("42F"),
            color: GridBoxColor::Orange,
            value: 42,
            up: String::from("48E"),
            down: String::from("36G"),
            right: String::from("50F"),
            left: String::from("34F"),
        });
    game_state.market.insert(
        String::from("50F"),
        GridBox {
            name: String::from("50F"),
            color: GridBoxColor::Yellow,
            value: 50,
            up: String::from("55E"),
            down: String::from("45G"),
            right: String::from("58F"),
            left: String::from("42F"),
        });
    game_state.market.insert(
        String::from("58F"),
        GridBox {
            name: String::from("58F"),
            color: GridBoxColor::Yellow,
            value: 58,
            up: String::from("62E"),
            down: String::from("54G"),
            right: String::from("65F"),
            left: String::from("50F"),
        });
    game_state.market.insert(
        String::from("65F"),
        GridBox {
            name: String::from("65F"),
            color: GridBoxColor::Clear,
            value: 65,
            up: String::from("67E"),
            down: String::from("63G"),
            right: String::from("67F"),
            left: String::from("58F"),
        });
    game_state.market.insert(
        String::from("67F"),
        GridBox {
            name: String::from("67F"),
            color: GridBoxColor::Red,
            value: 67,
            up: String::from("71E"),
            down: String::from("67G"),
            right: String::from("71F"),
            left: String::from("65F"),
        });
    game_state.market.insert(
        String::from("71F"),
        GridBox {
            name: String::from("71F"),
            color: GridBoxColor::Clear,
            value: 71,
            up: String::from("76E"),
            down: String::from("69G"),
            right: String::from("75F"),
            left: String::from("67F"),
        });
    game_state.market.insert(
        String::from("75F"),
        GridBox {
            name: String::from("75F"),
            color: GridBoxColor::Clear,
            value: 75,
            up: String::from("82E"),
            down: String::from("70G"),
            right: String::from("80F"),
            left: String::from("71F"),
        });
    game_state.market.insert(
        String::from("80F"),
        GridBox {
            name: String::from("80F"),
            color: GridBoxColor::Clear,
            value: 80,
            up: String::from("90E"),
            down: String::from("80F"),
            right: String::from("90E"),
            left: String::from("75F"),
        });
    /*
        Row G
     */
    game_state.market.insert(
        String::from("18G"),
        GridBox {
            name: String::from("18G"),
            color: GridBoxColor::Brown,
            value: 18,
            up: String::from("25F"),
            down: String::from("10H"),
            right: String::from("27G"),
            left: String::from("10H"),
        });
    game_state.market.insert(
        String::from("27G"),
        GridBox {
            name: String::from("27G"),
            color: GridBoxColor::Brown,
            value: 27,
            up: String::from("34F"),
            down: String::from("20H"),
            right: String::from("36G"),
            left: String::from("18G"),
        });
    game_state.market.insert(
        String::from("36G"),
        GridBox {
            name: String::from("36G"),
            color: GridBoxColor::Orange,
            value: 36,
            up: String::from("42F"),
            down: String::from("30H"),
            right: String::from("45G"),
            left: String::from("27G"),
        });
    game_state.market.insert(
        String::from("45G"),
        GridBox {
            name: String::from("45G"),
            color: GridBoxColor::Orange,
            value: 45,
            up: String::from("50F"),
            down: String::from("40H"),
            right: String::from("54G"),
            left: String::from("36G"),
        });
    game_state.market.insert(
        String::from("54G"),
        GridBox {
            name: String::from("54G"),
            color: GridBoxColor::Yellow,
            value: 54,
            up: String::from("58F"),
            down: String::from("50H"),
            right: String::from("63G"),
            left: String::from("45G"),
        });
    game_state.market.insert(
        String::from("63G"),
        GridBox {
            name: String::from("63G"),
            color: GridBoxColor::Clear,
            value: 63,
            up: String::from("65F"),
            down: String::from("60H"),
            right: String::from("67G"),
            left: String::from("54G"),
        });
    game_state.market.insert(
        String::from("67G"),
        GridBox {
            name: String::from("67G"),
            color: GridBoxColor::Clear,
            value: 67,
            up: String::from("67F"),
            down: String::from("67H"),
            right: String::from("69G"),
            left: String::from("63G"),
        });
    game_state.market.insert(
        String::from("69G"),
        GridBox {
            name: String::from("69G"),
            color: GridBoxColor::Clear,
            value: 69,
            up: String::from("71F"),
            down: String::from("68H"),
            right: String::from("70G"),
            left: String::from("67G"),
        });
    game_state.market.insert(
        String::from("70G"),
        GridBox {
            name: String::from("70G"),
            color: GridBoxColor::Clear,
            value: 70,
            up: String::from("75F"),
            down: String::from("70G"),
            right: String::from("75F"),
            left: String::from("69G"),
        });
    /*
        Row H
     */
    game_state.market.insert(
        String::from("10H"),
        GridBox {
            name: String::from("10H"),
            color: GridBoxColor::Brown,
            value: 10,
            up: String::from("18G"),
            down: String::from("10H"),
            right: String::from("20H"),
            left: String::from("10H"),
        });
    game_state.market.insert(
        String::from("20H"),
        GridBox {
            name: String::from("20H"),
            color: GridBoxColor::Brown,
            value: 20,
            up: String::from("27G"),
            down: String::from("10I"),
            right: String::from("30H"),
            left: String::from("10H"),
        });
    game_state.market.insert(
        String::from("30H"),
        GridBox {
            name: String::from("30H"),
            color: GridBoxColor::Brown,
            value: 30,
            up: String::from("36G"),
            down: String::from("20I"),
            right: String::from("40H"),
            left: String::from("20H"),
        });
    game_state.market.insert(
        String::from("40H"),
        GridBox {
            name: String::from("40H"),
            color: GridBoxColor::Orange,
            value: 40,
            up: String::from("45G"),
            down: String::from("30I"),
            right: String::from("50H"),
            left: String::from("30H"),
        });
    game_state.market.insert(
        String::from("50H"),
        GridBox {
            name: String::from("50H"),
            color: GridBoxColor::Yellow,
            value: 50,
            up: String::from("54G"),
            down: String::from("40I"),
            right: String::from("60H"),
            left: String::from("40H"),
        });
    game_state.market.insert(
        String::from("60H"),
        GridBox {
            name: String::from("60H"),
            color: GridBoxColor::Yellow,
            value: 60,
            up: String::from("63G"),
            down: String::from("50I"),
            right: String::from("67H"),
            left: String::from("50H"),
        });
    game_state.market.insert(
        String::from("67H"),
        GridBox {
            name: String::from("67H"),
            color: GridBoxColor::Clear,
            value: 67,
            up: String::from("67G"),
            down: String::from("60I"),
            right: String::from("68H"),
            left: String::from("60H"),
        });
    game_state.market.insert(
        String::from("68H"),
        GridBox {
            name: String::from("68H"),
            color: GridBoxColor::Clear,
            value: 68,
            up: String::from("69G"),
            down: String::from("68H"),
            right: String::from("69G"),
            left: String::from("67H"),
        });
    /*
        Row I
     */
    game_state.market.insert(
        String::from("10I"),
        GridBox {
            name: String::from("10I"),
            color: GridBoxColor::Brown,
            value: 10,
            up: String::from("20H"),
            down: String::from("10I"),
            right: String::from("20I"),
            left: String::from("10I"),
        });
    game_state.market.insert(
        String::from("20I"),
        GridBox {
            name: String::from("20I"),
            color: GridBoxColor::Brown,
            value: 20,
            up: String::from("30H"),
            down: String::from("10J"),
            right: String::from("30I"),
            left: String::from("10I"),
        });
    game_state.market.insert(
        String::from("30I"),
        GridBox {
            name: String::from("30I"),
            color: GridBoxColor::Brown,
            value: 30,
            up: String::from("40H"),
            down: String::from("20J"),
            right: String::from("40I"),
            left: String::from("20I"),
        });
    game_state.market.insert(
        String::from("40I"),
        GridBox {
            name: String::from("40I"),
            color: GridBoxColor::Orange,
            value: 40,
            up: String::from("50H"),
            down: String::from("30J"),
            right: String::from("50I"),
            left: String::from("30I"),
        });
    game_state.market.insert(
        String::from("50I"),
        GridBox {
            name: String::from("50I"),
            color: GridBoxColor::Yellow,
            value: 50,
            up: String::from("60H"),
            down: String::from("40J"),
            right: String::from("60I"),
            left: String::from("40I"),
        });
    game_state.market.insert(
        String::from("60I"),
        GridBox {
            name: String::from("60I"),
            color: GridBoxColor::Yellow,
            value: 60,
            up: String::from("67H"),
            down: String::from("50J"),
            right: String::from("67H"),
            left: String::from("50I"),
        });
    /*
        Row J
     */
    game_state.market.insert(
        String::from("10J"),
        GridBox {
            name: String::from("10J"),
            color: GridBoxColor::Brown,
            value: 10,
            up: String::from("20I"),
            down: String::from("10J"),
            right: String::from("20J"),
            left: String::from("10J"),
        });
    game_state.market.insert(
        String::from("20J"),
        GridBox {
            name: String::from("20J"),
            color: GridBoxColor::Brown,
            value: 20,
            up: String::from("30I"),
            down: String::from("10K"),
            right: String::from("30J"),
            left: String::from("10J"),
        });
    game_state.market.insert(
        String::from("30J"),
        GridBox {
            name: String::from("30J"),
            color: GridBoxColor::Brown,
            value: 30,
            up: String::from("40I"),
            down: String::from("20K"),
            right: String::from("40J"),
            left: String::from("20J"),
        });
    game_state.market.insert(
        String::from("40J"),
        GridBox {
            name: String::from("40J"),
            color: GridBoxColor::Orange,
            value: 40,
            up: String::from("50I"),
            down: String::from("30K"),
            right: String::from("50J"),
            left: String::from("30J"),
        });
    game_state.market.insert(
        String::from("50J"),
        GridBox {
            name: String::from("50J"),
            color: GridBoxColor::Yellow,
            value: 50,
            up: String::from("60I"),
            down: String::from("40K"),
            right: String::from("60I"),
            left: String::from("40J"),
        });
    /*
        Row K
     */
    game_state.market.insert(
        String::from("10K"),
        GridBox {
            name: String::from("10K"),
            color: GridBoxColor::Brown,
            value: 10,
            up: String::from("20J"),
            down: String::from("10K"),
            right: String::from("20K"),
            left: String::from("10K"),
        });
    game_state.market.insert(
        String::from("20K"),
        GridBox {
            name: String::from("20K"),
            color: GridBoxColor::Brown,
            value: 20,
            up: String::from("30J"),
            down: String::from("20K"),
            right: String::from("30K"),
            left: String::from("10K"),
        });
    game_state.market.insert(
        String::from("30K"),
        GridBox {
            name: String::from("30K"),
            color: GridBoxColor::Brown,
            value: 30,
            up: String::from("40J"),
            down: String::from("30K"),
            right: String::from("40K"),
            left: String::from("20K"),
        });
    game_state.market.insert(
        String::from("40K"),
        GridBox {
            name: String::from("40K"),
            color: GridBoxColor::Orange,
            value: 40,
            up: String::from("50J"),
            down: String::from("40K"),
            right: String::from("50J"),
            left: String::from("30K"),
        });

    info!("Stock Grid initialized");
}

