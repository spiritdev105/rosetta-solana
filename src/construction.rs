use std::str::FromStr;

use crate::{
    error::ApiError,
    is_bad_network,
    operations::get_tx_from_str,
    operations::matcher::InternalOperationMetadata,
    operations::matcher::Matcher,
    operations::spltoken::SplTokenOperationMetadata,
    types::{OperationType, WithNonce},
    utils::to_pub,
    Options,
};
use crate::{
    operations::utils::get_operations_from_encoded_tx,
    types::{
        AccountIdentifier, ConstructionCombineRequest, ConstructionCombineResponse,
        ConstructionDeriveRequest, ConstructionDeriveResponse, ConstructionHashRequest,
        ConstructionMetadata, ConstructionMetadataRequest, ConstructionMetadataResponse,
        ConstructionParseRequest, ConstructionParseResponse, ConstructionPayloadsRequest,
        ConstructionPayloadsResponse, ConstructionPreprocessRequest,
        ConstructionPreprocessResponse, ConstructionSubmitRequest, ConstructionSubmitResponse,
        CurveType, MetadataOptions, SignatureType, SigningPayload, TransactionIdentifier,
        TransactionIdentifierResponse,
    },
};
use rocket_contrib::json::Json;
use serde::{Deserialize, Serialize};
use solana_account_decoder::{UiAccount, UiAccountData, UiAccountEncoding, UiFeeCalculator};
use solana_sdk::{
    fee_calculator::FeeCalculator, hash::Hash, message::Message, program_pack::Pack,
    pubkey::Pubkey, signature::Signature, transaction::Transaction,
};
use solana_transaction_status::{EncodedTransaction, UiMessage, UiTransactionEncoding};
use spl_token::state::{Account, Mint};

pub fn construction_derive(
    construction_derive_request: ConstructionDeriveRequest,
    options: &Options,
) -> Result<Json<ConstructionDeriveResponse>, ApiError> {
    is_bad_network(&options, &construction_derive_request.network_identifier)?;

    if construction_derive_request.public_key.curve_type != CurveType::Edwards25519 {
        return Err(ApiError::UnsupportedCurve);
    };
    let hex_pubkey = hex::decode(&construction_derive_request.public_key.hex_bytes)?;
    let bs58_pubkey = bs58::encode(hex_pubkey).into_string();

    let response = ConstructionDeriveResponse {
        account_identifier: AccountIdentifier {
            address: bs58_pubkey,
            sub_account: None,
        },
    };
    Ok(Json(response))
}
pub fn construction_hash(
    construction_hash_request: ConstructionHashRequest,
    options: &Options,
) -> Result<Json<TransactionIdentifierResponse>, ApiError> {
    is_bad_network(&options, &construction_hash_request.network_identifier)?;

    let tx = get_tx_from_str(&construction_hash_request.signed_transaction)?;
    let response = TransactionIdentifierResponse {
        transaction_identifier: TransactionIdentifier {
            hash: tx.signatures[0].to_string(),
        },
    };
    Ok(Json(response))
}
//Create Metadata Request to send to construction/metadata
pub fn construction_preprocess(
    construction_preprocess_request: ConstructionPreprocessRequest,
    options: &Options,
) -> Result<Json<ConstructionPreprocessResponse>, ApiError> {
    is_bad_network(
        &options,
        &construction_preprocess_request.network_identifier,
    )?;

    let mut matcher = Matcher::new(&construction_preprocess_request.operations, None);
    let internal_operations = matcher.combine()?;

    let with_nonce = if let Some(x) = construction_preprocess_request.metadata {
        x.with_nonce
    } else {
        None
    };
    let response = ConstructionPreprocessResponse {
        options: Some(MetadataOptions {
            internal_operations,
            with_nonce,
        }),
    };
    Ok(Json(response))
}
//Get recent blockhash and other metadata

pub fn construction_metadata(
    construction_metadata_request: ConstructionMetadataRequest,
    options: &Options,
) -> Result<Json<ConstructionMetadataResponse>, ApiError> {
    is_bad_network(&options, &construction_metadata_request.network_identifier)?;
    #[serde(rename_all = "camelCase")]
    #[derive(Default, Serialize, Deserialize, Clone, Debug)]
    struct Info {
        authority: String,
        blockhash: String,
        fee_calculator: UiFeeCalculator,
    }
    #[derive(Default, Serialize, Deserialize, Clone, Debug)]
    struct Parsed {
        info: Info,
    }
    //optional metadata for some special types
    let mut with_nonce = None;
    let mut parsed = Parsed::default();
    let internal_meta = if let Some(x) = &construction_metadata_request.options {
        if let Some(n) = &x.with_nonce {
            let pubkey = &to_pub(&n.account);
            let acc = options.rpc.get_account(pubkey)?;
            let uiacc = UiAccount::encode(pubkey, acc, UiAccountEncoding::JsonParsed, None, None);
            if let UiAccountData::Json(parsed_acc) = uiacc.data {
                parsed = serde_json::from_value::<Parsed>(parsed_acc.parsed).unwrap();
                with_nonce = Some(WithNonce {
                    account: n.account.clone(),
                    authority: Some(parsed.info.authority),
                });
            }
        };
        let ops = x
            .internal_operations
            .iter()
            .map(|x| match x.type_ {
                //TODO: Add more metadata as required
                OperationType::SplToken__CreateAccount => {
                    let rent = options
                        .rpc
                        .get_minimum_balance_for_rent_exemption(Account::LEN)
                        .unwrap();
                    Some(InternalOperationMetadata::SplToken(
                        SplTokenOperationMetadata {
                            amount: Some(rent),
                            ..Default::default()
                        },
                    ))
                }
                OperationType::SplToken__CreateToken => {
                    let rent = options
                        .rpc
                        .get_minimum_balance_for_rent_exemption(Mint::LEN)
                        .unwrap();
                    Some(InternalOperationMetadata::SplToken(
                        SplTokenOperationMetadata {
                            amount: Some(rent),
                            ..Default::default()
                        },
                    ))
                }
                _ => None,
            })
            .collect::<Vec<Option<InternalOperationMetadata>>>();
        Some(ops)
    } else {
        None
    };
    //required metadata
    let (hash, fee_calculator) = if with_nonce.is_none() {
        options.rpc.get_recent_blockhash()?
    } else {
        (
            Hash::from_str(&parsed.info.blockhash).unwrap(),
            FeeCalculator::new(
                parsed
                    .info
                    .fee_calculator
                    .lamports_per_signature
                    .parse::<u64>()
                    .unwrap(),
            ),
        )
    };
    let response = ConstructionMetadataResponse {
        metadata: ConstructionMetadata {
            blockhash: hash.to_string(),
            fee_calculator,
            internal_meta,
            with_nonce,
        },
    };
    Ok(Json(response))
}
//Construct Payloads to Sign

pub fn construction_payloads(
    construction_payloads_request: ConstructionPayloadsRequest,
    options: &Options,
) -> Result<Json<ConstructionPayloadsResponse>, ApiError> {
    is_bad_network(&options, &construction_payloads_request.network_identifier)?;

    let mut with_nonce = None;
    let meta = if let Some(x) = &construction_payloads_request.metadata {
        with_nonce = x.with_nonce.clone();
        if let Some(x) = &x.internal_meta {
            Some(x.clone())
        } else {
            None
        }
    } else {
        None
    };
    let mut matcher = Matcher::new(&construction_payloads_request.operations, meta);
    let instructions = matcher.to_instructions()?;
    let mut fee_payer = None;
    let mut fee_payer_pub = None;
    instructions.iter().for_each(|x| {
        if let Some(y) = x.accounts.iter().find(|a| a.is_signer) {
            fee_payer_pub = Some(y.pubkey.clone());
        }
    });
    if let Some(x) = &fee_payer_pub {
        fee_payer = Some(x);
    };

    let msg = if let Some(x) = with_nonce {
        Message::new_with_nonce(
            instructions,
            fee_payer,
            &to_pub(&x.account),
            &to_pub(&x.authority.unwrap()),
        )
    } else {
        Message::new(&instructions, fee_payer)
    };
    let mut tx = Transaction::new_unsigned(msg);
    //recent_blockhash is required as metadata
    if let Some(x) = &construction_payloads_request.metadata {
        let h = bs58::decode(&x.blockhash).into_vec().unwrap();
        tx.message.recent_blockhash = Hash::new(&h);
    } else {
        return Err(ApiError::BadTransactionPayload);
    }

    let v = bincode::serialize(&tx);
    if v.is_err() {
        return Err(ApiError::BadTransactionPayload);
    }
    let unsigned_transaction = bs58::encode(v.unwrap()).into_string();
    let to_be_signed = hex::encode(tx.message.serialize());
    let signing_payloads = tx
        .message
        .account_keys
        .iter()
        .enumerate()
        .map(|(i, pubk)| {
            if tx.message.is_signer(i) {
                Some(SigningPayload {
                    account_identifier: Some(AccountIdentifier {
                        address: bs58::encode(pubk.to_bytes()).into_string(),
                        sub_account: None,
                    }),
                    hex_bytes: to_be_signed.clone(),
                    signature_type: Some(SignatureType::Ed25519),
                })
            } else {
                None
            }
        })
        .take_while(|e| e.is_some())
        .collect::<Vec<Option<SigningPayload>>>();

    let response = ConstructionPayloadsResponse {
        unsigned_transaction,
        payloads: signing_payloads,
    };
    Ok(Json(response))
}

//Parse Unsigned Transaction to to Confirm Correctness

pub fn construction_parse(
    construction_parse_request: ConstructionParseRequest,
    options: &Options,
) -> Result<Json<ConstructionParseResponse>, ApiError> {
    is_bad_network(&options, &construction_parse_request.network_identifier)?;

    let tx = get_tx_from_str(&construction_parse_request.transaction)?;
    let encoded_tx = EncodedTransaction::encode(tx, UiTransactionEncoding::JsonParsed);
    let mut signers: Vec<AccountIdentifier> = vec![];
    if construction_parse_request.signed {
        if let EncodedTransaction::Json(t) = &encoded_tx {
            if let UiMessage::Parsed(m) = &t.message {
                m.account_keys.iter().for_each(|x| {
                    if x.signer == true {
                        signers.push(AccountIdentifier {
                            address: x.pubkey.to_string(),
                            sub_account: None,
                        });
                    }
                });
            }
        }
    }
    let account_identifier_signers = if signers.len() == 0 {
        None
    } else {
        Some(signers)
    };
    let (operations, _) = get_operations_from_encoded_tx(&encoded_tx, None);
    let response = ConstructionParseResponse {
        operations: operations,
        account_identifier_signers,
    };
    Ok(Json(response))
}

//combine sign

pub fn construction_combine(
    construction_combine_request: ConstructionCombineRequest,
    options: &Options,
) -> Result<Json<ConstructionCombineResponse>, ApiError> {
    is_bad_network(&options, &construction_combine_request.network_identifier)?;

    let mut tx = get_tx_from_str(&construction_combine_request.unsigned_transaction)?;
    let pubkeys = construction_combine_request
        .signatures
        .iter()
        .map(|x| {
            let p = hex::decode(&x.public_key.hex_bytes).unwrap();
            Pubkey::new(&p)
        })
        .collect::<Vec<Pubkey>>();
    let positions = tx
        .get_signing_keypair_positions(pubkeys.as_slice())
        .unwrap();
    for i in 0..positions.len() {
        tx.signatures[positions[i].unwrap()] = Signature::new(&hex::decode(
            &construction_combine_request.signatures[i].hex_bytes,
        )?);
    }
    let v = bincode::serialize(&tx);
    if v.is_err() {
        return Err(ApiError::BadTransactionPayload);
    }
    let response = ConstructionCombineResponse {
        signed_transaction: bs58::encode(v.unwrap()).into_string(),
    };
    Ok(Json(response))
}

//broadcast signed tx

pub fn construction_submit(
    construction_submit_request: ConstructionSubmitRequest,
    options: &Options,
) -> Result<Json<ConstructionSubmitResponse>, ApiError> {
    is_bad_network(&options, &construction_submit_request.network_identifier)?;
    let tx = get_tx_from_str(&construction_submit_request.signed_transaction)?;
    let signatureres = options.rpc.send_transaction(&tx);
    let signature = signatureres?;

    let response = ConstructionSubmitResponse {
        transaction_identifier: TransactionIdentifier {
            hash: signature.to_string(),
        },
    };
    Ok(Json(response))
}
#[cfg(test)]
mod tests {
    use std::{thread, time::Duration};

    use ed25519_dalek::*;
    use serde_json::json;

    use crate::{consts, create_rpc_client, types::*};

    //live debug tests on devnet
    //TODO: remove hardcoded keys

    use super::*;

    fn source() -> String {
        "HJGPMwVuqrbm7BDMeA3shLkqdHUru337fgytM7HzqTnH".to_string()
    }
    fn dest() -> String {
        "CgVKbBwogjaqtGtPLkMBSkhwtkTMLVdSdHM5cWzyxT5n".to_string()
    }

    fn main_account_keypair() -> Keypair {
        let privkey =
            hex::decode("cb1a134c296fbf309d78fe9378c18bc129e5045fbe92d2ad8577ccc84689d4ef")
                .unwrap();
        let public =
            hex::decode("f22742d48ce6eeb0c062237b04a5b7f57bfeb8803e9287cd8a112320860e307a")
                .unwrap();

        let secret = ed25519_dalek::SecretKey::from_bytes(&privkey).unwrap();
        let pubkey = ed25519_dalek::PublicKey::from_bytes(&public).unwrap();
        let keypair = ed25519_dalek::Keypair {
            secret: secret,
            public: pubkey,
        };
        keypair
    }

    #[test]
    #[ignore]
    fn test_token_bulk() {
        let (k, p) = new_throwaway_signer();
        let (k2, p2) = new_throwaway_signer();

        let parsed = constructions_pipe(
            vec![
                Operation {
                    operation_identifier: OperationIdentifier {
                        index: 0,
                        network_index: None,
                    },
                    related_operations: None,
                    status: None,
                    account: None,
                    amount: None,
                    type_: OperationType::SplToken__CreateToken,
                    metadata: Some(json!({
                        "mint": p.to_string(),
                        "source": source()
                    })),
                },
                Operation {
                    operation_identifier: OperationIdentifier {
                        index: 1,
                        network_index: None,
                    },
                    related_operations: None,
                    status: None,
                    account: None,
                    amount: None,
                    type_: OperationType::SplToken__CreateAccount,
                    metadata: Some(json!({
                        "mint": p.to_string(),
                        "source": source(),
                        "destination": p2.to_string(),
                    })),
                },
                Operation {
                    operation_identifier: OperationIdentifier {
                        index: 2,
                        network_index: None,
                    },
                    related_operations: None,
                    status: None,
                    account: None,
                    amount: None,
                    type_: OperationType::SplToken__MintTo,
                    metadata: Some(json!({
                        "mint": p.to_string(),
                        "source": p2.to_string(),
                        "authority": source(),
                        "amount": 1000,
                    })),
                },
            ],
            vec![&main_account_keypair(), &k, &k2],
            None,
        );
    }
    #[test]
    #[ignore]
    fn test_construction_transfer() {
        let parsed = constructions_pipe(
            vec![
                Operation {
                    operation_identifier: OperationIdentifier {
                        index: 0,
                        network_index: None,
                    },
                    related_operations: None,
                    status: None,
                    account: None,
                    amount: None,
                    type_: OperationType::System__Transfer,
                    metadata: Some(json!({
                        "source": source(),
                        "destination": dest(),
                        "lamports": 10000,
                    })),
                },
                Operation {
                    operation_identifier: OperationIdentifier {
                        index: 1,
                        network_index: None,
                    },
                    related_operations: None,
                    status: None,
                    account: None,
                    amount: None,
                    type_: OperationType::System__Transfer,
                    metadata: Some(json!({
                        "source": source(),
                        "destination": dest(),
                        "lamports": 10000,
                    })),
                },
            ],
            vec![&main_account_keypair()],
            None,
        );
    }
    #[test]
    #[ignore]
    fn test_token_transfer_rosetta_style() {
        let rpc = create_rpc_client("https://devnet.solana.com".to_string());
        let parsed = constructions_pipe(
            vec![
                Operation {
                    operation_identifier: OperationIdentifier {
                        index: 10,
                        network_index: None,
                    },
                    related_operations: None,
                    status: None,
                    account: Some(AccountIdentifier {
                        address: "95Dq3sXa3omVjiyxBSD6UMrzPYdmyu6CFCw5wS4rhqgV".to_string(),
                        sub_account: None,
                    }),
                    amount: Some(Amount {
                        value: "-0.01".to_string(),
                        currency: Currency {
                            symbol: "3fJRYbtSYZo9SYhwgUBn2zjG98ASy3kuUEnZeHJXqREr".to_string(),
                            decimals: 2,
                            metadata: None,
                        },
                    }),
                    type_: OperationType::SplToken__Transfer,
                    metadata: Some(json!({
                        "authority": source(),
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
                        address: "GyUjMMeZH3PVXp4tk5sR8LgnVaLTvCPipQ3dQY74k75L".to_string(),
                        sub_account: None,
                    }),
                    amount: Some(Amount {
                        value: "0.01".to_string(),
                        currency: Currency {
                            symbol: "3fJRYbtSYZo9SYhwgUBn2zjG98ASy3kuUEnZeHJXqREr".to_string(),
                            decimals: 2,
                            metadata: None,
                        },
                    }),
                    type_: OperationType::SplToken__Transfer,
                    metadata: Some(json!({
                        "authority": source(),
                    })),
                },
            ],
            vec![&main_account_keypair()],
            None,
        );
    }

    #[test]
    #[ignore]
    fn test_nonce_accounts() {
        let (k, p) = new_throwaway_signer();
        let parsed = constructions_pipe(
            vec![Operation {
                operation_identifier: OperationIdentifier {
                    index: 0,
                    network_index: None,
                },
                related_operations: None,
                status: None,
                account: None,
                amount: None,
                type_: OperationType::System__CreateNonceAccount,
                metadata: Some(json!({
                    "source": source(),
                    "authority": source(),
                    "destination": p.to_string()
                })),
            }],
            vec![&main_account_keypair(), &k],
            None,
        );
        thread::sleep(Duration::from_secs(20));
        let parsed = constructions_pipe(
            vec![Operation {
                operation_identifier: OperationIdentifier {
                    index: 1,
                    network_index: None,
                },
                related_operations: None,
                status: None,
                account: None,
                amount: None,
                type_: OperationType::System__Transfer,
                metadata: Some(json!({
                    "source": source(),
                    "destination": dest(),
                    "lamports": 1000,
                })),
            }],
            vec![&main_account_keypair()],
            Some(p.to_string()),
        );
    }

    #[test]
    #[ignore]
    fn test_stake_accounts() {
        let (k, p) = new_throwaway_signer();
        let (k2, p2) = new_throwaway_signer();

        let parsed = constructions_pipe(
            vec![
                Operation {
                    operation_identifier: OperationIdentifier {
                        index: 1,
                        network_index: None,
                    },
                    related_operations: None,
                    status: None,
                    account: None,
                    amount: None,
                    type_: OperationType::Stake__CreateAccount,
                    metadata: Some(json!({
                        "source": source(),
                        "lamports": 1000000000,
                        "lockup": {
                            "epoch": 0,
                            "unix_timestamp": 0,
                            "custodian": source(),
                        },
                        "destination": p.to_string()
                    })),
                },
                Operation {
                    operation_identifier: OperationIdentifier {
                        index: 2,
                        network_index: None,
                    },
                    related_operations: None,
                    status: None,
                    account: None,
                    amount: None,
                    type_: OperationType::Stake__Delegate,
                    metadata: Some(json!({
                        "source": source(),
                        "destination": p.to_string(),
                        "vote_pubkey": "5MMCR4NbTZqjthjLGywmeT66iwE9J9f7kjtxzJjwfUx2".to_string()
                    })),
                },
                Operation {
                    operation_identifier: OperationIdentifier {
                        index: 3,
                        network_index: None,
                    },
                    related_operations: None,
                    status: None,
                    account: None,
                    amount: None,
                    type_: OperationType::Stake__Split,
                    metadata: Some(json!({
                        "source": p.to_string(),
                        "authority": source(),
                        "lamports": 500000000,
                        "destination": p2.to_string()
                    })),
                },
                Operation {
                    operation_identifier: OperationIdentifier {
                        index: 4,
                        network_index: None,
                    },
                    related_operations: None,
                    status: None,
                    account: None,
                    amount: None,
                    type_: OperationType::Stake__Merge,
                    metadata: Some(json!({
                        "source": p2.to_string(),
                        "authority": source(),
                        "destination": p.to_string()
                    })),
                },
                Operation {
                    operation_identifier: OperationIdentifier {
                        index: 5,
                        network_index: None,
                    },
                    related_operations: None,
                    status: None,
                    account: None,
                    amount: None,
                    type_: OperationType::Stake__SetLockup,
                    metadata: Some(json!({
                        "stake_pubkey": p.to_string(),
                        "source": source(),
                        "lockup": {
                            "epoch": 420,
                        }
                    })),
                },
                Operation {
                    operation_identifier: OperationIdentifier {
                        index: 5,
                        network_index: None,
                    },
                    related_operations: None,
                    status: None,
                    account: None,
                    amount: None,
                    type_: OperationType::Stake__Authorize,
                    metadata: Some(json!({
                        "staker": p2.to_string(),
                        "withdrawer": p2.to_string(),
                        "source": source(),
                        "stake_pubkey": p.to_string()
                    })),
                },
            ],
            vec![&main_account_keypair(), &k, &k2],
            None,
        );
    }
    #[test]
    #[ignore]
    fn stake_withdraw_deactivate() {
        let parsed = constructions_pipe(
            vec![
                /*
                Operation {
                    operation_identifier: OperationIdentifier {
                        index: 6,
                        network_index: None,
                    },
                    related_operations: None,
                    status: None,
                    account: None,
                    amount: None,
                    type_: OperationType::Stake__Deactivate,
                    metadata: Some(json!({
                        "source": source(),
                        "destination": "7pLKwSRmAR3pN3PkBnssm142Pg4Daj86WkWrnGC3Uh7h".to_string()
                    })),
                },*/
                Operation {
                    operation_identifier: OperationIdentifier {
                        index: 6,
                        network_index: None,
                    },
                    related_operations: None,
                    status: None,
                    account: None,
                    amount: None,
                    type_: OperationType::Stake__Withdraw,
                    metadata: Some(json!({
                        "source": "7pLKwSRmAR3pN3PkBnssm142Pg4Daj86WkWrnGC3Uh7h".to_string(),
                        "withdrawer": source(),
                        "destination": source(),
                        "lamports": 350000
                    })),
                },
            ],
            vec![&main_account_keypair()],
            None,
        );
    }

    #[test]
    #[ignore]
    fn stake_setlockup() {
        let parsed = constructions_pipe(
            vec![Operation {
                operation_identifier: OperationIdentifier {
                    index: 6,
                    network_index: None,
                },
                related_operations: None,
                status: None,
                account: None,
                amount: None,
                type_: OperationType::Stake__SetLockup,
                metadata: Some(json!({
                    "destination": "7pLKwSRmAR3pN3PkBnssm142Pg4Daj86WkWrnGC3Uh7h".to_string(),
                    "source": source(),
                    "lockup": {
                        "epoch": 0,
                        "unix_timestamp": 100,
                        "custodian": source(),
                    }
                })),
            }],
            vec![&main_account_keypair()],
            None,
        );
    }
    #[test]
    #[ignore]
    fn test_token_transfer() {
        let rpc = create_rpc_client("https://devnet.solana.com".to_string());
        let parsed = constructions_pipe(
            vec![Operation {
                operation_identifier: OperationIdentifier {
                    index: 0,
                    network_index: None,
                },
                related_operations: None,
                status: None,
                account: None,
                amount: None,
                type_: OperationType::SplToken__Transfer,
                metadata: Some(json!({
                    "authority": source(),
                    "source": "95Dq3sXa3omVjiyxBSD6UMrzPYdmyu6CFCw5wS4rhqgV",
                    "destination": "GyUjMMeZH3PVXp4tk5sR8LgnVaLTvCPipQ3dQY74k75L",
                    "amount": "10",
                    "decimals": 2,
                    "mint": "3fJRYbtSYZo9SYhwgUBn2zjG98ASy3kuUEnZeHJXqREr",
                })),
            }],
            vec![&main_account_keypair()],
            None,
        );
    }

    fn new_throwaway_signer() -> (Keypair, solana_sdk::pubkey::Pubkey) {
        let keypair = solana_sdk::signature::Keypair::new();
        let pubkey = solana_sdk::signature::Signer::pubkey(&keypair);
        (
            ed25519_dalek::Keypair::from_bytes(&keypair.to_bytes()).unwrap(),
            pubkey,
        )
    }
    #[test]
    #[ignore]
    fn test_construction_create_assoc_acc() {
        //wont create anymore coz already created change mint address

        let parsed = constructions_pipe(
            vec![Operation {
                operation_identifier: OperationIdentifier {
                    index: 0,
                    network_index: None,
                },
                related_operations: None,
                status: None,
                account: None,
                amount: None,
                type_: OperationType::SplToken__CreateAssocAccount,
                metadata: Some(json!({
                    "source": source(),
                    "mint": "3fJRYbtSYZo9SYhwgUBn2zjG98ASy3kuUEnZeHJXqREr".to_string(),
                })),
            }],
            vec![],
            None,
        );
    }

    fn constructions_pipe(
        operations: Vec<Operation>,
        mut keypairs: Vec<&Keypair>,
        nonce: Option<String>,
    ) -> ConstructionParseResponse {
        let rpc = create_rpc_client("https://devnet.solana.com".to_string());

        let options = Options {
            rpc: rpc,
            network: "devnet".to_string(),
        };
        let network_identifier = NetworkIdentifier {
            blockchain: consts::BLOCKCHAIN.to_string(),
            network: "devnet".to_string(),
            sub_network_identifier: None,
        };
        let prepmeta = if let Some(x) = nonce {
            Some(ConstructionPreprocessRequestMetadata {
                with_nonce: Some(WithNonce {
                    account: x,
                    authority: None,
                }),
            })
        } else {
            None
        };
        let preproess = construction_preprocess(
            ConstructionPreprocessRequest {
                network_identifier: network_identifier.clone(),
                operations: operations.clone(),
                metadata: prepmeta,
            },
            &options,
        )
        .unwrap();
        println!("Preproess {:?} \n\n", &preproess.options);

        let metadata = construction_metadata(
            ConstructionMetadataRequest {
                network_identifier: network_identifier.clone(),
                options: preproess.into_inner().options,
            },
            &options,
        )
        .unwrap();
        println!("Metadata {:?} \n\n", metadata);
        let payloads = construction_payloads(
            ConstructionPayloadsRequest {
                network_identifier: network_identifier.clone(),
                operations: operations,
                metadata: Some(metadata.into_inner().metadata),
            },
            &options,
        )
        .unwrap();
        println!("Payloads {:?} \n\n", payloads);
        let parsed = construction_parse(
            ConstructionParseRequest {
                network_identifier: network_identifier.clone(),
                signed: false,
                transaction: payloads.clone().unsigned_transaction,
            },
            &options,
        )
        .unwrap();
        println!("Parsed {:?} \n\n", parsed);
        let signatures = payloads
            .clone()
            .payloads
            .iter()
            .enumerate()
            .map(|(i, y)| {
                let x = y.clone().unwrap();
                crate::types::Signature {
                    signing_payload: SigningPayload {
                        hex_bytes: x.hex_bytes.clone(),
                        account_identifier: None,
                        signature_type: Some(SignatureType::Ed25519),
                    },
                    public_key: crate::types::PublicKey {
                        hex_bytes: hex::encode(&keypairs[i].public.as_bytes()),
                        curve_type: CurveType::Edwards25519,
                    },
                    signature_type: SignatureType::Ed25519,
                    hex_bytes: sign_msg(&keypairs[i], &x.hex_bytes),
                }
            })
            .collect::<Vec<crate::types::Signature>>();
        println!("Signatures {:?} \n\n", signatures);
        let combined = construction_combine(
            ConstructionCombineRequest {
                network_identifier: network_identifier.clone(),
                unsigned_transaction: payloads.clone().unsigned_transaction,
                signatures: signatures,
            },
            &options,
        )
        .unwrap();
        println!("Signed TX: {:?} \n\n", combined.signed_transaction.clone());

        let submited = construction_submit(
            ConstructionSubmitRequest {
                network_identifier: network_identifier.clone(),
                signed_transaction: combined.signed_transaction.clone(),
            },
            &options,
        );
        println!(
            "Broadcasted TX Hash: {:?} \n\n",
            submited.unwrap().clone().transaction_identifier.hash
        );
        return parsed.into_inner();
    }
    fn sign_msg(keypair: &Keypair, s: &str) -> String {
        let msg = hex::decode(s).unwrap();
        let signature = keypair.sign(&msg);
        hex::encode(signature.to_bytes())
    }
}
