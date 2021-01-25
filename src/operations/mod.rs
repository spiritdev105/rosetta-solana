pub mod utils;

use crate::{
    error::ApiError,
    types::{Operation, OperationType},
    utils::to_pub,
};
use solana_sdk::{instruction::Instruction, program_pack::Pack, system_instruction};

use serde::{Deserialize, Serialize};
use spl_token::state::Mint;

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct TransferOpMetaTokenAmount {
    #[serde(skip_serializing_if = "Option::is_none")]
    amount: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    decimals: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    uiAmount: Option<f64>,
}
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct TransferOpMeta {
    ///owner of sender address
    #[serde(skip_serializing_if = "Option::is_none")]
    authority: Option<String>,
    ///sender token wallet address
    #[serde(skip_serializing_if = "Option::is_none")]
    source: Option<String>,
    ///destination token wallet address
    #[serde(skip_serializing_if = "Option::is_none")]
    destination: Option<String>,
    /// owner of source address
    #[serde(skip_serializing_if = "Option::is_none")]
    mint: Option<String>,
    /// decimals
    #[serde(skip_serializing_if = "Option::is_none")]
    decimals: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "tokenAmount")]
    token_amount: Option<TransferOpMetaTokenAmount>,

    #[serde(skip_serializing_if = "Option::is_none")]
    amount: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    lamports: Option<u64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    space: Option<u64>,
}

impl TransferOpMeta {
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
impl From<&Option<serde_json::Value>> for TransferOpMeta {
    fn from(meta: &Option<serde_json::Value>) -> Self {
        let op = serde_json::from_value::<Self>(meta.clone().unwrap()).unwrap();
        op
    }
}

impl Operation {
    pub fn amount(&self) -> i64 {
        self.amount.clone().unwrap().value.parse::<i64>().unwrap()
    }
    pub fn address(&self) -> String {
        self.account.clone().unwrap().address
    }
    pub fn to_instruction(&self) -> Result<Instruction, ApiError> {
        let metadata = TransferOpMeta::from(&self.metadata);
        let amount = metadata.amount_u64();
        let instruction = match self.type_ {
            OperationType::System__CreateAccount => system_instruction::create_account(
                &to_pub(&metadata.source.clone().unwrap()),
                &to_pub(&metadata.mint.clone().unwrap()),
                1461600, //min rent
                Mint::LEN as u64,
                &spl_token::id(),
            ),
            OperationType::SplToken__InitializeMint => spl_token::instruction::initialize_mint(
                &spl_token::id(),
                &to_pub(&metadata.mint.clone().unwrap()),
                &to_pub(&metadata.source.clone().unwrap()),
                Some(&to_pub(&metadata.source.clone().unwrap())),
                metadata.decimals.unwrap(),
            )?,
            OperationType::System__Transfer => system_instruction::transfer(
                &to_pub(&metadata.source.unwrap()),
                &to_pub(&metadata.destination.unwrap()),
                amount,
            ),
            OperationType::SplToken__CreateAssocAccount => {
                spl_associated_token_account::create_associated_token_account(
                    &to_pub(&metadata.source.clone().unwrap()),
                    &to_pub(&metadata.source.unwrap()),
                    &to_pub(&metadata.mint.unwrap()),
                )
            }

            OperationType::SplToken__TransferChecked => spl_token::instruction::transfer_checked(
                &spl_token::id(),
                &to_pub(&metadata.source.unwrap()),
                &to_pub(&metadata.mint.unwrap()),
                &to_pub(&metadata.destination.unwrap()),
                &to_pub(&metadata.authority.unwrap()),
                &vec![],
                amount,
                metadata.decimals.unwrap(),
            )?,
            OperationType::SplToken__Transfer => spl_token::instruction::transfer(
                &spl_token::id(),
                &to_pub(&metadata.source.unwrap()),
                &to_pub(&metadata.destination.unwrap()),
                &to_pub(&metadata.authority.unwrap()),
                &vec![],
                amount,
            )?,
            OperationType::SplToken__Burn => spl_token::instruction::burn(
                &spl_token::id(),
                &to_pub(&metadata.source.clone().unwrap()),
                &to_pub(&metadata.mint.unwrap()),
                &to_pub(&metadata.source.unwrap()),
                &vec![],
                amount,
            )?,
            OperationType::SplToken__Mint => spl_token::instruction::mint_to(
                &spl_token::id(),
                &to_pub(&metadata.source.clone().unwrap()),
                &to_pub(&metadata.mint.unwrap()),
                &to_pub(&metadata.source.unwrap()),
                &vec![],
                amount,
            )?,
            OperationType::Unknown => {
                return Err(ApiError::BadOperations(
                    "Operation Not Supported".to_string(),
                ))
            }
        };
        Ok(instruction)
    }
}

#[cfg(test)]
mod tests {
    use crate::types::*;
    use serde_json::json;

    use super::utils::combine_related_operations;
    #[test]
    fn convert_op_test() {
        let ops = combine_related_operations(&vec![
            Operation {
                operation_identifier: OperationIdentifier {
                    index: 0,
                    network_index: None,
                },
                related_operations: None,
                status: None,
                account: Some(AccountIdentifier {
                    address: "SenderAddress".to_string(),
                    sub_account: None,
                }),
                amount: Some(Amount {
                    value: "-1000".to_string(),
                    currency: Currency {
                        symbol: "TEST".to_string(),
                        decimals: 10,
                        metadata: None,
                    },
                }),
                type_: OperationType::System__Transfer,
                metadata: None,
            },
            Operation {
                operation_identifier: OperationIdentifier {
                    index: 1,
                    network_index: None,
                },
                related_operations: None,
                status: None,
                account: Some(AccountIdentifier {
                    address: "DestinationAddress".to_string(),
                    sub_account: None,
                }),
                amount: Some(Amount {
                    value: "1000".to_string(),
                    currency: Currency {
                        symbol: "TEST".to_string(),
                        decimals: 10,
                        metadata: None,
                    },
                }),
                type_: OperationType::System__Transfer,
                metadata: None,
            },
            //unrelated operation
            Operation {
                operation_identifier: OperationIdentifier {
                    index: 5,
                    network_index: None,
                },
                related_operations: None,
                status: None,
                account: None,
                amount: None,
                type_: OperationType::System__Transfer,
                metadata: Some(json!({
                    "source": "SomeUnrelatedSender",
                    "destination": "SomeUnrelatedDest",
                    "lamports": 10000,
                })),
            },
            Operation {
                operation_identifier: OperationIdentifier {
                    index: 10,
                    network_index: None,
                },
                related_operations: None,
                status: None,
                account: Some(AccountIdentifier {
                    address: "SS".to_string(),
                    sub_account: None,
                }),
                amount: Some(Amount {
                    value: "-10".to_string(),
                    currency: Currency {
                        symbol: "MM".to_string(),
                        decimals: 2,
                        metadata: None,
                    },
                }),
                type_: OperationType::SplToken__TransferChecked,
                metadata: Some(json!({
                    "authority": "AA",
                })),
            },
            Operation {
                operation_identifier: OperationIdentifier {
                    index: 11,
                    network_index: None,
                },
                related_operations: None,
                status: None,
                account: Some(AccountIdentifier {
                    address: "DD".to_string(),
                    sub_account: None,
                }),
                amount: Some(Amount {
                    value: "10".to_string(),
                    currency: Currency {
                        symbol: "MM".to_string(),
                        decimals: 2,
                        metadata: None,
                    },
                }),
                type_: OperationType::SplToken__TransferChecked,
                metadata: Some(json!({
                    "authority": "AA",
                })),
            },
        ])
        .unwrap();
        assert_eq!(ops.len(), 3);
        assert_eq!(ops[0].metadata.clone().unwrap().to_string(),"{\"amount\":\"1000\",\"decimals\":10,\"destination\":\"DestinationAddress\",\"lamports\":1000,\"mint\":\"TEST\",\"source\":\"SenderAddress\"}");
        assert_eq!(ops[1].metadata.clone().unwrap().to_string(),"{\"destination\":\"SomeUnrelatedDest\",\"lamports\":10000,\"source\":\"SomeUnrelatedSender\"}");
        assert_eq!(ops[2].metadata.clone().unwrap().to_string(),"{\"amount\":\"10\",\"authority\":\"AA\",\"decimals\":2,\"destination\":\"DD\",\"lamports\":10,\"mint\":\"MM\",\"source\":\"SS\"}");
    }
}
