use crate::{
    account, block,
    call::call_direct,
    error::ApiError,
    network,
    types::CallRequest,
    types::{
        AccountBalanceRequest, AccountBalanceResponse, BlockRequest, BlockResponse,
        BlockTransactionRequest, BlockTransactionResponse, CallResponse, NetworkListResponse,
        NetworkOptionsResponse, NetworkRequest, NetworkStatusResponse,
    },
    Options, Options2,
};
use rocket::State;
use rocket_contrib::json::Json;
use solana_client::{http_sender::HttpSender, rpc_request::RpcRequest, rpc_sender::RpcSender};

#[post("/network/list")]
pub fn network_list(options: State<Options>) -> Result<Json<NetworkListResponse>, ApiError> {
    network::network_list(options.inner())
}
#[post("/network/options", data = "<network_request>")]
pub fn network_options(
    network_request: Json<NetworkRequest>,
    options: State<Options>,
) -> Result<Json<NetworkOptionsResponse>, ApiError> {
    network::network_options(network_request.into_inner(), options.inner())
}
#[post("/network/status", data = "<network_request>")]
pub fn network_status(
    network_request: Json<NetworkRequest>,
    options: State<Options>,
) -> Result<Json<NetworkStatusResponse>, ApiError> {
    network::network_status(network_request.into_inner(), options.inner())
}

#[post("/account/balance", data = "<account_balance_request>")]
pub fn account_balance(
    account_balance_request: Json<AccountBalanceRequest>,
    options: State<Options>,
) -> Result<Json<AccountBalanceResponse>, ApiError> {
    account::account_balance(account_balance_request.into_inner(), options.inner())
}

#[post("/block", data = "<block_request>")]
pub fn get_block(
    block_request: Json<BlockRequest>,
    options: State<Options>,
) -> Result<Json<BlockResponse>, ApiError> {
    block::block(block_request.into_inner(), options.inner())
}
#[post("/block/transaction", data = "<block_transaction_request>")]
pub fn block_transaction(
    block_transaction_request: Json<BlockTransactionRequest>,
    options: State<Options>,
) -> Result<Json<BlockTransactionResponse>, ApiError> {
    block::block_transaction(block_transaction_request.into_inner(), options.inner())
}

#[post("/call", data = "<call_request>")]
pub fn call(
    call_request: Json<CallRequest>,
    options: State<Options>,
    options2: State<Options2>,
) -> Result<Json<CallResponse>, ApiError> {
    call_direct(call_request.into_inner(), options.inner(), options2.inner())
}
