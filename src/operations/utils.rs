use crate::{
    consts,
    error::ApiError,
    types::AccountIdentifier,
    types::{Amount, Currency, Operation, OperationIdentifier, OperationStatusType, OperationType},
    utils::get_operation_type,
    utils::get_operation_type_with_program,
};

use super::OpMeta;
use solana_transaction_status::{
    parse_instruction::ParsedInstructionEnum, EncodedTransaction, UiInstruction, UiMessage,
    UiParsedInstruction,
};

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
                                    let parsed_meta = OpMeta::from(&Some(metadata.clone()));
                                    let parsed_meta_cloned = parsed_meta.clone();
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
                                    //sender push
                                    operations.push(Operation {
                                        operation_identifier: oi.clone(),
                                        related_operations: None,
                                        type_: optype.clone(),
                                        status: status.clone(),
                                        account: sender,
                                        amount: sender_amt,
                                        metadata: Some(metadata.clone()),
                                    });
                                    //receiver push
                                    operations.push(Operation {
                                        operation_identifier: oi2,
                                        related_operations: None,
                                        type_: optype,
                                        status: status.clone(),
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
                                        status: status.clone(),
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
