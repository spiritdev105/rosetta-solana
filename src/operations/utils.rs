use crate::{
    consts,
    types::AccountIdentifier,
    types::{Amount, Currency, Operation, OperationIdentifier, OperationStatusType, OperationType},
    utils::get_operation_type,
    utils::get_operation_type_with_program,
};

use solana_transaction_status::{
    parse_instruction::ParsedInstructionEnum, EncodedTransaction, UiInstruction, UiMessage,
    UiParsedInstruction,
};

use serde::{Deserialize, Serialize};

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
struct OpMeta {
    ///owner of sender address
    #[serde(skip_serializing_if = "Option::is_none", alias = "custodian")]
    authority: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    new_authority: Option<String>,
    ///sender token wallet address
    #[serde(
        skip_serializing_if = "Option::is_none",
        alias = "nonceAccount",
        alias = "stakeAccount",
        alias = "voteAccount"
    )]
    source: Option<String>,
    ///destination token wallet address
    #[serde(
        skip_serializing_if = "Option::is_none",
        alias = "newAccount",
        alias = "newSplitAccount"
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

#[macro_export]
macro_rules! set_meta {
    ($metadata:expr, $metastruct:ident) => {
        if let Some(x) = $metadata {
            serde_json::from_value::<$metastruct>(x).unwrap()
        } else {
            $metastruct::default()
        }
    };
}
#[macro_export]
macro_rules! merge_meta {
    ($metadata:expr, $defmeta:expr, $i:expr, $enum:ident) => {
        if let Some(x) = $defmeta {
            if let Some(y) = &x[$i] {
                match y {
                    InternalOperationMetadata::$enum(x) => {
                        $metadata.merge(x.clone());
                    }
                    _ => {}
                }
            }
        };
    };
}
//Convert tx recieved from json rpc  to rosetta style operations
//TODO: find All balance change scenarios and should be converted to -ve +ve balance change operations

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
                            if optype.is_balance_changing() {
                                let parsed_meta = OpMeta::from(&Some(metadata.clone()));
                                let mut parsed_meta_cloned = parsed_meta.clone();
                                let currency = Currency {
                                    symbol: parsed_meta
                                        .mint
                                        .unwrap_or(consts::NATIVE_SYMBOL.to_string()),
                                    decimals: parsed_meta
                                        .decimals
                                        .unwrap_or(consts::NATIVE_DECIMALS),
                                    metadata: None,
                                };
                                let sender = Some(AccountIdentifier {
                                    address: parsed_meta.source.unwrap(),
                                    sub_account: None,
                                });
                                let sender_amt = Some(Amount {
                                    value: format!("-{}", parsed_meta_cloned.amount_str()),
                                    currency: currency.clone(),
                                });
                                let receiver = Some(AccountIdentifier {
                                    address: parsed_meta.destination.unwrap(),
                                    sub_account: None,
                                });
                                let receiver_amt = Some(Amount {
                                    value: parsed_meta_cloned.amount_str(),
                                    currency: currency,
                                });

                                op_index += 1;
                                let oi2 = OperationIdentifier {
                                    index: (op_index) as u64,
                                    network_index: None,
                                };

                                //for construction test
                                parsed_meta_cloned.amount = None;
                                parsed_meta_cloned.lamports = None;
                                parsed_meta_cloned.source = None;
                                parsed_meta_cloned.destination = None;

                                let res = serde_json::to_value(parsed_meta_cloned).unwrap();
                                let (metasend, metarec) = if res.as_object().unwrap().is_empty() {
                                    (None, None)
                                } else {
                                    (Some(res.clone()), Some(res))
                                };
                                //sender push
                                operations.push(Operation {
                                    operation_identifier: oi.clone(),
                                    related_operations: None,
                                    type_: optype.clone(),
                                    status: status.clone(),
                                    account: sender,
                                    amount: sender_amt,
                                    metadata: metasend,
                                });
                                //receiver push
                                operations.push(Operation {
                                    operation_identifier: oi2,
                                    related_operations: None,
                                    type_: optype,
                                    status: status.clone(),
                                    account: receiver,
                                    amount: receiver_amt,
                                    metadata: metarec,
                                });
                            } else {
                                //TODO: See metadata in other op types and put here accordingly
                                operations.push(Operation {
                                    operation_identifier: oi,
                                    related_operations: None,
                                    type_: optype,
                                    status: status.clone(),
                                    account: None,
                                    amount: None,
                                    metadata: Some(metadata),
                                });
                            }
                        }
                        UiParsedInstruction::PartiallyDecoded(partially_decoded_instruction) => {
                            operations.push(Operation {
                                operation_identifier: oi,
                                related_operations: None,
                                type_: get_operation_type("Unknown"),
                                status: status.clone(),
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
