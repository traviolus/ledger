use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Uint128;
use crate::state::{Balance, Transactions};

#[cw_serde]
pub struct InstantiateMsg {
    pub name: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    Deposit { amount: Uint128 },
    Withdraw { amount: Uint128 },
    Transfer { to: String, amount: Uint128 },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(String)]
    GetLedgerName {},
    #[returns(Balance)]
    GetBalance { address: String },
    #[returns(Transactions)]
    GetTransactionHistory { address: String },
}
