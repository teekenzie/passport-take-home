#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ScoreResponse, OwnerResponse, ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{State, STATE, SCORES};

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
        ExecuteMsg::SetScore {entering_addr, entering_token, score} => try_set_score(deps, info, entering_addr, entering_token, score),
    }
}

pub fn try_set_score(deps: DepsMut, info: MessageInfo, entering_addr: String, entering_token: String, score: i32) -> Result<Response, ContractError> {
    let entry_addr = deps.api.addr_validate(&entering_addr)?;
    let entry_token = entering_token.as_str();
    let state = STATE.load(deps.storage)?;
    let update_score = |_| -> StdResult<i32> {
        Ok(score)
    };
    if info.sender != state.owner {
        return Err(ContractError:: Unauthorized {});
    }

    SCORES.update(deps.storage, (&entry_addr, entry_token), update_score)?;
    Ok(Response::new().add_attribute("method", "set_score")) 
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetOwner {} => to_binary(&query_owner(deps)?),
        QueryMsg::GetScore {entering_addr, entering_token} => to_binary(&query_score(deps, entering_addr, entering_token)?),
    }
}

fn query_owner(deps: Deps) -> StdResult<OwnerResponse> {
    let state = STATE.load(deps.storage)?;
    Ok(OwnerResponse { owner: state.owner }) 
}

fn query_score(deps: Deps, entering_addr: String, entering_token: String) -> StdResult<ScoreResponse> {
    let entry_addr = deps.api.addr_validate(&entering_addr)?;
    let entry_token = entering_token.as_str();
    let map_score = SCORES.load(deps.storage, (&entry_addr, entry_token))?;
    Ok (ScoreResponse {score: map_score})
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
        let auth_info = mock_info("creator", &coins(2, "token"));
        let msg = ExecuteMsg:: SetScore { entering_addr: "anyone".to_string(), entering_token: "token Mirror".to_string(), score: 3}; 
        let _res = execute(deps.as_mut(), mock_env(), auth_info, msg).unwrap();

        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetScore { entering_addr: "anyone".to_string(), entering_token: "token Mirror".to_string()}).unwrap();
        let value: ScoreResponse = from_binary(&res).unwrap();
        assert_eq!(3, value.score);
        /* setting the score basic end */

        /* unauthorized setting the score start */
        let unauth_info = mock_info("anyone", &coins(2, "token"));
        let msg = ExecuteMsg::SetScore { entering_addr: "anyone".to_string(), entering_token: "token Mirror".to_string(), score: 10 };
        let res = execute(deps.as_mut(), mock_env(), unauth_info, msg);
        match res {
            Err(ContractError::Unauthorized {}) => {}
            _ => panic!("Must return unauthorized error"),
        }
        /* unauthorized setting the score end */

        /* setting the score of another address start */
        let auth_info = mock_info("creator", &coins(2, "token"));
        let msg = ExecuteMsg:: SetScore { entering_addr: "anyone2".to_string(), entering_token: "token Mirror".to_string(), score: 13}; 
        let _res = execute(deps.as_mut(), mock_env(), auth_info, msg).unwrap();

        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetScore { entering_addr: "anyone2".to_string(), entering_token: "token Mirror".to_string()}).unwrap();
        let value: ScoreResponse = from_binary(&res).unwrap();
        assert_eq!(13, value.score);
        /* setting the score of another address end */

        /* setting the score of first address again start */
        let auth_info = mock_info("creator", &coins(2, "token"));
        let msg = ExecuteMsg:: SetScore { entering_addr: "anyone".to_string(), entering_token: "token Mirror".to_string(), score: 25}; 
        let _res = execute(deps.as_mut(), mock_env(), auth_info, msg).unwrap();

        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetScore { entering_addr: "anyone".to_string(), entering_token: "token Mirror".to_string()}).unwrap();
        let value: ScoreResponse = from_binary(&res).unwrap();
        assert_eq!(25, value.score);
        /* setting the score of first address again end */

        /* verify the second score untouched start */
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetScore { entering_addr: "anyone2".to_string(), entering_token: "token Mirror".to_string()}).unwrap();
        let value: ScoreResponse = from_binary(&res).unwrap();
        assert_eq!(13, value.score);
        /* verify the second score untouched end */

        /* storing using another token start */
        let auth_info = mock_info("creator", &coins(2, "token"));
        let msg = ExecuteMsg:: SetScore { entering_addr: "anyone".to_string(), entering_token: "token TerraUST".to_string(), score: 57}; 
        let _res = execute(deps.as_mut(), mock_env(), auth_info, msg).unwrap();

        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetScore { entering_addr: "anyone".to_string(), entering_token: "token TerraUST".to_string()}).unwrap();
        let value: ScoreResponse = from_binary(&res).unwrap();
        assert_eq!(57, value.score);
        /* storing using another token end */

        /* verify the other token is left untouched start */
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetScore { entering_addr: "anyone".to_string(), entering_token: "token Mirror".to_string()}).unwrap();
        let value: ScoreResponse = from_binary(&res).unwrap();
        assert_eq!(25, value.score);
        /* verify the other token is left untouched end*/
    } 
}
