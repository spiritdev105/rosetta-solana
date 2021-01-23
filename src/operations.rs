use crate::{
    consts,
    error::ApiError,
    types::AccountIdentifier,
    types::{Amount, Currency, Operation, OperationIdentifier, OperationStatusType, OperationType},
    utils::get_operation_type,
    utils::get_operation_type_with_program,
    utils::to_pub,
};

use serde::{Deserialize, Serialize};
use solana_sdk::{instruction::Instruction, system_instruction};
use solana_transaction_status::{
    parse_instruction::ParsedInstructionEnum, EncodedTransaction, UiInstruction, UiMessage,
    UiParsedInstruction,
};

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
struct TransferOpMetaTokenAmount {
    #[serde(skip_serializing_if = "Option::is_none")]
    amount: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    decimals: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    uiAmount: Option<f64>,
}
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
struct TransferOpMeta {
    ///owner of sender address
    #[serde(skip_serializing_if = "Option::is_none")]
    authority: Option<String>,
    ///sender token wallet address
    #[serde(skip_serializing_if = "Option::is_none")]
    source: Option<String>,
    ///destination token wallet address
    #[serde(skip_serializing_if = "Option::is_none")]
    destination: Option<String>,
    ///amount in tokens
    #[serde(skip_serializing_if = "Option::is_none", alias = "lamports")]
    amount: Option<u64>,
    /// owner of source address
    #[serde(skip_serializing_if = "Option::is_none")]
    mint: Option<String>,
    /// decimals
    #[serde(skip_serializing_if = "Option::is_none")]
    decimals: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "tokenAmount")]
    token_amount: Option<TransferOpMetaTokenAmount>,
}
impl TransferOpMeta {
    fn amount(&self) -> String {
        if let Some(x) = self.amount {
            x.to_string()
        } else if let Some(x) = &self.token_amount {
            x.amount.clone().unwrap()
        } else {
            "NaN".to_string()
        }
    }
}
impl From<&Option<serde_json::Value>> for TransferOpMeta {
    fn from(meta: &Option<serde_json::Value>) -> Self {
        serde_json::from_value::<Self>(meta.clone().unwrap()).unwrap()
    }
}

//TODO: further combine macros if possible
macro_rules! push_op {
    ($op:expr, $metadata:expr, $combined_operations:expr) => {
        let mut new_op = $op.clone();
        new_op.metadata = Some(serde_json::to_value($metadata).unwrap());
        new_op.related_operations = None;
        new_op.account = None;
        new_op.amount = None;

        $combined_operations.push(new_op);
    };
}

macro_rules! get_meta {
    ($op:expr, $metadata:expr, $metastruct:ident) => {
        $metadata = if let Some(x) = &$op.metadata {
            serde_json::from_value::<$metastruct>(x.clone()).unwrap()
        } else {
            $metastruct::default()
        };
    };
}

//convert rosetta style list of operations to single instruction per operation with metadata
pub fn combine_related_operations(operations: &Vec<Operation>) -> Result<Vec<Operation>, ApiError> {
    //For common operations e.g transfer rosetta "recommends" 1 op per account
    //but we do 1 op = 1 instruction
    //combine all related op to 1 op and put data in metadata

    let mut combined_operations: Vec<Operation> = vec![];
    let mut checked_related_op_indexes: Vec<u64> = vec![];
    operations.iter().for_each(|op| {
        if !checked_related_op_indexes.contains(&op.operation_identifier.index) {
            match op.type_ {
                OperationType::System__Transfer
                | OperationType::SplToken__Transfer
                | OperationType::SplToken__TransferChecked => {
                    let mut related_op: Option<&Operation> = None;
                    if op.account.is_some() && op.amount.is_some() {
                        let op_amount = op.amount.clone().unwrap();
                        let clean_amt = op_amount.value.replace("-", "");
                        related_op = operations.iter().find(|x| {
                            if let Some(xamt) = &x.amount {
                                if xamt.value.replace("-", "") == clean_amt
                                    && xamt.currency.symbol == op_amount.currency.symbol
                                    && xamt.currency.decimals == op_amount.currency.decimals
                                    && x.operation_identifier.index != op.operation_identifier.index
                                    && !checked_related_op_indexes
                                        .contains(&x.operation_identifier.index)
                                {
                                    true
                                } else {
                                    false
                                }
                            } else {
                                false
                            }
                        });
                    }
                    let mut metadata: TransferOpMeta;
                    get_meta!(op, metadata, TransferOpMeta);

                    if let Some(r) = &related_op {
                        let x = &r.operation_identifier;
                        let index = x.index;
                        let main_amount = op.amount();
                        let main_address = op.address();

                        let amount = r.amount();
                        let address = r.address();

                        if amount < 0 {
                            //negative = this is sender
                            metadata.source = Some(address);
                            metadata.destination = Some(main_address.clone());
                            metadata.amount = Some(main_amount as u64);
                        } else {
                            metadata.destination = Some(address);
                            metadata.source = Some(main_address.clone());
                            metadata.amount = Some(amount as u64);
                        }
                        checked_related_op_indexes.push(index);
                    }
                    if let Some(x) = &op.amount {
                        let currency = &x.currency;
                        metadata.decimals = Some(currency.decimals);
                        metadata.mint = Some(currency.symbol.clone()); //TODO: Symbol = mint address
                    };
                    push_op!(op, metadata, combined_operations);
                }
                _ => combined_operations.push(op.clone()),
            }
        }
    });

    Ok(combined_operations)
}
//TODO: Write all input Json-> Instructions required here
impl Operation {
    pub fn amount(&self) -> i64 {
        self.amount.clone().unwrap().value.parse::<i64>().unwrap()
    }
    pub fn address(&self) -> String {
        self.account.clone().unwrap().address
    }
    pub fn to_instruction(&self) -> Result<Instruction, ApiError> {
        let instruction = match self.type_ {
            OperationType::System__Transfer => {
                let metadata = TransferOpMeta::from(&self.metadata);
                system_instruction::transfer(
                    &to_pub(&metadata.source.unwrap()),
                    &to_pub(&metadata.destination.unwrap()),
                    metadata.amount.unwrap(),
                )
            }
            OperationType::SplToken__CreateAssocAccount => {
                let metadata = TransferOpMeta::from(&self.metadata);

                spl_associated_token_account::create_associated_token_account(
                    &to_pub(&metadata.source.clone().unwrap()),
                    &to_pub(&metadata.source.unwrap()),
                    &to_pub(&metadata.mint.unwrap()),
                )
            }
            OperationType::SplToken__TransferChecked => {
                let metadata = TransferOpMeta::from(&self.metadata);

                spl_token::instruction::transfer_checked(
                    &spl_token::id(),
                    &to_pub(&metadata.source.unwrap()),
                    &to_pub(&metadata.mint.unwrap()),
                    &to_pub(&metadata.destination.unwrap()),
                    &to_pub(&metadata.authority.unwrap()),
                    &vec![],
                    metadata.amount.unwrap(),
                    metadata.decimals.unwrap(),
                )
                .unwrap()
            }
            OperationType::SplToken__Transfer => {
                #[derive(Deserialize)]
                struct Metadata {
                    ///owner of sender address
                    authority: String,
                    ///sender token wallet address
                    source: String,
                    ///destination token wallet address
                    destination: String,
                    ///amount in tokens
                    amount: u64,
                };

                let metadata =
                    serde_json::from_value::<Metadata>(self.metadata.clone().unwrap()).unwrap();
                spl_token::instruction::transfer(
                    &spl_token::id(),
                    &to_pub(&metadata.source),
                    &to_pub(&metadata.destination),
                    &to_pub(&metadata.authority),
                    &vec![],
                    metadata.amount,
                )
                .unwrap()
            }
            OperationType::Unknown => {
                return Err(ApiError::BadOperations(
                    "Operation Not Supported".to_string(),
                ))
            }
        };
        Ok(instruction)
    }
}
pub fn get_operations_from_encoded_tx(
    transaction: &EncodedTransaction,
    status: Option<OperationStatusType>,
) -> (Vec<Operation>, String) {
    let mut operations = vec![];
    let mut tx_hash = String::from("");

    let mut op_index = 0 as u64;
    if let EncodedTransaction::Json(t) = &transaction {
        tx_hash = t.signatures[0].to_string();
        if let UiMessage::Parsed(m) = &t.message {
            m.instructions.iter().for_each(|instruction| {
                let oi = OperationIdentifier {
                    index: op_index as u64,
                    network_index: None,
                };
                if let UiInstruction::Parsed(ui_parsed_instruction) = &instruction {
                    match &ui_parsed_instruction {
                        UiParsedInstruction::Parsed(parsed_instruction) => {
                            let parsed_instruction_enum: ParsedInstructionEnum =
                                serde_json::from_value(parsed_instruction.parsed.clone())
                                    .unwrap_or(ParsedInstructionEnum {
                                        instruction_type: "Unknown".to_string(),
                                        info: serde_json::Value::Null,
                                    });
                            let optype = get_operation_type_with_program(
                                &parsed_instruction.program,
                                &parsed_instruction_enum.instruction_type,
                            );
                            let metadata = parsed_instruction_enum.info;
                            match &optype {
                                OperationType::System__Transfer
                                | OperationType::SplToken__Transfer
                                | OperationType::SplToken__TransferChecked => {
                                    let parsed_meta = TransferOpMeta::from(&Some(metadata.clone()));
                                    let parsed_meta_cloned = parsed_meta.clone();
                                    let currency = Currency {
                                        symbol: parsed_meta
                                            .mint
                                            .unwrap_or(consts::NATIVE_SYMBOL.to_string()),
                                        decimals: parsed_meta
                                            .decimals
                                            .unwrap_or(consts::NATIVE_DECIMALS),
                                    };
                                    let sender = Some(AccountIdentifier {
                                        address: parsed_meta.source.unwrap(),
                                        sub_account: None,
                                    });
                                    let sender_amt = Some(Amount {
                                        value: format!("-{}", parsed_meta_cloned.amount()),
                                        currency: currency.clone(),
                                    });
                                    let receiver = Some(AccountIdentifier {
                                        address: parsed_meta.destination.unwrap(),
                                        sub_account: None,
                                    });
                                    let receiver_amt = Some(Amount {
                                        value: parsed_meta_cloned.amount(),
                                        currency: currency,
                                    });

                                    op_index += 1;
                                    let oi2 = OperationIdentifier {
                                        index: (op_index) as u64,
                                        network_index: None,
                                    };
                                    //sender push
                                    operations.push(Operation {
                                        operation_identifier: oi.clone(),
                                        related_operations: None,
                                        type_: optype.clone(),
                                        status: status.clone(), //TODO: sucess/faliure for now
                                        account: sender,
                                        amount: sender_amt,
                                        metadata: Some(metadata.clone()),
                                    });
                                    //receiver push
                                    operations.push(Operation {
                                        operation_identifier: oi2,
                                        related_operations: None,
                                        type_: optype,
                                        status: status.clone(), //TODO: sucess/faliure for now
                                        account: receiver,
                                        amount: receiver_amt,
                                        metadata: Some(metadata),
                                    });
                                }
                                _ => {
                                    //TODO: See metadata in other op types and put here accordingly
                                    operations.push(Operation {
                                        operation_identifier: oi,
                                        related_operations: None,
                                        type_: optype,
                                        status: status.clone(), //TODO: sucess/faliure for now
                                        account: None,
                                        amount: None,
                                        metadata: Some(metadata),
                                    });
                                }
                            };
                        }
                        UiParsedInstruction::PartiallyDecoded(partially_decoded_instruction) => {
                            operations.push(Operation {
                                operation_identifier: oi,
                                related_operations: None,
                                type_: get_operation_type("Unknown"),
                                status: status.clone(), //TODO: sucess/faliure for now
                                account: None,
                                amount: None,
                                metadata: Some(
                                    serde_json::to_value(&partially_decoded_instruction).unwrap(),
                                ),
                            });
                        }
                    }
                }
                op_index += 1;
            })
        }
    }
    (operations, tx_hash)
}

#[cfg(test)]
mod tests {
    use crate::types::*;
    use serde_json::json;

    use super::*;
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
        assert_eq!(ops[0].metadata.clone().unwrap().to_string(),"{\"amount\":1000,\"decimals\":10,\"destination\":\"DestinationAddress\",\"mint\":\"TEST\",\"source\":\"SenderAddress\"}");
        assert_eq!(ops[1].metadata.clone().unwrap().to_string(),"{\"amount\":10000,\"destination\":\"SomeUnrelatedDest\",\"source\":\"SomeUnrelatedSender\"}");
        assert_eq!(ops[2].metadata.clone().unwrap().to_string(),"{\"amount\":10,\"authority\":\"AA\",\"decimals\":2,\"destination\":\"DD\",\"mint\":\"MM\",\"source\":\"SS\"}");
    }
}
