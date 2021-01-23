use crate::types::OperationType;

pub const BLOCKCHAIN: &str = "solana";
pub const MIDDLEWARE_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const ROSETTA_VERSION: &str = "1.4.4";
pub const NODE_VERSION: &str = "1.4.17";

pub const NATIVE_SYMBOL: &str = "SOL";
pub const NATIVE_DECIMALS: u8 = 9;
pub const SPL_PROGRAM_ID: &str = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA";
pub const SPL_ISSUER: &str = "BPFLoader2111111111111111111111111111111111";
pub const SEPARATOR: &str = "__"; //TODO: This should be only once in str or breaks

pub fn get_json_rpc_methods() -> Vec<String> {
    vec![
        "getAccountInfo".to_string(),
        "getBalance".to_string(),
        "getBlockCommitment".to_string(),
        "getBlockTime".to_string(),
        "getClusterNodes".to_string(),
        "getConfirmedBlock".to_string(),
        "getConfirmedBlocks".to_string(),
        "getConfirmedBlocksWithLimit".to_string(),
        "getConfirmedSignaturesForAddress".to_string(),
        "getConfirmedSignaturesForAddress2".to_string(),
        "getConfirmedTransaction".to_string(),
        "getEpochInfo".to_string(),
        "getEpochSchedule".to_string(),
        "getFeeCalculatorForBlockhash".to_string(),
        "getFeeRateGovernor".to_string(),
        "getFees".to_string(),
        "getFirstAvailableBlock".to_string(),
        "getGenesisHash".to_string(),
        "getIdentity".to_string(),
        "getInflationGovernor".to_string(),
        "getInflationRate".to_string(),
        "getLargestAccounts".to_string(),
        "getLeaderSchedule".to_string(),
        "getMinimumBalanceForRentExemption".to_string(),
        "getMultipleAccounts".to_string(),
        "getProgramAccounts".to_string(),
        "getRecentBlockhash".to_string(),
        "getRecentPerformanceSamples".to_string(),
        "getSignatureStatuses".to_string(),
        "getSlot".to_string(),
        "getSlotLeader".to_string(),
        "getStakeActivation".to_string(),
        "getSupply".to_string(),
        "getTransactionCount".to_string(),
        "getVersion".to_string(),
        "getVoteAccounts".to_string(),
        "minimumLedgerSlot".to_string(),
        "requestAirdrop".to_string(),
        "sendTransaction".to_string(),
        "simulateTransaction".to_string(),
        "setLogFilter".to_string(),
        "validatorExit".to_string(),
        "Subscription".to_string(),
        "accountSubscribe".to_string(),
        "accountUnsubscribe".to_string(),
        "logsSubscribe".to_string(),
        "logsUnsubscribe".to_string(),
        "programSubscribe".to_string(),
        "programUnsubscribe".to_string(),
        "signatureSubscribe".to_string(),
        "signatureUnsubscribe".to_string(),
        "slotSubscribe".to_string(),
        "slotUnsubscribe".to_string(),
        "getTokenAccountBalance".to_string(),
        "getTokenAccountsByDelegate".to_string(),
        "getTokenAccountsByOwner".to_string(),
        "getTokenLargestAccounts".to_string(),
        "getTokenSupply".to_string(),
    ]
}
