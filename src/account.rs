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

#[derive(Debug, Deserialize)]
struct TokenAmount {
    amount: String,
    decimals: u8,
}
#[derive(Debug, Deserialize)]
struct Info {
    #[serde(rename = "tokenAmount")]
    token_amount: TokenAmount,
    mint: String,
    owner: String,
}
#[derive(Debug, Deserialize)]
struct Parsed {
    info: Info,
}
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
            metadata: None,
        },
        value: balance.to_string(),
    };
    let symbols = if let Some(x) = &account_balance_request.currencies {
        x.iter().map(|c| c.symbol.as_str()).collect::<Vec<&str>>()
    } else {
        vec![]
    };

    let token_acc = options.rpc.get_token_account(&pubkey);
    if let Ok(Some(x)) = token_acc {
        let symbol = x.mint;

        if symbols.len() == 0 || symbols.contains(&symbol.as_str()) {
            balances.push(Amount {
                currency: Currency {
                    symbol,
                    decimals: x.token_amount.decimals,
                    metadata: None,
                },
                value: x.token_amount.amount,
            });
        }
    }

    let account_tokens = options
        .rpc
        .get_token_accounts_by_owner(&pubkey, TokenAccountsFilter::ProgramId(spl_token::id()))?;
    account_tokens.into_iter().for_each(|x| {
        if let solana_account_decoder::UiAccountData::Json(parsed_acc) = x.account.data {
            let parsed = serde_json::from_value::<Parsed>(parsed_acc.parsed).unwrap();
            let amount = parsed.info.token_amount.amount;
            let decimals = parsed.info.token_amount.decimals;
            let symbol = parsed.info.mint;
            if symbols.len() == 0 || symbols.contains(&symbol.as_str()) {
                balances.push(Amount {
                    currency: Currency {
                        symbol,
                        decimals,
                        metadata: Some(serde_json::json!({"pubkey":  x.pubkey})),
                    },
                    value: amount,
                });
            }
        };
    });

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

#[cfg(test)]
mod tests {
    use crate::{
        consts, create_rpc_client, types::AccountBalanceRequest, types::AccountIdentifier,
        types::NetworkIdentifier, Options,
    };

    use super::account_balance;

    #[test]
    #[ignore]
    fn test_balance() {
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

        let acc = account_balance(
            AccountBalanceRequest {
                network_identifier: network_identifier,
                account_identifier: AccountIdentifier {
                    address: "Cnqmx3sbJf35852dAWvwf7GhuxMGWm5gGgw3biebSsBM".to_string(),
                    sub_account: None,
                },
                block_identifier: None,
                currencies: None,
            },
            &options,
        );
        assert_eq!(true, acc.is_ok());
    }
}
