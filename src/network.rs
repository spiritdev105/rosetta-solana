use crate::{
    consts,
    error::ApiError,
    is_bad_network,
    types::Allow,
    types::RpcRequestInternal,
    types::{
        BlockIdentifier, NetworkIdentifier, NetworkListResponse, NetworkOptionsResponse,
        NetworkRequest, NetworkStatusResponse, OperationStatus, OperationStatusType, OperationType,
        Peer, Version,
    },
    Options,
};

use rocket_contrib::json::Json;
use strum::IntoEnumIterator;

pub fn network_list(options: &Options) -> Result<Json<NetworkListResponse>, ApiError> {
    let response = NetworkListResponse {
        network_identifiers: vec![NetworkIdentifier {
            blockchain: consts::BLOCKCHAIN.to_string(),
            network: options.network.clone(), //TODO: genesis config cluster type
            sub_network_identifier: None,
        }],
    };
    Ok(Json(response))
}
pub fn network_options(
    network_request: NetworkRequest,
    options: &Options,
) -> Result<Json<NetworkOptionsResponse>, ApiError> {
    is_bad_network(&options, &network_request.network_identifier)?;

    let version = Version {
        rosetta_version: consts::ROSETTA_VERSION.to_string(),
        node_version: consts::NODE_VERSION.to_string(),
        middleware_version: consts::MIDDLEWARE_VERSION.to_string(),
    };

    let operation_statuses = vec![
        OperationStatus {
            status: OperationStatusType::Success,
            successful: true,
        },
        OperationStatus {
            status: OperationStatusType::Faliure,
            successful: false,
        },
    ];

    let errors = ApiError::all_errors();

    let allow = Allow {
        operation_statuses,
        operation_types: OperationType::iter().collect(),
        errors,
        historical_balance_lookup: false,
        timestamp_start_index: Some(0), // TODO: find this
        call_methods: RpcRequestInternal::iter().collect(),
        balance_exemptions: vec![],
    };

    let response = NetworkOptionsResponse { version, allow };
    Ok(Json(response))
}
pub fn network_status(
    network_request: NetworkRequest,
    options: &Options,
) -> Result<Json<NetworkStatusResponse>, ApiError> {
    is_bad_network(&options, &network_request.network_identifier)?;

    let genesis = options.rpc.get_genesis_hash()?;
    let index = options.rpc.get_first_available_block()?;
    let genesis_block_identifier = BlockIdentifier {
        index: index,
        hash: genesis.to_string(),
    };
    let (slot, slot_time, current_block_identifier) = get_current_block(&options)?;
    let current_block_timestamp = (slot_time * 1000) as u64;
    let cluster_nodes = options.rpc.get_cluster_nodes()?;
    let peers: Vec<Peer> = cluster_nodes
        .into_iter()
        .map(|x| Peer { peer_id: x.pubkey })
        .collect();

    let response = NetworkStatusResponse {
        current_block_identifier,
        current_block_timestamp,
        genesis_block_identifier,
        peers,
    };

    Ok(Json(response))
}

pub fn get_current_block(options: &Options) -> Result<(u64, i64, BlockIdentifier), ApiError> {
    let slot = options.rpc.get_slot()?;
    let slot_time = options.rpc.get_block_time(slot)?;
    let current_block_identifier = BlockIdentifier {
        index: slot,
        hash: slot.to_string(), //TODO: should be hash not slot
    };
    Ok((slot, slot_time, current_block_identifier))
}
