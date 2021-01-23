use crate::{error::ApiError, is_bad_network, operations::combine_related_operations, Options};
use crate::{
    operations::get_operations_from_encoded_tx,
    types::{
        AccountIdentifier, ConstructionCombineRequest, ConstructionCombineResponse,
        ConstructionDeriveRequest, ConstructionDeriveResponse, ConstructionHashRequest,
        ConstructionMetadata, ConstructionMetadataRequest, ConstructionMetadataResponse,
        ConstructionParseRequest, ConstructionParseResponse, ConstructionPayloadsRequest,
        ConstructionPayloadsResponse, ConstructionPreprocessRequest,
        ConstructionPreprocessResponse, ConstructionSubmitRequest, ConstructionSubmitResponse,
        CurveType, MetadataOptions, Operation, SignatureType, SigningPayload,
        TransactionIdentifier, TransactionIdentifierResponse,
    },
};
use rocket_contrib::json::Json;
use solana_sdk::{
    hash::Hash, instruction::Instruction, message::Message, pubkey::Pubkey, signature::Signature,
    transaction::Transaction,
};
use solana_transaction_status::{EncodedTransaction, UiMessage, UiTransactionEncoding};

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

    let response = ConstructionPreprocessResponse {
        options: MetadataOptions {}, //TODO: Add as necessary
    };
    Ok(Json(response))
}
//Get recent blockhash and other metadata

pub fn construction_metadata(
    construction_metadata_request: ConstructionMetadataRequest,
    options: &Options,
) -> Result<Json<ConstructionMetadataResponse>, ApiError> {
    is_bad_network(&options, &construction_metadata_request.network_identifier)?;

    let (hash, fee_calculator) = options.rpc.get_recent_blockhash()?;
    let response = ConstructionMetadataResponse {
        metadata: ConstructionMetadata {
            blockhash: hash.to_string(),
            fee_calculator,
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

    let mut tx = tx_from_operations(&construction_payloads_request.operations)?;
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
    let payloads = vec![SigningPayload {
        account_identifier: Some(AccountIdentifier {
            address: bs58::encode(tx.message.account_keys[0].to_bytes()).into_string(),
            sub_account: None,
        }),
        hex_bytes: to_be_signed,
        signature_type: Some(SignatureType::Ed25519),
    }];
    let response = ConstructionPayloadsResponse {
        unsigned_transaction,
        payloads,
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
    println!("{:?}", response);
    Ok(Json(response))
}

//broadcast signed tx

pub fn construction_submit(
    construction_submit_request: ConstructionSubmitRequest,
    options: &Options,
) -> Result<Json<ConstructionSubmitResponse>, ApiError> {
    is_bad_network(&options, &construction_submit_request.network_identifier)?;
    let tx = get_tx_from_str(&construction_submit_request.signed_transaction)?;
    let signature = options.rpc.send_transaction(&tx)?;
    let response = ConstructionSubmitResponse {
        transaction_identifier: TransactionIdentifier {
            hash: signature.to_string(),
        },
    };
    Ok(Json(response))
}
fn tx_from_operations(operations: &Vec<Operation>) -> Result<Transaction, ApiError> {
    let instructions = combine_related_operations(operations)?
        .iter()
        .map(|x| x.to_instruction().unwrap())
        .collect::<Vec<Instruction>>();
    let mut fee_payer = None;
    instructions.iter().for_each(|x| {
        if let Some(y) = x.accounts.iter().find(|a| a.is_signer) {
            fee_payer = Some(&y.pubkey);
        }
    });
    let msg = Message::new(&instructions, fee_payer);
    let tx = Transaction::new_unsigned(msg);
    /*
    FIXME: If Operation types are "Unknown" then because this(EncodedTransaction) returns PartiallyDecodedInstruction for some cases.
    println!("{:?}", &tx);
    println!(
        "{:?}",
        EncodedTransaction::encode(tx.clone(), UiTransactionEncoding::JsonParsed)
    );
    */
    Ok(tx)
}
fn get_tx_from_str(s: &str) -> Result<Transaction, ApiError> {
    let try_bs58 = bs58::decode(&s).into_vec().unwrap_or(vec![]);
    if try_bs58.len() == 0 {
        return Err(ApiError::InvalidSignedTransaction);
    }
    let data = try_bs58;
    /*
    let try_base64 = base64::decode(&s);
    let data = if try_base64.is_err() {
    } else {
        try_base64.unwrap()
    };
    */
    Ok(bincode::deserialize(&data).unwrap())
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use ed25519_dalek::*;
    use serde_json::json;

    use crate::{consts, create_rpc_client, types::*};

    //live debug tests on devnet

    use super::*;
    #[test]
    fn test_construction_transfer() {
        let parsed = constructions_pipe(vec![
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
        ]);

        assert_eq!(
            parsed.operations[0].to_instruction().unwrap().accounts[0]
                .pubkey
                .to_string(),
            source()
        );
        assert_eq!(
            parsed.operations[0].to_instruction().unwrap().accounts[1]
                .pubkey
                .to_string(),
            dest()
        );
    }
    #[test]
    fn test_token_transfer_rosetta_style() {
        let rpc = create_rpc_client("https://devnet.solana.com".to_string());
        let parsed = constructions_pipe(vec![
            Operation {
                operation_identifier: OperationIdentifier {
                    index: 10,
                    network_index: None,
                },
                related_operations: Some(vec![OperationIdentifier {
                    index: 11,
                    network_index: None,
                }]),
                status: None,
                account: Some(AccountIdentifier {
                    address: "95Dq3sXa3omVjiyxBSD6UMrzPYdmyu6CFCw5wS4rhqgV".to_string(),
                    sub_account: None,
                }),
                amount: Some(Amount {
                    value: "-10".to_string(),
                    currency: Currency {
                        symbol: "3fJRYbtSYZo9SYhwgUBn2zjG98ASy3kuUEnZeHJXqREr".to_string(),
                        decimals: 2,
                    },
                }),
                type_: OperationType::SplToken__TransferChecked,
                metadata: Some(json!({
                    "authority": source(),
                })),
            },
            Operation {
                operation_identifier: OperationIdentifier {
                    index: 11,
                    network_index: None,
                },
                related_operations: Some(vec![OperationIdentifier {
                    index: 10,
                    network_index: None,
                }]),
                status: None,
                account: Some(AccountIdentifier {
                    address: "GyUjMMeZH3PVXp4tk5sR8LgnVaLTvCPipQ3dQY74k75L".to_string(),
                    sub_account: None,
                }),
                amount: Some(Amount {
                    value: "10".to_string(),
                    currency: Currency {
                        symbol: "3fJRYbtSYZo9SYhwgUBn2zjG98ASy3kuUEnZeHJXqREr".to_string(),
                        decimals: 2,
                    },
                }),
                type_: OperationType::SplToken__TransferChecked,
                metadata: Some(json!({
                    "authority": source(),
                })),
            },
        ]);
    }

    #[test]
    fn test_token_transfer() {
        let rpc = create_rpc_client("https://devnet.solana.com".to_string());
        let parsed = constructions_pipe(vec![Operation {
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
                "amount": 10,
                "decimals": 2,
                "mint": "3fJRYbtSYZo9SYhwgUBn2zjG98ASy3kuUEnZeHJXqREr",
            })),
        }]);
    }
    #[test]
    fn test_construction_create_assoc_acc() {
        //wont create anymore coz already created change mint address

        let parsed = constructions_pipe(vec![Operation {
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
        }]);
    }
    fn source() -> String {
        "HJGPMwVuqrbm7BDMeA3shLkqdHUru337fgytM7HzqTnH".to_string()
    }
    fn dest() -> String {
        "CgVKbBwogjaqtGtPLkMBSkhwtkTMLVdSdHM5cWzyxT5n".to_string()
    }
    fn constructions_pipe(operations: Vec<Operation>) -> ConstructionParseResponse {
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
        let metadata = construction_metadata(
            ConstructionMetadataRequest {
                network_identifier: network_identifier.clone(),
                options: None,
            },
            &options,
        )
        .unwrap();

        let payloads = construction_payloads(
            ConstructionPayloadsRequest {
                network_identifier: network_identifier.clone(),
                operations: operations,
                metadata: Some(metadata.into_inner().metadata),
            },
            &options,
        )
        .unwrap();
        let parsed = construction_parse(
            ConstructionParseRequest {
                network_identifier: network_identifier.clone(),
                signed: false,
                transaction: payloads.clone().unsigned_transaction,
            },
            &options,
        )
        .unwrap();

        let signatures = payloads
            .clone()
            .payloads
            .iter()
            .map(|x| crate::types::Signature {
                signing_payload: SigningPayload {
                    hex_bytes: x.hex_bytes.clone(),
                    account_identifier: None,
                    signature_type: Some(SignatureType::Ed25519),
                },
                public_key: crate::types::PublicKey {
                    hex_bytes: "f22742d48ce6eeb0c062237b04a5b7f57bfeb8803e9287cd8a112320860e307a"
                        .to_string(),
                    curve_type: CurveType::Edwards25519,
                },
                signature_type: SignatureType::Ed25519,
                hex_bytes: sign_msg(&x.hex_bytes),
            })
            .collect::<Vec<crate::types::Signature>>();
        let combined = construction_combine(
            ConstructionCombineRequest {
                network_identifier: network_identifier.clone(),
                unsigned_transaction: payloads.clone().unsigned_transaction,
                signatures: signatures,
            },
            &options,
        )
        .unwrap();
        println!("Signed TX: {:?}", combined.signed_transaction.clone());

        let submited = construction_submit(
            ConstructionSubmitRequest {
                network_identifier: network_identifier.clone(),
                signed_transaction: combined.signed_transaction.clone(),
            },
            &options,
        );
        println!(
            "Broadcasted TX Hash: {:?}",
            submited.unwrap().clone().transaction_identifier.hash
        );
        return parsed.into_inner();
    }
    fn sign_msg(s: &str) -> String {
        let privkey =
            hex::decode("cb1a134c296fbf309d78fe9378c18bc129e5045fbe92d2ad8577ccc84689d4ef")
                .unwrap();
        let public =
            hex::decode("f22742d48ce6eeb0c062237b04a5b7f57bfeb8803e9287cd8a112320860e307a")
                .unwrap();
        let msg = hex::decode(s).unwrap();
        let secret = ed25519_dalek::SecretKey::from_bytes(&privkey).unwrap();
        let pubkey = ed25519_dalek::PublicKey::from_bytes(&public).unwrap();
        let keypair = ed25519_dalek::Keypair {
            secret: secret,
            public: pubkey,
        };
        let signature = keypair.sign(&msg);
        hex::encode(signature.to_bytes())
    }
}
