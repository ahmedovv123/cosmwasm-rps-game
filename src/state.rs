use std::fmt::Display;

use cw_controllers::{Admin, Hooks};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]

pub struct State {
    pub owner: Addr
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Game {
    pub host: Addr,
    pub opponent: Addr,
    pub host_move: GameMove,
    pub opp_move: Option<GameMove>,
    pub result: Option<GameResult>
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum GameMove {
    Rock {},
    Paper {},
    Scissors {}
}

impl Display for GameMove {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GameMove::Rock {  } => write!(f, "Rock"),
            GameMove::Paper {  } => write!(f, "Paper"),
            GameMove::Scissors {  } => write!(f, "Scissors"),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum GameResult {
    HostWins {},
    OpponentWins {},
    Tie {}
}

impl Display for GameResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GameResult::HostWins {} => write!(f, "Host Wins!"),
            GameResult::OpponentWins {  } => write!(f, "Opponent Wins!"),
            GameResult::Tie {  } => write!(f, "Game is Tie !")
        }
    }
}

pub const STATE: Item<State> = Item::new("state");
pub const GAME: Map<(Addr, Addr), Game> = Map::new("game");
pub const ADMIN: Admin = Admin::new("admin");
pub const HOOKS: Hooks = Hooks::new("cw4-hooks");