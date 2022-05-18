#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ScoreResponse, OwnerResponse, ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{State, STATE};
use cosmwasm_std::Addr;

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:take-home";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let state = State {
        owner: info.sender.clone(),
        addrs: vec![],
        scores: vec![],
    };
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    STATE.save(deps.storage, &state)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", info.sender))
    }

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::SetScore { addr, score } => try_set_score(deps, info, addr, score),
    }
}

pub fn try_set_score(deps: DepsMut, info: MessageInfo, addr: Addr, score: i32) -> Result<Response, ContractError> {
    STATE.update(deps.storage, |mut state| -> Result<_, ContractError> {
        if info.sender != state.owner {
            return Err(ContractError::Unauthorized {});
        }
        let index = search(&addr, &state.addrs);
        if index == -1 {
            state.addrs.push(addr);
            state.scores.push(score);
        } else {
            state.scores[index as usize] = score;
        }
        Ok(state)
    })?;
    Ok(Response::new().add_attribute("method", "reset"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetOwner {} => to_binary(&query_owner(deps)?),
        QueryMsg::GetScore {addr} => to_binary(&query_score(deps, addr)?),
    }
}

fn query_score(deps:Deps, addr:Addr) ->  StdResult<ScoreResponse> {
    let state = STATE.load(deps.storage)?;
    let index = search(&addr, &state.addrs);
    let mut result = 0;
    if index != -1 {
        result = state.scores[index as usize];
    }
    Ok(ScoreResponse {score: result})
}

fn query_owner(deps: Deps) -> StdResult<OwnerResponse> {
    let state = STATE.load(deps.storage)?;
    Ok(OwnerResponse { owner: state.owner }) 
}

fn search(addr: &Addr, addrs: &Vec<Addr>) -> i32 {
    for (i, vec_addr) in addrs.iter().enumerate() {
        if *vec_addr == *addr {
            return i as i32;
        }
    }
    return -1;
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies_with_balance, mock_env, mock_info};
    use cosmwasm_std::{coins, from_binary};

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies_with_balance(&coins(2, "token"));

        let msg = InstantiateMsg {};
        let info = mock_info("creator", &coins(1000, "earth"));

        // we can just call .unwrap() to assert this was a success
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());
        
        // it worked, let's query the owner
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetOwner {}).unwrap();
        let value: OwnerResponse = from_binary(&res).unwrap();
        assert_eq!("creator", value.owner);
    }

    #[test]
    fn set_score() {
        let mut deps = mock_dependencies_with_balance(&coins(2, "token")); 

        let msg = InstantiateMsg {};
        let info = mock_info("creator", &coins(2, "token"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        /* setting the score basic start */
        let other = Addr::unchecked("anyone");
        let auth_info = mock_info("creator", &coins(2, "token"));
        let msg = ExecuteMsg:: SetScore { addr: other, score: 3}; 
        let _res = execute(deps.as_mut(), mock_env(), auth_info, msg).unwrap();

        let other = Addr::unchecked("anyone"); 
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetScore { addr: other}).unwrap();
        let value: ScoreResponse = from_binary(&res).unwrap();
        assert_eq!(3, value.score);
        /* setting the score basic end */

        /* unauthorized setting the score start */
        let other = Addr::unchecked("anyone");
        let unauth_info = mock_info("anyone", &coins(2, "token"));
        let msg = ExecuteMsg::SetScore { addr: other, score: 10 };
        let res = execute(deps.as_mut(), mock_env(), unauth_info, msg);
        match res {
            Err(ContractError::Unauthorized {}) => {}
            _ => panic!("Must return unauthorized error"),
        }
        /* unauthorized setting the score end */

        /* setting the score of another address start */
        let other = Addr::unchecked("anyone2");
        let auth_info = mock_info("creator", &coins(2, "token"));
        let msg = ExecuteMsg:: SetScore { addr: other, score: 13}; 
        let _res = execute(deps.as_mut(), mock_env(), auth_info, msg).unwrap();

        let other = Addr::unchecked("anyone2"); 
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetScore { addr: other}).unwrap();
        let value: ScoreResponse = from_binary(&res).unwrap();
        assert_eq!(13, value.score);
        /* setting the score of another address end */

        /* setting the score of first address again start */
        let other = Addr::unchecked("anyone");
        let auth_info = mock_info("creator", &coins(2, "token"));
        let msg = ExecuteMsg:: SetScore { addr: other, score: 25}; 
        let _res = execute(deps.as_mut(), mock_env(), auth_info, msg).unwrap();

        let other = Addr::unchecked("anyone"); 
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetScore { addr: other}).unwrap();
        let value: ScoreResponse = from_binary(&res).unwrap();
        assert_eq!(25, value.score);
        /* setting the score of first address again end */

        /* verify the second score untouched start */
        let other = Addr::unchecked("anyone2"); 
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetScore { addr: other}).unwrap();
        let value: ScoreResponse = from_binary(&res).unwrap();
        assert_eq!(13, value.score);
        /* verify the second score untouched end */
    }
}
