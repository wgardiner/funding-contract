use cosmwasm_std::{
    to_binary, Api, Binary, Env, Extern, HandleResponse, InitResponse, MessageInfo, Querier,
    StdResult, Storage, CanonicalAddr,
};

use crate::error::ContractError;
use crate::msg::{CountResponse, HandleMsg, InitMsg, QueryMsg};
use crate::state::{config, config_read, State};

// Note, you can use StdResult in some functions where you do not
// make use of the custom errors
pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    _env: Env,
    info: MessageInfo,
    msg: InitMsg,
) -> StdResult<InitResponse> {
    // let pw: Vec<CanonicalAddr> = msg.proposer_whitelist.into_iter().map(|x| deps.api.canonical_address(&x)).collect::<Vec<CanonicalAddr>>()?;
    let pw: Vec<_> = msg.proposer_whitelist.into_iter().map(|x| deps.api.canonical_address(&x)).filter_map(Result::ok).collect::<Vec<_>>();
    let state = State {
        // count: msg.count,
        owner: deps.api.canonical_address(&info.sender)?,
        proposer_whitelist: pw,
        // voter_whitelist: Vec::new(),
        // // proposal_min_period: 10,
        // // voting_min_period: 10,
        // proposal_period_start: Some(1602868551443),
        // proposal_period_end: Some(1602868551445),
        // voting_period_start: Some(1602868551443),
        // voting_period_end: Some(1602868551443),
        // funding_formula: Some("QUADRATIC".to_string()),
        votes: Vec::new(),
        proposals: Vec::new(),
    };
    config(&mut deps.storage).save(&state)?;

    Ok(InitResponse::default())
    // TODO: handle expired with Err
}

// // And declare a custom Error variant for the ones where you will want to make use of it
// pub fn handle<S: Storage, A: Api, Q: Querier>(
//     deps: &mut Extern<S, A, Q>,
//     _env: Env,
//     info: MessageInfo,
//     msg: HandleMsg,
// ) -> Result<HandleResponse, ContractError> {
//     match msg {
//         // HandleMsg::Increment {} => try_increment(deps),
//         // HandleMsg::Reset { count } => try_reset(deps, info, count),
//     }
// }

// pub fn try_increment<S: Storage, A: Api, Q: Querier>(
//     deps: &mut Extern<S, A, Q>,
// ) -> Result<HandleResponse, ContractError> {
//     config(&mut deps.storage).update(|mut state| -> Result<_, ContractError> {
//         state.count += 1;
//         Ok(state)
//     })?;

//     Ok(HandleResponse::default())
// }

// pub fn try_reset<S: Storage, A: Api, Q: Querier>(
//     deps: &mut Extern<S, A, Q>,
//     info: MessageInfo,
//     count: i32,
// ) -> Result<HandleResponse, ContractError> {
//     let api = &deps.api;
//     config(&mut deps.storage).update(|mut state| -> Result<_, ContractError> {
//         if api.canonical_address(&info.sender)? != state.owner {
//             return Err(ContractError::Unauthorized {});
//         }
//         state.count = count;
//         Ok(state)
//     })?;
//     Ok(HandleResponse::default())
// }

pub fn query<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    _env: Env,
    msg: QueryMsg,
) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetCount {} => to_binary(&query_count(deps)?),
    }
}

fn query_count<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>) -> StdResult<CountResponse> {
    let state = config_read(&deps.storage).load()?;
    Ok(CountResponse {
        // count: state.proposer_whitelist.iter().map(|x| deps.api.human_address(x)).collect()?,
        count: state.proposer_whitelist,

    })
}

// pub fn query<S: Storage, A: Api, Q: Querier>(
//     deps: &Extern<S, A, Q>,
//     _env: Env,
//     msg: QueryMsg,
// ) -> StdResult<Binary> {
//     match msg {
//         QueryMsg::GetCount {} => to_binary(&query_count(deps)?),
//     }
// }

// fn query_count<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>) -> StdResult<CountResponse> {
//     let state = config_read(&deps.storage).load()?;
//     Ok(CountResponse { count: state.count })
// }

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{coins, from_binary, HumanAddr};

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies(&[]);

        let msg = InitMsg {
            // count: 17
            proposer_whitelist: vec![HumanAddr::from("proposer_0")],

        };
        let info = mock_info("creator", &coins(1000, "earth"));

        // we can just call .unwrap() to assert this was a success
        let res = init(&mut deps, mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        // it worked, let's query the state
        let res = query(&deps, mock_env(), QueryMsg::GetCount {}).unwrap();
        let value: CountResponse = from_binary(&res).unwrap();
        // let value: CountResponse = from_binary(&res).unwrap();
        println!("{:?}", value);
        // assert_eq!(17, value.count);
    }

    // #[test]
    // fn increment() {
    //     let mut deps = mock_dependencies(&coins(2, "token"));

    //     let msg = InitMsg { count: 17 };
    //     let info = mock_info("creator", &coins(2, "token"));
    //     let _res = init(&mut deps, mock_env(), info, msg).unwrap();

    //     // beneficiary can release it
    //     let info = mock_info("anyone", &coins(2, "token"));
    //     let msg = HandleMsg::Increment {};
    //     let _res = handle(&mut deps, mock_env(), info, msg).unwrap();

    //     // should increase counter by 1
    //     let res = query(&deps, mock_env(), QueryMsg::GetCount {}).unwrap();
    //     let value: CountResponse = from_binary(&res).unwrap();
    //     assert_eq!(18, value.count);
    // }

    // #[test]
    // fn reset() {
    //     let mut deps = mock_dependencies(&coins(2, "token"));

    //     let msg = InitMsg { count: 17 };
    //     let info = mock_info("creator", &coins(2, "token"));
    //     let _res = init(&mut deps, mock_env(), info, msg).unwrap();

    //     // beneficiary can release it
    //     let unauth_info = mock_info("anyone", &coins(2, "token"));
    //     let msg = HandleMsg::Reset { count: 5 };
    //     let res = handle(&mut deps, mock_env(), unauth_info, msg);
    //     match res {
    //         Err(ContractError::Unauthorized {}) => {}
    //         _ => panic!("Must return unauthorized error"),
    //     }

    //     // only the original creator can reset the counter
    //     let auth_info = mock_info("creator", &coins(2, "token"));
    //     let msg = HandleMsg::Reset { count: 5 };
    //     let _res = handle(&mut deps, mock_env(), auth_info, msg).unwrap();

    //     // should now be 5
    //     let res = query(&deps, mock_env(), QueryMsg::GetCount {}).unwrap();
    //     let value: CountResponse = from_binary(&res).unwrap();
    //     assert_eq!(5, value.count);
    // }
}
