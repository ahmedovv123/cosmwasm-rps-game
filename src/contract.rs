use std::convert::TryInto;

#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Addr, StdError, Order};
use cw2::set_contract_version;
use serde::__private::de::Content;

use crate::error::ContractError;
use crate::msg::{ResultResponse, ExecuteMsg, InstantiateMsg, QueryMsg, GameResponse};
use crate::state::{State, STATE, Game, GAME, GameMove, GameResult, HOOKS};

use crate::state::{ADMIN};
use cw0::{maybe_addr};
// version info for migration info
const CONTRACT_NAME: &str = "crates.io:rps";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    mut deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let state = State {
        owner: info.sender.clone(),

    };
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    STATE.save(deps.storage, &state)?;
    let api = deps.api;
    ADMIN.set(deps.branch(), maybe_addr(api, _msg.admin)?)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", info.sender.clone())
    )
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    let api = deps.api;
    match msg {
        ExecuteMsg::StartGame { opponent, first_move } => try_start_game(deps, info, opponent, first_move),
        ExecuteMsg::UpdateAdmin { address } => Ok(ADMIN.execute_update_admin(deps, info, maybe_addr(api, Some(address))?)?),
        ExecuteMsg::AddHook { address } => Ok(HOOKS.execute_add_hook(&ADMIN, deps, info, api.addr_validate(&address)?)?),
        ExecuteMsg::RemoveHook { address } => Ok(HOOKS.execute_remove_hook(&ADMIN, deps, info, api.addr_validate(&address)?)?),
        ExecuteMsg::Respond { host, second_move } => try_respond_to_game(deps, info, host, second_move),
    }
}

pub fn try_respond_to_game(deps: DepsMut, info: MessageInfo, host: String, second_move: GameMove) -> Result<Response, ContractError> {

    // validate host address
    let host_addr = deps.api.addr_validate(&host)?;
    let responder_addr = info.sender;

    //load game by passing host addr and opponent addr
    let mut game = GAME.load(deps.storage, (host_addr.clone(), responder_addr.clone()))?;

    if game.opponent != responder_addr {
        return Err(ContractError::UnauthorizedOpponent {})
    } else {
        game.opp_move = Some(second_move);
        game.result = get_game_result(game.clone());
    }

    let game_response = ResultResponse { result: game.result.unwrap() };
    GAME.remove(deps.storage, (host_addr, responder_addr));

    Ok(game_response.into())


}

pub fn get_game_result(game: Game) -> Option<GameResult> {
    let host_move = game.host_move;
    let opp_move = game.opp_move?;

    match host_move {
        GameMove::Rock {} => match opp_move {
            GameMove::Rock {} => Some(GameResult::Tie {}),
            GameMove::Paper {} => Some(GameResult::OpponentWins {}),
            GameMove::Scissors {} => Some(GameResult::HostWins {}),
        }
        GameMove::Paper {} => match opp_move {
            GameMove::Rock {} => Some(GameResult::HostWins {}),
            GameMove::Paper {} => Some(GameResult::Tie {}),
            GameMove::Scissors {} => Some(GameResult::OpponentWins {})
        }
        GameMove::Scissors {} => match opp_move {
            GameMove::Rock {} => Some(GameResult::OpponentWins {}),
            GameMove::Paper {} => Some(GameResult::HostWins {}),
            GameMove::Scissors {} => Some(GameResult::Tie {})
        }
    }

}


pub fn try_start_game(deps: DepsMut, info:MessageInfo, opponent: String, first_move: GameMove) -> Result<Response, ContractError> {
   
    // validate opponent address
    let opponent_addr = deps.api.addr_validate(&opponent)?;

    // try to start game, if game is already started with given host it will throw error, otherwise it will create a new game object and save it under host key
    let start_game = |host: Option<Game>| -> Result<Game, ContractError> {
        match host {
            Some(_) => Err(ContractError::AlreadyStarted {}),
            None => {
                let game = Game {
                    host: info.sender.clone(),
                    opponent: opponent_addr.clone(),
                    host_move: first_move.clone(),
                    opp_move: None,
                    result: None
                };
    
                Ok(game)
            }
        }
    };
    

    GAME.update(deps.storage, (info.sender.clone(), opponent_addr.clone()), start_game)?;


    let res = Response::new()
        .add_attribute("action", "start_game")
        .add_attribute("host", info.sender)
        .add_attribute("opponent", opponent_addr)
        .add_attribute("host_move", first_move.to_string());

    Ok(res)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetResult { host, opponent } => to_binary(&query_result(deps, host, opponent)?),
        QueryMsg::GetAdmin {} => to_binary(&ADMIN.query_admin(deps)?)
    }
}

pub fn query_result(deps: Deps, host: String, opponent: String) -> StdResult<ResultResponse> {

    // validate host address
    let validated_host = deps.api.addr_validate(&host)?;
    let validated_opponent = deps.api.addr_validate(&opponent)?;

    // load game map
    let game = GAME.may_load(deps.storage, (validated_host, validated_opponent))?;

    // get game result if its exist otherwise throw error
    match game {
        Some(game) => match game.result {
            Some(result) => Ok(ResultResponse { result }),
            None => Err(StdError::not_found("The game still has no winner"))
        }
        None => Err(StdError::not_found("Host has not started a game"))
    }


}

#[cfg(test)]
mod tests {
    use crate::state::GameMove;

    use super::*;
    use cosmwasm_std::testing::{mock_dependencies_with_balance, mock_env, mock_info};
    use cosmwasm_std::{coins, from_binary};
    use cw_controllers::AdminResponse;

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies_with_balance(&coins(2, "token"));

        
        let info = mock_info("creator", &coins(1000, "earth"));
        let msg = InstantiateMsg { owner: info.sender.clone(), admin: Some("creator".to_string()) };

        // we can just call .unwrap() to assert this was a success
        let res = instantiate(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();
        assert_eq!(0, res.messages.len());

        // it worked, let's query the result
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetResult { host: info.sender.to_string(), opponent: "opponent".to_string() });

        let _expected_result = "Host has not started a game".to_string();
        match res {
            Err(StdError::NotFound { kind: _expected_result }) => {},
            _ => panic!("Should not found game and panic")
        }

        let msg = QueryMsg::GetAdmin {};
        let _res = &query(deps.as_ref(), mock_env(), msg).unwrap();
        let value: AdminResponse = from_binary(&_res).unwrap();

        assert_eq!(Some("creator".to_string()), value.admin)
        
    }

    #[test]
    fn start_game() {
        let mut deps = mock_dependencies_with_balance(&coins(2, "token"));

        let info = mock_info("creator", &coins(1000, "earth"));
        let msg = ExecuteMsg::StartGame { opponent: "opponent".to_string(), first_move: GameMove::Paper {} };

        // try to start game
        execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

        // load game map
        let game = GAME.load(&deps.storage, (info.sender.clone(), Addr::unchecked("opponent"))).unwrap();

        assert_eq!(info.sender, game.host);
        assert_eq!(GameMove::Paper {}, game.host_move);
        assert_eq!(Addr::unchecked("opponent"), game.opponent);
        assert_eq!(None, game.opp_move);
        assert_eq!(None, game.result);
        
    }

    #[test]
    fn start_game_with_same_host_diff_opponent() {
        let mut deps = mock_dependencies_with_balance(&coins(2, "token"));
        let info = mock_info("creator", &coins(1000, "earth"));
        let msg = ExecuteMsg::StartGame { opponent: "opponent".to_string(), first_move: GameMove::Paper {} };

        // try to start game
        execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

        let msg = ExecuteMsg::StartGame { opponent: "opponent2".to_string(), first_move: GameMove::Paper {} };

        // try to start second game
        execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

        // load games map
        let game1 = GAME.load(&deps.storage, (info.sender.clone(), Addr::unchecked("opponent"))).unwrap();
        let game2 = GAME.load(&deps.storage, (info.sender.clone(), Addr::unchecked("opponent2"))).unwrap();

        assert_eq!(info.sender, game1.host);
        assert_eq!(GameMove::Paper {}, game1.host_move);
        assert_eq!(Addr::unchecked("opponent"), game1.opponent);
        assert_eq!(None, game1.opp_move);
        assert_eq!(None, game1.result);

        assert_eq!(info.sender, game2.host);
        assert_eq!(GameMove::Paper {}, game2.host_move);
        assert_eq!(Addr::unchecked("opponent2"), game2.opponent);
        assert_eq!(None, game2.opp_move);
        assert_eq!(None, game2.result);

    }

    #[test]
    pub fn try_to_change_admin() {
        let mut deps = mock_dependencies_with_balance(&coins(2, "token"));
        let info = mock_info("creator", &coins(1000, "earth"));
        let msg = ExecuteMsg::UpdateAdmin { address: "new_admin".to_string() };
        let _initMsg = InstantiateMsg { owner: info.sender.clone(), admin: Some("creator".to_string()) };
        let env = mock_env();

        instantiate(deps.as_mut(), mock_env(), info.clone(), _initMsg).unwrap();
        execute(deps.as_mut(), env, info, msg).unwrap();

        let msg = QueryMsg::GetAdmin {};
        let _res = &query(deps.as_ref(), mock_env(), msg).unwrap();
        let value: AdminResponse = from_binary(&_res).unwrap();

        assert_eq!(Some("new_admin".to_string()), value.admin)
    }

}
