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

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct OpMetaTokenAmount {
    #[serde(skip_serializing_if = "Option::is_none")]
    amount: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    decimals: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    uiAmount: Option<f64>,
}
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct OpMeta {
    ///owner of sender address
    #[serde(skip_serializing_if = "Option::is_none", alias = "custodian")]
    authority: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    new_authority: Option<String>,
    ///sender token wallet address
    #[serde(skip_serializing_if = "Option::is_none")]
    source: Option<String>,
    ///destination token wallet address
    #[serde(
        skip_serializing_if = "Option::is_none",
        alias = "nonce_pubkey",
        alias = "stake_pubkey"
    )]
    destination: Option<String>,
    /// owner of source address
    #[serde(skip_serializing_if = "Option::is_none")]
    mint: Option<String>,
    /// decimals
    #[serde(skip_serializing_if = "Option::is_none")]
    decimals: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "tokenAmount")]
    token_amount: Option<OpMetaTokenAmount>,

    #[serde(skip_serializing_if = "Option::is_none")]
    amount: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    lamports: Option<u64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    space: Option<u64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    owner: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", alias = "voter")]
    staker: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    withdrawer: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    vote_pubkey: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    custodian: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    comission: Option<u8>,
}
impl OpMeta {
    fn amount_str(&self) -> String {
        if let Some(x) = &self.lamports {
            x.to_string()
        } else if let Some(x) = &self.amount {
            x.to_string()
        } else if let Some(x) = &self.token_amount {
            x.amount.clone().unwrap()
        } else {
            "0".to_string()
        }
    }
    fn amount_u64(&self) -> u64 {
        if let Some(x) = &self.lamports {
            *x
        } else if let Some(x) = &self.amount {
            x.parse::<u64>().unwrap()
        } else if let Some(x) = &self.token_amount {
            x.amount.clone().unwrap().parse::<u64>().unwrap()
        } else {
            0 as u64
        }
    }
}
impl From<&Option<serde_json::Value>> for OpMeta {
    fn from(meta: &Option<serde_json::Value>) -> Self {
        let op = serde_json::from_value::<Self>(meta.clone().unwrap()).unwrap();
        op
    }
}

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
