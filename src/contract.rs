#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{Binary, Deps, DepsMut, Env, MessageInfo, Response, StdError, StdResult, to_json_binary, Uint128};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{Balance, BALANCES, NAME, Transaction, TRANSACTIONS, Transactions};

const CONTRACT_NAME: &str = "crates.io:ledger";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    NAME.save(deps.storage, &msg.name)?;

    Ok(Response::new())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Deposit { amount } => try_deposit(deps, env, info, amount),
        ExecuteMsg::Withdraw { amount } => try_withdraw(deps, env, info, amount),
        ExecuteMsg::Transfer { to, amount } => try_transfer(deps, env, info, to, amount),
    }
}

fn try_deposit(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    amount: Uint128,
) -> Result<Response, ContractError> {
    let sender = info.sender.clone();
    let mut balance = BALANCES.may_load(deps.storage, &sender)?.unwrap_or(Balance { amount: Uint128::zero() });

    balance.amount += amount;
    BALANCES.save(deps.storage, &sender, &balance)?;

    let transaction = Transaction {
        from: sender.clone(),
        to: None,
        amount,
        height: env.block.height,
        timestamp: env.block.time.seconds(),
        action: "deposit".to_string(),
    };
    TRANSACTIONS.update(deps.storage, &sender, |history| -> StdResult<_> {
        let mut history = history.unwrap_or_default();
        history.push(transaction);
        Ok(history)
    })?;

    Ok(Response::new().add_attribute("method", "deposit").add_attribute("amount", amount.to_string()))
}

fn try_withdraw(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    amount: Uint128,
) -> Result<Response, ContractError> {
    let sender = info.sender.clone();
    let mut balance = BALANCES.may_load(deps.storage, &sender)?.ok_or(ContractError::Std(StdError::generic_err("No balance found")))?;

    if balance.amount < amount {
        return Err(ContractError::Std(StdError::generic_err("Insufficient balance")));
    }

    balance.amount -= amount;
    BALANCES.save(deps.storage, &sender, &balance)?;

    let transaction = Transaction {
        from: sender.clone(),
        to: None,
        amount,
        height: env.block.height,
        timestamp: env.block.time.seconds(),
        action: "withdraw".to_string(),
    };
    TRANSACTIONS.update(deps.storage, &sender, |history| -> StdResult<_> {
        let mut history = history.unwrap_or_default();
        history.push(transaction);
        Ok(history)
    })?;

    Ok(Response::new().add_attribute("method", "withdraw").add_attribute("amount", amount.to_string()))
}

fn try_transfer(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    to: String,
    amount: Uint128,
) -> Result<Response, ContractError> {
    let sender = info.sender.clone();
    let recipient = deps.api.addr_validate(&to)?;
    let mut sender_balance = BALANCES.may_load(deps.storage, &sender)?.ok_or(ContractError::Std(StdError::generic_err("No balance found")))?;

    if sender_balance.amount < amount {
        return Err(ContractError::Std(StdError::generic_err("Insufficient balance")));
    }

    sender_balance.amount -= amount;
    BALANCES.save(deps.storage, &sender, &sender_balance)?;

    let mut recipient_balance = BALANCES.may_load(deps.storage, &recipient)?.unwrap_or(Balance { amount: Uint128::zero() });
    recipient_balance.amount += amount;
    BALANCES.save(deps.storage, &recipient, &recipient_balance)?;

    let transaction = Transaction {
        from: sender.clone(),
        to: Some(recipient.clone()),
        amount,
        height: env.block.height,
        timestamp: env.block.time.seconds(),
        action: "transfer".to_string(),
    };
    TRANSACTIONS.update(deps.storage, &sender, |history| -> StdResult<_> {
        let mut history = history.unwrap_or_default();
        history.push(transaction.clone());
        Ok(history)
    })?;
    TRANSACTIONS.update(deps.storage, &recipient, |history| -> StdResult<_> {
        let mut history = history.unwrap_or_default();
        history.push(transaction);
        Ok(history)
    })?;

    Ok(Response::new().add_attribute("method", "transfer").add_attribute("amount", amount.to_string()))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetLedgerName {} => to_json_binary(&query_get_name(deps)?),
        QueryMsg::GetBalance { address } => to_json_binary(&query_get_balance(deps, address)?),
        QueryMsg::GetTransactionHistory { address } => to_json_binary(&query_get_transaction_history(deps, address)?),
    }
}

fn query_get_name(
    deps: Deps
) -> StdResult<String> {
    NAME.load(deps.storage)
}

fn query_get_balance(
    deps: Deps,
    address: String,
) -> StdResult<Balance> {
    let addr = deps.api.addr_validate(&address)?;
    let balance = BALANCES.may_load(deps.storage, &addr)?.unwrap_or(Balance { amount: Uint128::zero() });

    Ok(balance)
}

fn query_get_transaction_history(
    deps: Deps,
    address: String,
) -> StdResult<Transactions> {
    let addr = deps.api.addr_validate(&address)?;
    let history = TRANSACTIONS.may_load(deps.storage, &addr)?.unwrap_or_default();

    Ok(history)
}
