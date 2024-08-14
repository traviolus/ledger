use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::{Item, Map};

#[cw_serde]
pub struct Balance {
    pub amount: Uint128,
}

#[cw_serde]
pub struct Transaction {
    pub from: Addr,
    pub to: Option<Addr>,
    pub amount: Uint128,
    pub height: u64,
    pub timestamp: u64,
    pub action: String,
}

pub type Transactions = Vec<Transaction>;

pub const NAME: Item<String> = Item::new("name");
pub const BALANCES: Map<&Addr, Balance> = Map::new("balances");
pub const TRANSACTIONS: Map<&Addr, Vec<Transaction>> = Map::new("transactions");
