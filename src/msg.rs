use cosmwasm_std::{Addr, Response};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::state::{Game, GameMove, GameResult};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub owner: Addr,
    pub admin: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    StartGame {
        opponent: String,
        first_move: GameMove,
    },
    UpdateAdmin {
        address: String,
    },
    AddHook {
        address: String,
    },
    RemoveHook {
        address: String,
    },
    Respond {
        host: String,
        second_move: GameMove,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    // GetCount returns the current count as a json-encoded number
    GetResult { host: String, opponent: String },
    GetAdmin {},
}

// We define a custom struct for each query response
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct ResultResponse {
    pub result: GameResult,
}

impl Into<Response> for ResultResponse {
    fn into(self) -> Response {
        Response::new()
            .add_attribute("game_status", "finished")
            .add_attribute("Result", self.result.to_string())
    }
}

// We define a custom struct for each query response
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct GameResponse {
    pub result: Vec<Game>,
}
