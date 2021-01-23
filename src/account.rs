use crate::{
    consts,
    error::ApiError,
    is_bad_network,
    network::get_current_block,
    types::AccountBalanceRequest,
    types::AccountBalanceResponse,
    types::{Amount, Currency},
    Options,
};
use rocket::State;
use rocket_contrib::json::Json;
use serde::Deserialize;
use solana_client::rpc_request::TokenAccountsFilter;
use solana_sdk::pubkey::Pubkey;

pub fn account_balance(
    account_balance_request: AccountBalanceRequest,
    options: &Options,
) -> Result<Json<AccountBalanceResponse>, ApiError> {
    is_bad_network(&options, &account_balance_request.network_identifier)?;

    if account_balance_request.block_identifier.is_some() {
        return Err(ApiError::HistoricBalancesUnsupported);
    }

    let mut balances = vec![];
    let address = &account_balance_request.account_identifier.address;
    let pubkey = address.parse::<Pubkey>()?;
    let balance = options.rpc.get_balance(&pubkey)?;
    let native_balance = Amount {
        currency: Currency {
            symbol: consts::NATIVE_SYMBOL.to_string(),
            decimals: consts::NATIVE_DECIMALS,
        },
        value: balance.to_string(),
    };

    let account_tokens = options.rpc.get_token_accounts_by_owner(
        &pubkey,
        TokenAccountsFilter::ProgramId(consts::SPL_PROGRAM_ID.parse::<Pubkey>()?),
    )?;

    #[derive(Deserialize)]
    struct TokenAmount {
        amount: String,
        decimals: u8,
    }
    #[derive(Deserialize)]
    struct Info {
        token_amount: TokenAmount,
        mint: String,
    }
    #[derive(Deserialize)]
    struct Parsed {
        info: Info,
    }
    #[derive(Deserialize)]
    struct Data {
        parsed: Parsed,
    }
    let symbols = if let Some(x) = &account_balance_request.currencies {
        x.iter().map(|c| c.symbol.as_str()).collect::<Vec<&str>>()
    } else {
        vec![]
    };
    account_tokens.iter().for_each(|x| {
        let acc = x.account.decode();
        if let Some(a) = acc {
            let y: Data = a.deserialize_data().unwrap(); //TODO: Fix this
            let symbol = y.parsed.info.mint; // TODO: Not symbol but mint address
            if symbols.len() == 0 || symbols.contains(&symbol.as_str()) {
                balances.push(Amount {
                    currency: Currency {
                        symbol: symbol,
                        decimals: y.parsed.info.token_amount.decimals,
                    },
                    value: y.parsed.info.token_amount.amount,
                });
            }
        }
    });
    //FIXME: Just why
    if symbols.len() == 0 || symbols.contains(&consts::NATIVE_SYMBOL.to_string().as_str()) {
        balances.push(native_balance);
    }

    let (_, _, current_block_identifier) = get_current_block(&options)?;

    let response = AccountBalanceResponse {
        block_identifier: current_block_identifier,
        balances: balances,
    };
    Ok(Json(response))
}
