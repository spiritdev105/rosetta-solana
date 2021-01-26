use serde::{Deserialize, Serialize};
use solana_sdk::instruction::Instruction;
use solana_stake_program::{
    stake_instruction,
    stake_instruction::LockupArgs,
    stake_state::{Lockup, StakeAuthorize},
};

use crate::{error::ApiError, types::OperationType, utils::to_pub};

#[derive(Default, Clone, Debug, Deserialize, Serialize)]
pub struct StakeOperationMetadata {
    source: Option<String>,
    #[serde(alias = "stake_pubkey")]
    destination: Option<String>,
    lamports: Option<u64>,
    authority: Option<String>,
    staker: Option<String>,
    withdrawer: Option<String>,
    vote_pubkey: Option<String>,
    lockup: Option<LockupMeta>,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct StakeOperation {
    #[serde(rename = "type")]
    pub type_: OperationType,
    pub metadata: StakeOperationMetadata,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct LockupMeta {
    /// UnixTimestamp at which this stake will allow withdrawal, unless the
    ///   transaction is signed by the custodian
    pub unix_timestamp: Option<solana_sdk::clock::UnixTimestamp>,
    /// epoch height at which this stake will allow withdrawal, unless the
    ///   transaction is signed by the custodian
    pub epoch: Option<solana_sdk::clock::Epoch>,
    /// custodian signature on a transaction exempts the operation from
    ///  lockup constraints
    pub custodian: Option<String>,
}
impl From<LockupMeta> for Lockup {
    fn from(meta: LockupMeta) -> Self {
        Lockup {
            unix_timestamp: meta.unix_timestamp.unwrap_or(0),
            epoch: meta.epoch.unwrap_or(0),
            custodian: to_pub(
                &meta
                    .custodian
                    .unwrap_or("11111111111111111111111111111111".to_string()), //TODO: Find this
            ),
        }
    }
}
impl From<LockupMeta> for LockupArgs {
    fn from(meta: LockupMeta) -> Self {
        let new_custodian = if let Some(x) = meta.custodian {
            Some(to_pub(&x))
        } else {
            None
        };
        LockupArgs {
            unix_timestamp: meta.unix_timestamp,
            epoch: meta.epoch,
            custodian: new_custodian,
        }
    }
}
pub fn to_instruction(
    type_: OperationType,
    metadata: StakeOperationMetadata,
) -> Result<Vec<Instruction>, ApiError> {
    let instructions = match type_ {
        OperationType::Stake__CreateAccount => {
            let authorized = solana_stake_program::stake_state::Authorized {
                staker: to_pub(
                    &metadata
                        .staker
                        .unwrap_or(metadata.source.clone().unwrap().clone()),
                ),
                withdrawer: to_pub(
                    &metadata
                        .withdrawer
                        .unwrap_or(metadata.source.clone().unwrap()),
                ),
            };
            stake_instruction::create_account(
                &to_pub(&metadata.source.unwrap()),
                &to_pub(&metadata.destination.unwrap()),
                &authorized,
                &Lockup::from(metadata.lockup.unwrap()),
                metadata.lamports.unwrap(),
            )
        }
        OperationType::Stake__Delegate => vec![stake_instruction::delegate_stake(
            &to_pub(&metadata.destination.unwrap()),
            &to_pub(&metadata.authority.unwrap_or(metadata.source.unwrap())),
            &to_pub(&metadata.vote_pubkey.unwrap()),
        )],
        OperationType::Stake__Split => stake_instruction::split(
            &to_pub(&metadata.source.unwrap()),
            &to_pub(&metadata.authority.unwrap()),
            metadata.lamports.unwrap(),
            &to_pub(&metadata.destination.unwrap()),
        ),
        OperationType::Stake__Merge => stake_instruction::merge(
            &to_pub(&metadata.destination.unwrap()),
            &to_pub(&metadata.source.unwrap()),
            &to_pub(&metadata.authority.unwrap()),
        ),
        OperationType::Stake__Authorize => {
            let mut inx = vec![];
            if let Some(x) = &metadata.staker {
                inx.push(stake_instruction::authorize(
                    &to_pub(&metadata.destination.clone().unwrap()),
                    &to_pub(&metadata.source.clone().unwrap()),
                    &to_pub(x),
                    StakeAuthorize::Staker,
                ))
            }
            if let Some(x) = &metadata.withdrawer {
                inx.push(stake_instruction::authorize(
                    &to_pub(&metadata.destination.unwrap()),
                    &to_pub(&metadata.source.unwrap()),
                    &to_pub(x),
                    StakeAuthorize::Withdrawer,
                ))
            }
            inx
        }
        OperationType::Stake__Withdraw => vec![stake_instruction::withdraw(
            &to_pub(&metadata.source.unwrap()),
            &to_pub(&metadata.withdrawer.unwrap()),
            &to_pub(&metadata.destination.unwrap()),
            metadata.lamports.unwrap(),
            //to_pub_optional(metadata.custodian),
            None,
        )],
        OperationType::Stake__Deactivate => vec![stake_instruction::deactivate_stake(
            &to_pub(&metadata.destination.unwrap()),
            &to_pub(&metadata.authority.unwrap_or(metadata.source.unwrap())),
        )],
        OperationType::Stake__SetLockup => vec![stake_instruction::set_lockup(
            &to_pub(&metadata.destination.unwrap()),
            &LockupArgs::from(metadata.lockup.unwrap()),
            &to_pub(&metadata.authority.unwrap_or(metadata.source.unwrap())),
        )],
        _ => {
            return Err(ApiError::BadOperations("Invalid Operation".to_string()));
        }
    };
    Ok(instructions)
}
