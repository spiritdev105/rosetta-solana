use std::str::FromStr;

use crate::{
    error::ApiError,
    is_bad_network,
    types::Feature,
    types::{AcceptanceCriteria, FeaturesRequest, FeaturesResponse},
    utils::to_pub,
    Options,
};
use rocket_contrib::json::Json;
use solana_sdk::program_pack::Pack;
use solana_sdk::{feature_set::FEATURE_NAMES, signature::Signature};
use solana_transaction_status::{
    EncodedConfirmedTransaction, EncodedTransaction, UiInstruction, UiMessage,
    UiTransactionEncoding,
};
use spl_feature_proposal::instruction::FeatureProposalInstruction;

pub fn get_features(
    req: FeaturesRequest,
    options: &Options,
) -> Result<Json<FeaturesResponse>, ApiError> {
    is_bad_network(&options, &req.network_identifier)?;
    let feature_program_id = spl_feature_proposal::id();
    let accounts = options.rpc.get_program_accounts(&feature_program_id)?;
    let features = accounts
        .iter()
        .map(|(x, _)| {
            let tx = options
                .rpc
                .get_confirmed_signatures_for_address2(x)
                .unwrap();
            //tx sig of proposal
            let parsed_tx = options
                .rpc
                .get_confirmed_transaction(
                    &Signature::from_str(&tx.last().unwrap().signature).unwrap(),
                    UiTransactionEncoding::Json,
                )
                .unwrap();
            println!("{:?}", parsed_tx);
            let feature = if let EncodedTransaction::Json(x) = parsed_tx.transaction.transaction {
                if let UiMessage::Raw(x) = x.message {
                    let funding_address = x.account_keys[0].clone();
                    let feature_proposal_address = x.account_keys[1].clone();
                    let mint_address = x.account_keys[2].clone();
                    let distributor_token_address = x.account_keys[3].clone();
                    let acceptance_token_address = x.account_keys[4].clone();
                    let feature_id_address = x.account_keys[5].clone();
                    let name = FEATURE_NAMES
                        .get(&to_pub(&feature_id_address))
                        .unwrap_or(&feature_id_address.as_str())
                        .to_string();

                    let mut ttm = 0;
                    let mut ac = AcceptanceCriteria::default();
                    let decoded_data = bs58::decode(x.instructions[0].data.to_string())
                        .into_vec()
                        .unwrap();
                    let proposal_instruction =
                        FeatureProposalInstruction::unpack_from_slice(&decoded_data).unwrap();
                    if let FeatureProposalInstruction::Propose {
                        tokens_to_mint,
                        acceptance_criteria,
                    } = proposal_instruction
                    {
                        ttm = tokens_to_mint;
                        ac = AcceptanceCriteria {
                            tokens_required: acceptance_criteria.tokens_required,
                            deadline: acceptance_criteria.deadline,
                        }
                    }
                    let f = Feature {
                        name,
                        funding_address,
                        feature_proposal_address,
                        mint_address,
                        distributor_token_address,
                        acceptance_token_address,
                        feature_id_address,
                        tokens_to_mint: ttm,
                        acceptance_criteria: ac,
                    };
                    Some(f)
                } else {
                    None
                }
            } else {
                None
            };
            feature
        })
        .filter_map(|x| x)
        .collect::<Vec<Feature>>();
    Ok(Json(FeaturesResponse { features }))
}
//EncodedConfirmedTransaction { slot: 36265038, transaction: EncodedTransactionWithStatusMeta { transaction: Json(UiTransaction { signatures: ["rCFXW6KBwqeubDqSggyDM8YxU4NianBDzfeHwbmB36Eaw9n5qxfuqvtFLqkkDZuLRJyshH1qyZSxCUsvLKP69nK", "3AK9zmZmTDz5BLXdDwibcSzmXVfqcMWgdpm7TmF5omBAu1gfxvUm3TvKbaBDAHXGVHmbZ5yNsAb3MUSZBeKLow7w"], message: Raw(UiRawMessage { header: MessageHeader { num_required_signatures: 2, num_readonly_signed_accounts: 0, num_readonly_unsigned_accounts: 4 }, account_keys: ["HJGPMwVuqrbm7BDMeA3shLkqdHUru337fgytM7HzqTnH", "BnSuisc6EfQ6JYCpVppYGPXweJbWwGFo5rP1urReHqd", "DQzjTELf9udq4xErkwU53YQMwWpy5bYecMbziuBVLcjv", "DE9f1grJTYk1epgq86ioeNGymYv24isSCJrgnnwmeR8c", "CSxXC756mZYrAquQNs7EJ4u9xLZC6YGTnvvQqTeDom3L", "3sfjf5r6iTuLVuwa92td6YN1ytfXVSTiJDECrQqNYzHW", "11111111111111111111111111111111", "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA", "SysvarRent111111111111111111111111111111111", "Feat1YXHhH6t1juaWF74WLcfv4XoNocjXA6sPWHNgAse"], recent_blockhash: "DsrkdRiw8rf1wKGfPmDPEwjGNe8Q1dYnyQcKwyXUo2Zp", instructions: [UiCompiledInstruction { program_id_index: 9, accounts: [0, 1, 2, 3, 4, 5, 6, 7, 8], data: "19ZzoL2fCKATvq3fcagV6fxt4zxmfefipP" }] }) }), meta: Some(UiTransactionStatusMeta { err: None, status: Ok(()), fee: 10000, pre_balances: [5382919120, 0, 0, 0, 0, 0, 1, 1118555520, 1, 1332533760], post_balances: [5375406240, 1009200, 1461600, 2039280, 2039280, 953520, 1, 1118555520, 1, 1332533760], inner_instructions: None, log_messages: Some([]), pre_token_balances: Some([]), post_token_balances: Some([]) }) } }

//EncodedConfirmedTransaction { slot: 36265038, transaction: EncodedTransactionWithStatusMeta { transaction: Json(UiTransaction { signatures: ["rCFXW6KBwqeubDqSggyDM8YxU4NianBDzfeHwbmB36Eaw9n5qxfuqvtFLqkkDZuLRJyshH1qyZSxCUsvLKP69nK", "3AK9zmZmTDz5BLXdDwibcSzmXVfqcMWgdpm7TmF5omBAu1gfxvUm3TvKbaBDAHXGVHmbZ5yNsAb3MUSZBeKLow7w"], message: Parsed(UiParsedMessage { account_keys: [ParsedAccount { pubkey: "HJGPMwVuqrbm7BDMeA3shLkqdHUru337fgytM7HzqTnH", writable: true, signer: true }, ParsedAccount { pubkey: "BnSuisc6EfQ6JYCpVppYGPXweJbWwGFo5rP1urReHqd", writable: true, signer: true }, ParsedAccount { pubkey: "DQzjTELf9udq4xErkwU53YQMwWpy5bYecMbziuBVLcjv", writable: true, signer: false }, ParsedAccount { pubkey: "DE9f1grJTYk1epgq86ioeNGymYv24isSCJrgnnwmeR8c", writable: true, signer: false }, ParsedAccount { pubkey: "CSxXC756mZYrAquQNs7EJ4u9xLZC6YGTnvvQqTeDom3L", writable: true, signer: false }, ParsedAccount { pubkey: "3sfjf5r6iTuLVuwa92td6YN1ytfXVSTiJDECrQqNYzHW", writable: true, signer: false }, ParsedAccount { pubkey: "11111111111111111111111111111111", writable: false, signer: false }, ParsedAccount { pubkey: "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA", writable: false, signer: false }, ParsedAccount { pubkey: "SysvarRent111111111111111111111111111111111", writable: false, signer: false }, ParsedAccount { pubkey: "Feat1YXHhH6t1juaWF74WLcfv4XoNocjXA6sPWHNgAse", writable: false, signer: false }], recent_blockhash: "DsrkdRiw8rf1wKGfPmDPEwjGNe8Q1dYnyQcKwyXUo2Zp", instructions: [Parsed(PartiallyDecoded(UiPartiallyDecodedInstruction { program_id: "Feat1YXHhH6t1juaWF74WLcfv4XoNocjXA6sPWHNgAse", accounts: ["HJGPMwVuqrbm7BDMeA3shLkqdHUru337fgytM7HzqTnH", "BnSuisc6EfQ6JYCpVppYGPXweJbWwGFo5rP1urReHqd", "DQzjTELf9udq4xErkwU53YQMwWpy5bYecMbziuBVLcjv", "DE9f1grJTYk1epgq86ioeNGymYv24isSCJrgnnwmeR8c", "CSxXC756mZYrAquQNs7EJ4u9xLZC6YGTnvvQqTeDom3L", "3sfjf5r6iTuLVuwa92td6YN1ytfXVSTiJDECrQqNYzHW", "11111111111111111111111111111111", "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA", "SysvarRent111111111111111111111111111111111"], data: "19ZzoL2fCKATvq3fcagV6fxt4zxmfefipP" }))] }) }), meta: Some(UiTransactionStatusMeta { err: None, status: Ok(()), fee: 10000, pre_balances: [5382919120, 0, 0, 0, 0, 0, 1, 1118555520, 1, 1332533760], post_balances: [5375406240, 1009200, 1461600, 2039280, 2039280, 953520, 1, 1118555520, 1, 1332533760], inner_instructions: None, log_messages: Some([]), pre_token_balances: Some([]), post_token_balances: Some([]) }) } },
