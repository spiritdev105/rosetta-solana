#![feature(proc_macro_hygiene, decl_macro)]
#[macro_use]
extern crate rocket;

mod account;
mod api_routes;
mod block;
mod construction;
mod consts;
mod error;
mod network;
mod operations;
mod types;
mod utils;

use std::{env, time::Duration};

use api_routes::construction::*;
use api_routes::data::*;
use error::ApiError;
//use routes::construction::*;
use rocket::{config::Environment, Config};
use solana_client::rpc_client::RpcClient;
use types::NetworkIdentifier;

pub struct Options {
    rpc: RpcClient,
    network: String,
}

#[cfg(debug_assertions)]
fn get_rocket_env() -> Environment {
    Environment::Development
}

#[cfg(not(debug_assertions))]
fn get_rocket_env() -> Environment {
    Environment::Production
}

fn main() {
    let rpc_url = env::var("RPC_URL").unwrap_or("https://devnet.solana.com".to_string());
    let network = env::var("NETWORK_NAME").unwrap_or("devnet".to_string());
    let host = env::var("HOST").unwrap_or("localhost".to_string());
    let port = env::var("PORT").unwrap_or("8080".to_string());
    let mode = env::var("MODE").unwrap_or("online".to_string());
    let rpc = create_rpc_client(rpc_url);
    let options = Options { rpc: rpc, network };
    let config = Config::build(get_rocket_env())
        .address(host)
        .port(port.parse::<u16>().unwrap())
        .finalize()
        .unwrap();

    let r = if mode == "offline" {
        routes![
            network_list,
            network_options,
            construction_derive,
            construction_preprocess,
            construction_combine,
            construction_parse,
            construction_hash
        ]
    } else {
        routes![
            network_list,
            network_options,
            network_status,
            account_balance,
            get_block, // /block
            block_transaction,
            //TODO: make offline/online paths
            construction_combine,
            construction_derive,
            construction_hash,
            construction_metadata,
            construction_parse,
            construction_payloads,
            construction_preprocess,
            construction_submit,
        ]
    };
    rocket::custom(config)
        .mount("/", r)
        .manage(options)
        //.register(catchers![internal_error])
        .launch();
}

const DEFAULT_RPC_TIMEOUT_SECONDS: u64 = 30;
fn create_rpc_client(url: String) -> RpcClient {
    //let json_rpc_url = solana_cli_config::Config::default().json_rpc_url;
    let rpc_timeout = Duration::from_secs(DEFAULT_RPC_TIMEOUT_SECONDS);
    RpcClient::new_with_timeout(url, rpc_timeout)
}

pub fn is_bad_network(
    options: &Options,
    network_identifier: &NetworkIdentifier,
) -> Result<(), ApiError> {
    if network_identifier.blockchain != consts::BLOCKCHAIN
        || network_identifier.network != options.network
    {
        return Err(ApiError::BadNetwork);
    }
    Ok(())
}
