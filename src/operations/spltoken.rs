use crate::{error::ApiError, types::OperationType, utils::to_pub};
use merge::Merge;
use serde::{Deserialize, Serialize};
use solana_sdk::{instruction::Instruction, program_pack::Pack, system_instruction};
use spl_token::state::Mint;

const MIN_RENT: u64 = 100000;
#[derive(Merge, Default, Clone, Debug, Deserialize, Serialize)]
pub struct SplTokenOperationMetadata {
    pub source: Option<String>,
    pub destination: Option<String>,
    pub mint: Option<String>,
    pub authority: Option<String>,
    pub freeze_authority: Option<String>,
    pub amount: Option<u64>,
    pub decimals: Option<u8>,
}

pub fn to_instruction(
    type_: OperationType,
    metadata: SplTokenOperationMetadata,
) -> Result<Vec<Instruction>, ApiError> {
    let p;
    let freeze_authority = if let Some(f) = &metadata.freeze_authority {
        p = to_pub(f);
        Some(&p)
    } else {
        None
    };
    let source = if let Some(s) = &metadata.source {
        to_pub(s)
    } else {
        return Err(ApiError::PlaceHolderError("Source missing".to_string()));
    };
    let instruction = match type_ {
        OperationType::SplToken__InitializeMint => vec![spl_token::instruction::initialize_mint(
            &spl_token::id(),
            &to_pub(&metadata.mint.unwrap()),
            &to_pub(&metadata.source.unwrap()),
            freeze_authority,
            metadata.decimals.unwrap(),
        )?],
        OperationType::SplToken__InitializeAccount => {
            vec![spl_token::instruction::initialize_account(
                &spl_token::id(),
                &to_pub(&metadata.destination.unwrap()),
                &to_pub(&metadata.mint.unwrap()),
                &source,
            )?]
        }
        OperationType::SplToken__CreateToken => vec![
            system_instruction::create_account(
                &source,
                &to_pub(&metadata.mint.clone().unwrap()),
                metadata.amount.unwrap(),
                Mint::LEN as u64,
                &spl_token::id(),
            ),
            spl_token::instruction::initialize_mint(
                &spl_token::id(),
                &to_pub(&metadata.mint.unwrap()),
                &to_pub(&metadata.authority.unwrap()),
                freeze_authority,
                metadata.decimals.unwrap_or(2),
            )?,
        ],
        OperationType::SplToken__CreateAccount => vec![
            system_instruction::create_account(
                &source,
                &to_pub(&metadata.destination.clone().unwrap()),
                metadata.amount.unwrap(),
                spl_token::state::Account::LEN as u64,
                &spl_token::id(),
            ),
            spl_token::instruction::initialize_account(
                &spl_token::id(),
                &to_pub(&metadata.destination.unwrap()),
                &to_pub(&metadata.mint.unwrap()),
                &to_pub(&metadata.authority.unwrap()),
            )?,
        ],
        OperationType::SplToken__Approve => vec![spl_token::instruction::approve(
            &spl_token::id(),
            &source,
            &to_pub(&metadata.destination.unwrap()),
            &to_pub(&metadata.authority.unwrap()),
            &vec![],
            metadata.amount.unwrap(),
        )?],
        OperationType::SplToken__Revoke => vec![spl_token::instruction::revoke(
            &spl_token::id(),
            &source,
            &to_pub(&metadata.authority.unwrap()),
            &vec![],
        )?],

        OperationType::SplToken__MintTo => vec![spl_token::instruction::mint_to(
            &spl_token::id(),
            &to_pub(&metadata.mint.unwrap()),
            &source,
            &to_pub(&metadata.authority.unwrap()),
            &vec![],
            metadata.amount.unwrap(),
        )?],

        OperationType::SplToken__Burn => vec![spl_token::instruction::burn(
            &spl_token::id(),
            &source,
            &to_pub(&metadata.mint.unwrap()),
            &source,
            &vec![],
            metadata.amount.unwrap(),
        )?],
        OperationType::SplToken__CloseAccount => vec![spl_token::instruction::close_account(
            &spl_token::id(),
            &source,
            &to_pub(&metadata.authority.clone().unwrap()),
            &to_pub(&metadata.authority.unwrap()),
            &vec![],
        )?],
        OperationType::SplToken__FreezeAccount => vec![spl_token::instruction::freeze_account(
            &spl_token::id(),
            &source,
            &to_pub(&metadata.mint.clone().unwrap()),
            &to_pub(&metadata.authority.unwrap()),
            &vec![],
        )?],
        OperationType::SplToken__ThawAccount => vec![spl_token::instruction::thaw_account(
            &spl_token::id(),
            &source,
            &to_pub(&metadata.mint.clone().unwrap()),
            &to_pub(&metadata.authority.unwrap()),
            &vec![],
        )?],
        OperationType::SplToken__CreateAssocAccount => vec![
            spl_associated_token_account::create_associated_token_account(
                &source,
                &source,
                &to_pub(&metadata.mint.unwrap()),
            ),
        ],

        OperationType::SplToken__TransferChecked => vec![spl_token::instruction::transfer_checked(
            &spl_token::id(),
            &source,
            &to_pub(&metadata.mint.unwrap()),
            &to_pub(&metadata.destination.unwrap()),
            &to_pub(&metadata.authority.unwrap()),
            &vec![],
            metadata.amount.unwrap(),
            metadata.decimals.unwrap(),
        )?],
        OperationType::SplToken__Transfer => vec![spl_token::instruction::transfer(
            &spl_token::id(),
            &source,
            &to_pub(&metadata.destination.unwrap()),
            &to_pub(&metadata.authority.unwrap()),
            &vec![],
            metadata.amount.unwrap(),
        )?],
        _ => {
            return Err(ApiError::BadOperations("Invalid Operation".to_string()));
        }
    };
    Ok(instruction)
}
