use crate::{
    consts,
    error::ApiError,
    types::AccountIdentifier,
    types::{Amount, Currency, Operation, OperationIdentifier, OperationStatusType, OperationType},
    utils::get_operation_type,
    utils::get_operation_type_with_program,
};

use solana_transaction_status::{
    parse_instruction::ParsedInstructionEnum, EncodedTransaction, UiInstruction, UiMessage,
    UiParsedInstruction,
};

use super::TransferOpMeta;

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
        if let Some(x) = &$op.metadata {
            $metadata = serde_json::from_value::<$metastruct>(x.clone()).unwrap();
        } else {
            $metadata = $metastruct::default()
        };
    };
}

//simple matcher
pub fn combine_related_operations(operations: &Vec<Operation>) -> Result<Vec<Operation>, ApiError> {
    let mut combined_operations: Vec<Operation> = vec![];
    let mut checked_related_op_indexes: Vec<u64> = vec![];
    operations.iter().for_each(|op| {
        if !checked_related_op_indexes.contains(&op.operation_identifier.index) {
            let mut metadata: TransferOpMeta;
            get_meta!(op, metadata, TransferOpMeta);

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
                            metadata.amount = Some(main_amount.to_string());
                            metadata.lamports = Some(main_amount as u64);
                        } else {
                            metadata.destination = Some(address);
                            metadata.source = Some(main_address.clone());
                            metadata.amount = Some(amount.to_string());
                            metadata.lamports = Some(amount as u64);
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
                _ => {
                    if let Some(x) = &op.account {
                        metadata.source = Some(x.address.clone());
                    };
                    push_op!(op, metadata, combined_operations);
                }
            }
        }
    });

    Ok(combined_operations)
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
