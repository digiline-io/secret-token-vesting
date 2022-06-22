use cosmwasm_std::HumanAddr;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InitMsg {
    pub retrieval_time: u64, // time where the funds are unlocked
    pub validator: HumanAddr, // validator where the funds will be staked to
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleMsg {
    RetrieveFunds {},
    CompoundFunds {},
    Unstake {}
}


#[derive(Serialize, Deserialize, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub enum HandleAnswer {
    FundsRetrieved {},
    FundsCompounded {},
    FundsUnstaked {}
}



#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    FundsStatus {
        block_time: u64,
    }
}

#[derive(Serialize, Deserialize, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub enum QueryAnswer {
    FundsStatus {
        retrievable: bool, // whether the funds are retrievable
        remaining_time: u64, // how much longer the funds are locked
    }
}
