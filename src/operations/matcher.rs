use merge::Merge;
use serde_json::Value;
use solana_sdk::instruction::Instruction;

use super::stake::*;
use super::system::*;
use super::vote::*;
use super::{spltoken, spltoken::*, stake, system, vote};
use crate::{
    error::ApiError,
    merge_meta, set_meta,
    types::Operation,
    types::{OperationType, OptionalInternalOperationMetadatas},
};
use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum InternalOperationMetadata {
    System(SystemOperationMetadata),
    Vote(VoteOperationMetadata),
    Stake(StakeOperationMetadata),
    SplToken(SplTokenOperationMetadata),
}
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct InternalOperation {
    pub metadata: InternalOperationMetadata,
    pub type_: OperationType,
}
impl InternalOperation {
    fn to_instruction(self) -> Result<Vec<Instruction>, ApiError> {
        Ok(match self.metadata {
            InternalOperationMetadata::System(x) => system::to_instruction(self.type_, x)?,
            InternalOperationMetadata::Vote(x) => vote::to_instruction(self.type_, x)?,
            InternalOperationMetadata::Stake(x) => stake::to_instruction(self.type_, x)?,
            InternalOperationMetadata::SplToken(x) => spltoken::to_instruction(self.type_, x)?,
        })
    }
}
pub struct Matcher<'a> {
    checked_indexes: Vec<u64>,
    operations: &'a Vec<Operation>,
    meta: OptionalInternalOperationMetadatas,
}

impl<'a> Matcher<'a> {
    pub fn new(operations: &Vec<Operation>, meta: OptionalInternalOperationMetadatas) -> Matcher {
        Matcher {
            operations,
            checked_indexes: vec![],
            meta,
        }
    }
    pub fn to_instructions(&mut self) -> Result<Vec<Instruction>, ApiError> {
        let combined = self.combine()?;
        Ok(combined
            .into_iter()
            .map(|x| x.to_instruction().unwrap())
            .flatten()
            .collect())
    }

    pub fn combine(&mut self) -> Result<Vec<InternalOperation>, ApiError> {
        let mut internal_operations = vec![];
        for i in 0..self.operations.len() {
            let operation = self.operations[i].clone();
            if !self
                .checked_indexes
                .contains(&operation.operation_identifier.index)
            {
                let mut meta_clone = operation.metadata.clone();
                if let Some(ref acc) = operation.account {
                    if let Some(ref amt) = operation.amount {
                        let clean_amt = amt.value.replace("-", "");
                        let matched_operation = self.operations.iter().find(|sub_op| {
                            if let Some(sub_op_amt) = &sub_op.amount {
                                if sub_op_amt.value.replace("-", "") == clean_amt
                                    && sub_op.type_ == operation.type_
                                    && sub_op_amt.currency.symbol == amt.currency.symbol
                                    && sub_op_amt.currency.decimals == amt.currency.decimals
                                    && sub_op.operation_identifier.index
                                        != operation.operation_identifier.index
                                    && !self
                                        .checked_indexes
                                        .contains(&sub_op.operation_identifier.index)
                                {
                                    true
                                } else {
                                    false
                                }
                            } else {
                                false
                            }
                        });
                        if let Some(matched_op) = matched_operation {
                            let x = &matched_op.operation_identifier;
                            let index = x.index;
                            let main_amount = amt.value.parse::<f64>().unwrap();
                            let main_address = acc.address.clone();

                            let amount = matched_op.amount();
                            let address = matched_op.address();

                            //if subamt is -ve then subamt is sender is source
                            let (source, destination, lamports) = if amount < 0.0 {
                                //negative = this is sender
                                (address, main_address, main_amount as u64)
                            } else {
                                (main_address, address, amount as u64)
                            };
                            if meta_clone.is_none() {
                                meta_clone = Some(serde_json::json!({}));
                            }
                            if let Some(ref mut m) = meta_clone {
                                if let Value::Object(ref mut map) = m {
                                    map.insert(
                                        "source".to_string(),
                                        serde_json::Value::String(source.clone()),
                                    );
                                    map.insert(
                                        "destination".to_string(),
                                        serde_json::Value::String(destination),
                                    );
                                    map.insert(
                                        "lamports".to_string(),
                                        serde_json::Value::Number(serde_json::Number::from(
                                            lamports,
                                        )),
                                    );
                                    map.insert(
                                        "amount".to_string(),
                                        serde_json::Value::Number(serde_json::Number::from(
                                            lamports,
                                        )),
                                    );
                                }
                            }
                            self.checked_indexes.push(index);
                        }
                    } else {
                        if meta_clone.is_none() {
                            meta_clone = Some(serde_json::json!({}));
                        }
                        if let Some(ref mut m) = meta_clone {
                            if let Value::Object(ref mut map) = m {
                                map.insert(
                                    "source".to_string(),
                                    serde_json::Value::String(acc.address.clone()),
                                );
                                if map.get("authority").is_none() {
                                    map.insert(
                                        "authority".to_string(),
                                        serde_json::Value::String(acc.address.clone()),
                                    );
                                }
                            }
                        }
                    }
                }
                if let Some(ref mut m) = meta_clone {
                    if let Value::Object(ref mut map) = m {
                        if map.get("authority").is_none() {
                            map.insert("authority".to_string(), map.get("source").unwrap().clone());
                        }
                    }
                }
                match operation.type_ {
                    OperationType::System__Assign
                    | OperationType::System__CreateAccount
                    | OperationType::System__Transfer
                    | OperationType::System__Allocate
                    | OperationType::System__CreateNonceAccount
                    | OperationType::System__AdvanceNonce
                    | OperationType::System__WithdrawFromNonce
                    | OperationType::System__AuthorizeNonce => {
                        let mut new_metadata = set_meta!(meta_clone, SystemOperationMetadata);
                        merge_meta!(new_metadata, &self.meta, internal_operations.len(), System);

                        internal_operations.push(InternalOperation {
                            type_: operation.type_,
                            metadata: InternalOperationMetadata::System(new_metadata),
                        })
                    }
                    OperationType::SplToken__InitializeMint
                    | OperationType::SplToken__InitializeAccount
                    | OperationType::SplToken__CreateToken
                    | OperationType::SplToken__CreateAccount
                    | OperationType::SplToken__Transfer
                    | OperationType::SplToken__Approve
                    | OperationType::SplToken__Revoke
                    | OperationType::SplToken__MintTo
                    | OperationType::SplToken__Burn
                    | OperationType::SplToken__CloseAccount
                    | OperationType::SplToken__FreezeAccount
                    | OperationType::SplToken__ThawAccount
                    | OperationType::SplToken__TransferChecked
                    | OperationType::SplToken__CreateAssocAccount => {
                        let mut new_metadata = set_meta!(meta_clone, SplTokenOperationMetadata);
                        merge_meta!(
                            new_metadata,
                            &self.meta,
                            internal_operations.len(),
                            SplToken
                        );

                        internal_operations.push(InternalOperation {
                            type_: operation.type_,
                            metadata: InternalOperationMetadata::SplToken(new_metadata),
                        })
                    }
                    OperationType::Stake__CreateAccount
                    | OperationType::Stake__Delegate
                    | OperationType::Stake__Split
                    | OperationType::Stake__Merge
                    | OperationType::Stake__Authorize
                    | OperationType::Stake__Withdraw
                    | OperationType::Stake__Deactivate
                    | OperationType::Stake__SetLockup => {
                        let mut new_metadata = set_meta!(meta_clone, StakeOperationMetadata);
                        merge_meta!(new_metadata, &self.meta, internal_operations.len(), Stake);
                        internal_operations.push(InternalOperation {
                            type_: operation.type_,
                            metadata: InternalOperationMetadata::Stake(new_metadata),
                        })
                    }
                    OperationType::Vote__CreateAccount
                    | OperationType::Vote__Authorize
                    | OperationType::Vote__Withdraw
                    | OperationType::Vote__UpdateValidatorIdentity
                    | OperationType::Vote__UpdateCommission => {
                        let mut new_metadata = set_meta!(meta_clone, VoteOperationMetadata);
                        merge_meta!(new_metadata, &self.meta, internal_operations.len(), Vote);
                        internal_operations.push(InternalOperation {
                            type_: operation.type_,
                            metadata: InternalOperationMetadata::Vote(new_metadata),
                        })
                    }
                    OperationType::Unknown => {}
                };
            }
        }
        Ok(internal_operations)
    }
}

#[cfg(test)]
mod tests {

    use crate::types::*;
    use serde_json::json;

    use super::Matcher;

    #[test]
    fn convert_op_test() {
        let operations = &vec![
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
        ];
        let mut matcher = Matcher::new(&operations, None);
        let ops = matcher.combine().unwrap();
        assert_eq!(ops.len(), 3);
        println!("{:?}", ops);
    }
}
