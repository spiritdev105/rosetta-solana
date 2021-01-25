use std::str::FromStr;

use crate::{
    error::ApiError,
    is_bad_network,
    types::Block,
    types::BlockRequest,
    types::BlockTransactionResponse,
    types::{BlockIdentifier, BlockResponse, BlockTransactionRequest, Transaction},
    Options,
};

use rocket_contrib::json::Json;

use solana_sdk::signature::Signature;
use solana_transaction_status::UiTransactionEncoding;

pub fn block(
    block_request: BlockRequest,
    options: &Options,
) -> Result<Json<BlockResponse>, ApiError> {
    is_bad_network(&options, &block_request.network_identifier)?;

    let block_index = block_request
        .block_identifier
        .index
        .ok_or_else(|| ApiError::BadRequest)?;
    let block_result = options.rpc.get_confirmed_block_with_encoding(
        block_index,
        solana_transaction_status::UiTransactionEncoding::JsonParsed,
    );
    let block;
    if block_result.is_err() {
        return Ok(Json(BlockResponse { block: None }));
    } else {
        block = block_result.unwrap();
    }
    let transactions = block
        .transactions
        .iter()
        .map(|x| Transaction::from(x))
        .collect::<Vec<Transaction>>();
    let response = BlockResponse {
        block: Some(Block {
            block_identifier: BlockIdentifier {
                index: block_index,
                hash: block.blockhash,
            },
            parent_block_identifier: BlockIdentifier {
                index: block.parent_slot,
                hash: block.previous_blockhash,
            },
            timestamp: (block.block_time.unwrap_or(1611091000) as u64) * 1000,
            transactions: transactions,
        }),
    };
    Ok(Json(response))
}

pub fn block_transaction(
    block_transaction_request: BlockTransactionRequest,
    options: &Options,
) -> Result<Json<BlockTransactionResponse>, ApiError> {
    is_bad_network(&options, &block_transaction_request.network_identifier)?;
    let hash = Signature::from_str(
        &block_transaction_request
            .transaction_identifier
            .hash
            .as_str(),
    )?;

    //FIXME: invalid type: null, expected struct EncodedConfirmedTransaction when tx dont exists should return Option<EncodedConfirmedTransaction> instead of directly to prevent this shold be fixed in sdk itself
    let tx = options
        .rpc
        .get_confirmed_transaction(&hash, UiTransactionEncoding::JsonParsed)?;
    let response = BlockTransactionResponse {
        transaction: Transaction::from(&tx.transaction),
    };
    Ok(Json(response))
}
