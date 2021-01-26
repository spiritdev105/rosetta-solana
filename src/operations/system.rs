use serde::{Deserialize, Serialize};
use solana_sdk::{instruction::Instruction, system_instruction};

use crate::{error::ApiError, types::OperationType, utils::to_pub};

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct SystemOperationMetadata {
    source: Option<String>,
    destination: Option<String>,
    space: Option<u64>,
    lamports: Option<u64>,
    authority: Option<String>,
    new_authority: Option<String>,
}
pub fn to_instruction(
    type_: OperationType,
    metadata: SystemOperationMetadata,
) -> Result<Vec<Instruction>, ApiError> {
    let instructions = match type_ {
        OperationType::System__CreateAccount => vec![system_instruction::create_account(
            &to_pub(&metadata.source.unwrap()),
            &to_pub(&metadata.destination.unwrap()),
            metadata.lamports.unwrap(),
            metadata.space.unwrap(),
            &spl_token::id(),
        )],
        OperationType::System__Assign => vec![system_instruction::assign(
            &to_pub(&metadata.source.unwrap()),
            &spl_token::id(),
        )],
        OperationType::System__Transfer => vec![system_instruction::transfer(
            &to_pub(&metadata.source.unwrap()),
            &to_pub(&metadata.destination.unwrap()),
            metadata.lamports.unwrap(),
        )],
        OperationType::System__CreateNonceAccount => system_instruction::create_nonce_account(
            &to_pub(&metadata.source.unwrap()),
            &to_pub(&metadata.destination.unwrap()),
            &to_pub(&metadata.authority.unwrap()),
            metadata.lamports.unwrap_or(1000000000),
        ),

        OperationType::System__AdvanceNonceAccount => {
            vec![system_instruction::advance_nonce_account(
                &to_pub(&metadata.destination.unwrap()),
                &to_pub(&metadata.authority.unwrap()),
            )]
        }
        OperationType::System__WithdrawNonceAccount => {
            vec![system_instruction::withdraw_nonce_account(
                &to_pub(&metadata.source.unwrap()),
                &to_pub(&metadata.authority.unwrap()),
                &to_pub(&metadata.destination.unwrap()),
                metadata.lamports.unwrap(),
            )]
        }
        OperationType::System__AuthorizeNonceAccount => {
            vec![system_instruction::authorize_nonce_account(
                &to_pub(&metadata.destination.unwrap()),
                &to_pub(&metadata.authority.unwrap()),
                &to_pub(&metadata.new_authority.unwrap()),
            )]
        }
        OperationType::System__Allocate => vec![system_instruction::allocate(
            &to_pub(&metadata.source.unwrap()),
            metadata.space.unwrap(),
        )],
        _ => {
            return Err(ApiError::BadOperations("Invalid Operation".to_string()));
        }
    };
    Ok(instructions)
}
