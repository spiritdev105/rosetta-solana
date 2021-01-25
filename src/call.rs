use rocket_contrib::json::Json;
use solana_client::{http_sender::HttpSender, rpc_request::RpcRequest, rpc_sender::RpcSender};

use crate::{
    error::ApiError,
    is_bad_network,
    types::{CallRequest, CallResponse},
    Options, Options2,
};

//temporary workaround sender when rpc client adds send()->Value fn this can be removed
pub struct RpcSender2 {
    sender: HttpSender,
}
impl RpcSender2 {
    pub fn new(rpc_url: String) -> RpcSender2 {
        let sender = HttpSender::new(rpc_url);
        RpcSender2 { sender }
    }
    pub fn send(
        &self,
        request: RpcRequest,
        params: serde_json::Value,
    ) -> Result<serde_json::Value, ApiError> {
        let response = self
            .sender
            .send(request, params)
            .map_err(|err| err.into_with_request(request))?;
        Ok(response)
    }
}
pub fn call_direct(
    req: CallRequest,
    options: &Options,
    options2: &Options2,
) -> Result<Json<CallResponse>, ApiError> {
    is_bad_network(&options, &req.network_identifier)?;

    if !(req.parameters.is_array() || req.parameters.is_null()) {
        return Err(ApiError::BadRequest);
    }

    let result = options2.rpc2.send(
        solana_client::rpc_request::RpcRequest::from(req.method),
        req.parameters,
    )?;
    let response = CallResponse {
        result,
        idempotent: false,
    };
    Ok(Json(response))
}
