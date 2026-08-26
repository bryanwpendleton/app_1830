use bevy::prelude::*;
use bevy_egui::EguiContexts;

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

