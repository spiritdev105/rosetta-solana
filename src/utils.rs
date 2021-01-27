use std::str::FromStr;

use crate::{
    consts,
    operations::utils::get_operations_from_encoded_tx,
    types::{OperationStatusType, OperationType, Transaction, TransactionIdentifier},
};
use convert_case::{Case, Casing};

use solana_sdk::pubkey::Pubkey;
use solana_transaction_status::EncodedTransactionWithStatusMeta;

impl From<&EncodedTransactionWithStatusMeta> for Transaction {
    fn from(x: &EncodedTransactionWithStatusMeta) -> Self {
        let mut status = OperationStatusType::Success;
        if let Some(x) = &x.meta {
            if let Some(_) = &x.err {
                status = OperationStatusType::Faliure;
            }
        };
        let (operations, tx_hash) = get_operations_from_encoded_tx(&x.transaction, Some(status));
        Transaction {
            transaction_identifier: TransactionIdentifier { hash: tx_hash },
            metadata: x.meta.clone(),
            operations: operations,
        }
    }
}
pub fn to_pub(s: &str) -> Pubkey {
    Pubkey::from_str(&s).unwrap()
}
pub fn to_pub_optional(s: Option<String>) -> Option<Pubkey> {
    if let Some(x) = &s {
        Some(Pubkey::from_str(x).unwrap())
    } else {
        None
    }
}

pub fn get_operation_type_with_program(program: &str, s: &str) -> OperationType {
    let to_pascal = program.to_case(Case::Pascal);

    let newstr = format!(
        "{}{}{}",
        to_pascal,
        consts::SEPARATOR,
        s.to_case(Case::Pascal)
    );
    OperationType::from_str(&newstr).unwrap_or(OperationType::Unknown)
}
pub fn get_operation_type(s: &str) -> OperationType {
    let x = s.split(consts::SEPARATOR).collect::<Vec<&str>>();
    if x.len() < 2 {
        return OperationType::Unknown;
    }
    get_operation_type_with_program(x[0], x[1])
}

#[cfg(test)]
mod tests {
    use crate::types::*;
    use serde_json::json;

    use super::*;
    #[test]
    fn op_type_test() {
        assert_eq!(
            get_operation_type("spl-token__transfer"),
            OperationType::SplToken__Transfer
        );
        assert_eq!(
            get_operation_type_with_program("spl-token", "transfer"),
            OperationType::SplToken__Transfer
        );
        assert_eq!(
            get_operation_type_with_program("spl-token", "transferChecked"),
            OperationType::SplToken__TransferChecked
        );
        assert_eq!(
            get_operation_type_with_program("system", "withdrawFromNonce"),
            OperationType::System__WithdrawFromNonce
        );
        assert_eq!(
            get_operation_type("system__transfer"),
            OperationType::System__Transfer
        );
        assert_eq!(get_operation_type("Invalid"), OperationType::Unknown);
    }
}
