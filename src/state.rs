use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::{Item, Map};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub owner: Addr,
    pub denom: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct FundsBox {
    pub creator: Addr,
    pub funds: Uint128,
    pub hashed_password: String,
    pub opener: Option<Addr>,
}

pub const STATE: Item<State> = Item::new("state");
pub const BOXES: Map<u64, FundsBox> = Map::new("boxes");
pub const BOXES_COUNT: Item<u64> = Item::new("boxes_count");
