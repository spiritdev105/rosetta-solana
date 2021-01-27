use crate::{error::ApiError, types::OperationType, utils::to_pub};
use merge::Merge;
use serde::{Deserialize, Serialize};
use solana_sdk::instruction::Instruction;
use solana_vote_program::{
    vote_instruction,
    vote_state::{VoteAuthorize, VoteInit},
};

use super::matcher::InternalOperation;

#[derive(Merge, Default, Clone, Debug, Deserialize, Serialize)]
pub struct VoteOperationMetadata {
    pub source: Option<String>,
    pub destination: Option<String>,
    pub lamports: Option<u64>,
    pub authority: Option<String>,
    pub voter: Option<String>,
    pub withdrawer: Option<String>,
    pub vote_pubkey: Option<String>,
    pub comission: Option<u8>,
}
pub fn to_instruction(
    type_: OperationType,
    metadata: VoteOperationMetadata,
) -> Result<Vec<Instruction>, ApiError> {
    let instructions = match type_ {
        OperationType::Vote__CreateAccount => {
            let authority = to_pub(&metadata.authority.unwrap());
            //TODO: Add in meta later
            let vote_init = VoteInit {
                node_pubkey: authority.clone(),
                authorized_voter: authority.clone(),
                authorized_withdrawer: authority.clone(),
                commission: 100,
            };
            vote_instruction::create_account(
                &to_pub(&metadata.source.unwrap()),
                &to_pub(&metadata.destination.unwrap()),
                &vote_init,
                metadata.lamports.unwrap(),
            )
        }
        OperationType::Vote__Authorize => {
            let mut inx = vec![];
            if let Some(x) = &metadata.voter {
                inx.push(vote_instruction::authorize(
                    &to_pub(&metadata.destination.clone().unwrap()),
                    &to_pub(&metadata.source.clone().unwrap()),
                    &to_pub(x),
                    VoteAuthorize::Voter,
                ))
            }
            if let Some(x) = &metadata.withdrawer {
                inx.push(vote_instruction::authorize(
                    &to_pub(&metadata.destination.unwrap()),
                    &to_pub(&metadata.source.unwrap()),
                    &to_pub(x),
                    VoteAuthorize::Withdrawer,
                ))
            }
            inx
        }
        OperationType::Vote__Withdraw => vec![vote_instruction::withdraw(
            &to_pub(&metadata.source.unwrap()),
            &to_pub(&metadata.authority.unwrap()),
            metadata.lamports.unwrap(),
            &to_pub(&metadata.destination.unwrap()),
        )],
        OperationType::Vote__UpdateValidatorIdentity => {
            vec![vote_instruction::update_validator_identity(
                &to_pub(&metadata.vote_pubkey.unwrap()),
                &to_pub(&metadata.withdrawer.unwrap()),
                &to_pub(&metadata.voter.unwrap()),
            )]
        }
        OperationType::Vote__UpdateCommission => vec![vote_instruction::update_commission(
            &to_pub(&metadata.vote_pubkey.unwrap()),
            &to_pub(&metadata.withdrawer.unwrap()),
            metadata.comission.unwrap(),
        )],
        _ => {
            return Err(ApiError::BadOperations("Invalid Operation".to_string()));
        }
    };
    Ok(instructions)
}
