pub mod matcher;
pub mod spltoken;
pub mod stake;
pub mod system;
pub mod utils;
pub mod vote;

use crate::{
    error::ApiError,
    types::{Operation, OptionalInternalOperationMetadatas},
};

use serde::{Deserialize, Serialize};
use solana_sdk::{message::Message, transaction::Transaction};

use self::matcher::Matcher;

impl Operation {
    pub fn amount(&self) -> f64 {
        self.amount.clone().unwrap().value.parse::<f64>().unwrap()
    }
    pub fn address(&self) -> String {
        self.account.clone().unwrap().address
    }
}

pub fn get_tx_from_str(s: &str) -> Result<Transaction, ApiError> {
    let try_bs58 = bs58::decode(&s).into_vec().unwrap_or(vec![]);
    if try_bs58.len() == 0 {
        return Err(ApiError::InvalidSignedTransaction);
    }
    let data = try_bs58;
    /*
    let try_base64 = base64::decode(&s);
    let data = if try_base64.is_err() {
    } else {
        try_base64.unwrap()
    };
    */
    Ok(bincode::deserialize(&data).unwrap())
}
