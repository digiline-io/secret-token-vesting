use cosmwasm_std::{to_binary, Api, Binary, Env, Extern, HandleResponse, InitResponse, Querier, StdError, StdResult, Storage, HandleResult, CosmosMsg, Uint128, BankMsg, Coin, StakingMsg};

use crate::msg::{HandleMsg, HandleAnswer, InitMsg, QueryMsg, QueryAnswer};
use crate::state::{save, load, Config, CONFIG_KEY};

pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: InitMsg,
) -> StdResult<InitResponse> {
    if env.message.sent_funds.is_empty() {
        return Err(StdError::generic_err("No funds sent"));
    }

    if env.message.sent_funds.len() > 1 {
        return Err(StdError::generic_err("Too many funds sent"));
    }

    if env.message.sent_funds[0].amount < Uint128::from(1u128) {
        return Err(StdError::generic_err("Amount must be greater than 0"));
    }

    if env.message.sent_funds[0].denom != "uscrt" {
        return Err(StdError::generic_err("Denom must be uscrt"));
    }

    let config = Config {
        owner: deps.api.canonical_address(&env.message.sender)?,
        retrieval_time: msg.retrieval_time,
        funds_retrieved: false,
        funds_unstaked: false,
        amount: env.message.sent_funds[0].amount.clone(),
        validator: msg.validator.clone(),
    };

    // Save data to storage
    save(&mut deps.storage, CONFIG_KEY, &config)?;
    //
    // let stakemsg = CosmosMsg::Staking(StakingMsg::Delegate {
    //     validator: msg.validator.clone(),
    //     amount: env.message.sent_funds[0].clone(),
    // });

    Ok(InitResponse {
        messages: vec![],
        // messages: vec![stakemsg],
        log: vec![]
    })
}

///-------------------------------------------- HANDLES ---------------------------------
pub fn handle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: HandleMsg,
) -> StdResult<HandleResponse> {
    match msg {
        HandleMsg::RetrieveFunds {} => handle_retrieve_funds(deps, env),
        HandleMsg::CompoundFunds {} => handle_compound_funds(deps, env),
        HandleMsg::Unstake {} => handle_unstake(deps, env),
    }
}



/// Returns HandleResult
///
/// Placeholder handle
///
/// # Arguments
///
/// * `deps` - mutable reference to Extern containing all the contract's external dependencies
/// * `env` - Env of contract's environment
pub fn handle_retrieve_funds<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
) -> HandleResult {
    let mut config: Config = load(&deps.storage, CONFIG_KEY)?;
    if config.owner != deps.api.canonical_address(&env.message.sender)? {
        return Err(StdError::generic_err("Only the owner can retrieve funds"));
    }
    if config.retrieval_time > env.block.time {
        return Err(StdError::generic_err("Retrieval time has not passed"));
    }
    if config.funds_retrieved {
        return Err(StdError::generic_err("Funds have already been retrieved"));
    }

    let sendmsg = CosmosMsg::Bank(BankMsg::Send {
        from_address: env.contract.address,
        to_address: deps.api.human_address(&config.owner)?,
        amount: vec![
            Coin {
                denom: "uscrt".to_string(),
                amount: config.amount.clone(),
            },
        ]
    });

    config.funds_retrieved = true;
    save(&mut deps.storage, CONFIG_KEY, &config)?;

    Ok(HandleResponse {
        messages: vec![sendmsg],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::FundsRetrieved {})?),
    })
}


/// Returns HandleResult
///
/// Placeholder handle
///
/// # Arguments
///
/// * `deps` - mutable reference to Extern containing all the contract's external dependencies
/// * `env` - Env of contract's environment
pub fn handle_compound_funds<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
) -> HandleResult {
    let mut config: Config = load(&deps.storage, CONFIG_KEY)?;
    if config.owner != deps.api.canonical_address(&env.message.sender)? {
        return Err(StdError::generic_err("Only the owner can compound funds"));
    }
    if config.funds_retrieved {
        return Err(StdError::generic_err("Funds have been retrieved already"));
    }
    if config.funds_unstaked {
        return Err(StdError::generic_err("Funds have been unstaked"));
    }

    let rewards_amount = deps.querier.query_delegation(
        env.contract.address.clone(),
        config.validator.clone(),
    )?;

    if rewards_amount.is_none() {
        return Err(StdError::generic_err("No rewards found"));
    }

    let rewards_amount = rewards_amount.unwrap();
    let rewards = rewards_amount.accumulated_rewards;

    if rewards.denom != "uscrt" {
        return Err(StdError::generic_err("Something went wrong, rewards denom must be uscrt"));
    }

    let retrieve_reward_msg = CosmosMsg::Staking(StakingMsg::Withdraw {
        recipient: Some(env.contract.address),
        validator: config.validator.clone()
    });

    config.amount += rewards.amount;

    save(&mut deps.storage, CONFIG_KEY, &config)?;

    Ok(HandleResponse {
        messages: vec![retrieve_reward_msg],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::FundsCompounded {})?),
    })
}


/// Returns HandleResult
///
/// Placeholder handle
///
/// # Arguments
///
/// * `deps` - mutable reference to Extern containing all the contract's external dependencies
/// * `env` - Env of contract's environment
pub fn handle_unstake<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
) -> HandleResult {
    let mut config: Config = load(&deps.storage, CONFIG_KEY)?;
    if config.owner != deps.api.canonical_address(&env.message.sender)? {
        return Err(StdError::generic_err("Only the owner can retrieve funds"));
    }
    if config.funds_retrieved {
        return Err(StdError::generic_err("Funds have already been retrieved"));
    }
    if config.funds_unstaked {
        return Err(StdError::generic_err("Funds have already been unstaked"));
    }

    let unstakemsg = CosmosMsg::Staking(StakingMsg::Undelegate {
        validator: config.validator.clone(),
        amount: Coin {
            denom: "uscrt".to_string(),
            amount: config.amount.clone(),
        }
    });

    config.funds_unstaked = true;
    save(&mut deps.storage, CONFIG_KEY, &config)?;

    Ok(HandleResponse {
        messages: vec![unstakemsg],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::FundsUnstaked {})?),
    })
}

// ---------------------------------------- QUERIES --------------------------------------


pub fn query<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    msg: QueryMsg,
) -> StdResult<Binary> {
    match msg {
        QueryMsg::FundsStatus { block_time } => to_binary(&query_funds_status(deps, block_time)?),
    }
}

fn query_funds_status<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>, block_time: u64) -> StdResult<QueryAnswer> {
    let config: Config = load(&deps.storage, CONFIG_KEY)?;
    let remaining_time = if config.retrieval_time > block_time {
        config.retrieval_time - block_time
    } else {
        0
    };
    let retrievable = remaining_time == 0;

    Ok(QueryAnswer::FundsStatus {
        remaining_time,
        retrievable,
    })
}
