use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::Addr;
use cw_storage_plus::Item;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub owner: Addr,
    pub addrs: Vec<Addr>,
    pub scores: Vec<i32>,
}

pub const STATE: Item<State> = Item::new("state");
