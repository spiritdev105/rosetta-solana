use crate::{
    types::{
        ConstructionCombineRequest, ConstructionCombineResponse, ConstructionDeriveRequest,
        ConstructionDeriveResponse, ConstructionHashRequest, ConstructionMetadataRequest,
        ConstructionMetadataResponse, ConstructionParseRequest, ConstructionParseResponse,
        ConstructionPayloadsRequest, ConstructionPayloadsResponse, ConstructionPreprocessRequest,
        ConstructionPreprocessResponse, ConstructionSubmitRequest, ConstructionSubmitResponse,
        TransactionIdentifierResponse,
    },
    *,
};
use rocket::State;
use rocket_contrib::json::Json;

#[post("/construction/derive", data = "<construction_derive_request>")]
pub fn construction_derive(
    construction_derive_request: Json<ConstructionDeriveRequest>,
    options: State<Options>,
) -> Result<Json<ConstructionDeriveResponse>, ApiError> {
    construction::construction_derive(construction_derive_request.into_inner(), options.inner())
}
#[post("/construction/hash", data = "<construction_hash_request>")]
pub fn construction_hash(
    construction_hash_request: Json<ConstructionHashRequest>,
    options: State<Options>,
) -> Result<Json<TransactionIdentifierResponse>, ApiError> {
    construction::construction_hash(construction_hash_request.into_inner(), options.inner())
}
//Create Metadata Request to send to construction/metadata
#[post("/construction/preprocess", data = "<construction_preprocess_request>")]
pub fn construction_preprocess(
    construction_preprocess_request: Json<ConstructionPreprocessRequest>,
    options: State<Options>,
) -> Result<Json<ConstructionPreprocessResponse>, ApiError> {
    construction::construction_preprocess(
        construction_preprocess_request.into_inner(),
        options.inner(),
    )
}
//Get recent blockhash and other metadata
#[post("/construction/metadata", data = "<construction_metadata_request>")]
pub fn construction_metadata(
    construction_metadata_request: Json<ConstructionMetadataRequest>,
    options: State<Options>,
) -> Result<Json<ConstructionMetadataResponse>, ApiError> {
    construction::construction_metadata(construction_metadata_request.into_inner(), options.inner())
}
//Construct Payloads to Sign
#[post("/construction/payloads", data = "<construction_payloads_request>")]
pub fn construction_payloads(
    construction_payloads_request: Json<ConstructionPayloadsRequest>,
    options: State<Options>,
) -> Result<Json<ConstructionPayloadsResponse>, ApiError> {
    construction::construction_payloads(construction_payloads_request.into_inner(), options.inner())
}

//Parse Unsigned Transaction to to Confirm Correctness
#[post("/construction/parse", data = "<construction_parse_request>")]
pub fn construction_parse(
    construction_parse_request: Json<ConstructionParseRequest>,
    options: State<Options>,
) -> Result<Json<ConstructionParseResponse>, ApiError> {
    construction::construction_parse(construction_parse_request.into_inner(), options.inner())
}

//combine sign
#[post("/construction/combine", data = "<construction_combine_request>")]
pub fn construction_combine(
    construction_combine_request: Json<ConstructionCombineRequest>,
    options: State<Options>,
) -> Result<Json<ConstructionCombineResponse>, ApiError> {
    construction::construction_combine(construction_combine_request.into_inner(), options.inner())
}

//broadcast signed tx
#[post("/construction/submit", data = "<construction_submit_request>")]
pub fn construction_submit(
    construction_submit_request: Json<ConstructionSubmitRequest>,
    options: State<Options>,
) -> Result<Json<ConstructionSubmitResponse>, ApiError> {
    construction::construction_submit(construction_submit_request.into_inner(), options.inner())
}
