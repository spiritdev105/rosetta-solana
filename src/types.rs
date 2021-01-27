use serde::{Deserialize, Serialize};
use solana_client::rpc_request::RpcRequest;
use solana_sdk::{fee_calculator::FeeCalculator, hash::Hash};
use solana_transaction_status::UiTransactionStatusMeta;

use crate::operations::matcher::{InternalOperation, InternalOperationMetadata};

// Objects

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CallRequest {
    pub network_identifier: NetworkIdentifier,
    pub method: RpcRequestInternal,
    pub parameters: serde_json::Value,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CallResponse {
    pub result: serde_json::Value,
    pub idempotent: bool,
}

//TODO: orphan rule cant impl deserialize for RpcRequest
#[derive(Clone, Debug, Deserialize, Serialize, strum::EnumIter, strum_macros::EnumString)]
pub enum RpcRequestInternal {
    DeregisterNode,
    ValidatorExit,
    GetAccountInfo,
    GetBalance,
    GetBlockTime,
    GetClusterNodes,
    GetConfirmedBlock,
    GetConfirmedBlocks,
    GetConfirmedBlocksWithLimit,
    GetConfirmedSignaturesForAddress,
    GetConfirmedSignaturesForAddress2,
    GetConfirmedTransaction,
    GetEpochInfo,
    GetEpochSchedule,
    GetFeeCalculatorForBlockhash,
    GetFeeRateGovernor,
    GetFees,
    GetFirstAvailableBlock,
    GetGenesisHash,
    GetHealth,
    GetIdentity,
    GetInflationGovernor,
    GetInflationRate,
    GetLargestAccounts,
    GetLeaderSchedule,
    GetMinimumBalanceForRentExemption,
    GetMultipleAccounts,
    GetProgramAccounts,
    GetRecentBlockhash,
    GetSnapshotSlot,
    GetSignatureStatuses,
    GetSlot,
    GetSlotLeader,
    GetStorageTurn,
    GetStorageTurnRate,
    GetSlotsPerSegment,
    GetStoragePubkeysForSlot,
    GetSupply,
    GetTokenAccountBalance,
    GetTokenAccountsByDelegate,
    GetTokenAccountsByOwner,
    GetTokenSupply,
    GetTotalSupply,
    GetTransactionCount,
    GetVersion,
    GetVoteAccounts,
    MinimumLedgerSlot,
    RegisterNode,
    RequestAirdrop,
    SendTransaction,
    SimulateTransaction,
    SignVote,
}

impl From<RpcRequestInternal> for RpcRequest {
    fn from(r: RpcRequestInternal) -> Self {
        match r {
            RpcRequestInternal::DeregisterNode => RpcRequest::DeregisterNode,
            RpcRequestInternal::ValidatorExit => RpcRequest::ValidatorExit,
            RpcRequestInternal::GetAccountInfo => RpcRequest::GetAccountInfo,
            RpcRequestInternal::GetBalance => RpcRequest::GetBalance,
            RpcRequestInternal::GetBlockTime => RpcRequest::GetBlockTime,
            RpcRequestInternal::GetClusterNodes => RpcRequest::GetClusterNodes,
            RpcRequestInternal::GetConfirmedBlock => RpcRequest::GetConfirmedBlock,
            RpcRequestInternal::GetConfirmedBlocks => RpcRequest::GetConfirmedBlocks,
            RpcRequestInternal::GetConfirmedBlocksWithLimit => {
                RpcRequest::GetConfirmedBlocksWithLimit
            }
            RpcRequestInternal::GetConfirmedSignaturesForAddress => {
                RpcRequest::GetConfirmedSignaturesForAddress
            }
            RpcRequestInternal::GetConfirmedSignaturesForAddress2 => {
                RpcRequest::GetConfirmedSignaturesForAddress2
            }
            RpcRequestInternal::GetConfirmedTransaction => RpcRequest::GetConfirmedTransaction,
            RpcRequestInternal::GetEpochInfo => RpcRequest::GetEpochInfo,
            RpcRequestInternal::GetEpochSchedule => RpcRequest::GetEpochSchedule,
            RpcRequestInternal::GetFeeCalculatorForBlockhash => {
                RpcRequest::GetFeeCalculatorForBlockhash
            }
            RpcRequestInternal::GetFeeRateGovernor => RpcRequest::GetFeeRateGovernor,
            RpcRequestInternal::GetFees => RpcRequest::GetFees,
            RpcRequestInternal::GetFirstAvailableBlock => RpcRequest::GetFirstAvailableBlock,
            RpcRequestInternal::GetGenesisHash => RpcRequest::GetGenesisHash,
            RpcRequestInternal::GetHealth => RpcRequest::GetHealth,
            RpcRequestInternal::GetIdentity => RpcRequest::GetIdentity,
            RpcRequestInternal::GetInflationGovernor => RpcRequest::GetInflationGovernor,
            RpcRequestInternal::GetInflationRate => RpcRequest::GetInflationRate,
            RpcRequestInternal::GetLargestAccounts => RpcRequest::GetLargestAccounts,
            RpcRequestInternal::GetLeaderSchedule => RpcRequest::GetLeaderSchedule,
            RpcRequestInternal::GetMinimumBalanceForRentExemption => {
                RpcRequest::GetMinimumBalanceForRentExemption
            }
            RpcRequestInternal::GetMultipleAccounts => RpcRequest::GetMultipleAccounts,
            RpcRequestInternal::GetProgramAccounts => RpcRequest::GetProgramAccounts,
            RpcRequestInternal::GetRecentBlockhash => RpcRequest::GetRecentBlockhash,
            RpcRequestInternal::GetSnapshotSlot => RpcRequest::GetSnapshotSlot,
            RpcRequestInternal::GetSignatureStatuses => RpcRequest::GetSignatureStatuses,
            RpcRequestInternal::GetSlot => RpcRequest::GetSlot,
            RpcRequestInternal::GetSlotLeader => RpcRequest::GetSlotLeader,
            RpcRequestInternal::GetStorageTurn => RpcRequest::GetStorageTurn,
            RpcRequestInternal::GetStorageTurnRate => RpcRequest::GetStorageTurnRate,
            RpcRequestInternal::GetSlotsPerSegment => RpcRequest::GetSlotsPerSegment,
            RpcRequestInternal::GetStoragePubkeysForSlot => RpcRequest::GetStoragePubkeysForSlot,
            RpcRequestInternal::GetSupply => RpcRequest::GetSupply,
            RpcRequestInternal::GetTokenAccountBalance => RpcRequest::GetTokenAccountBalance,
            RpcRequestInternal::GetTokenAccountsByDelegate => {
                RpcRequest::GetTokenAccountsByDelegate
            }
            RpcRequestInternal::GetTokenAccountsByOwner => RpcRequest::GetTokenAccountsByOwner,
            RpcRequestInternal::GetTokenSupply => RpcRequest::GetTokenSupply,
            RpcRequestInternal::GetTotalSupply => RpcRequest::GetTotalSupply,
            RpcRequestInternal::GetTransactionCount => RpcRequest::GetTransactionCount,
            RpcRequestInternal::GetVersion => RpcRequest::GetVersion,
            RpcRequestInternal::GetVoteAccounts => RpcRequest::GetVoteAccounts,
            RpcRequestInternal::MinimumLedgerSlot => RpcRequest::MinimumLedgerSlot,
            RpcRequestInternal::RegisterNode => RpcRequest::RegisterNode,
            RpcRequestInternal::RequestAirdrop => RpcRequest::RequestAirdrop,
            RpcRequestInternal::SendTransaction => RpcRequest::SendTransaction,
            RpcRequestInternal::SimulateTransaction => RpcRequest::SimulateTransaction,
            RpcRequestInternal::SignVote => RpcRequest::SignVote,
        }
    }
}
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Allow {
    pub operation_statuses: Vec<OperationStatus>,
    pub operation_types: Vec<OperationType>,
    pub errors: Vec<Error>,
    pub historical_balance_lookup: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp_start_index: Option<u64>,
    pub call_methods: Vec<RpcRequestInternal>,
    pub balance_exemptions: Vec<BalanceExemption>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct BalanceExemption {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sub_account_address: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub currency: Option<Currency>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exemption_type: Option<ExemptionType>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum ExemptionType {
    #[serde(rename = "greater_or_equal")]
    GreaterOrEqual,
    #[serde(rename = "less_or_equal")]
    LessOrEqual,
    #[serde(rename = "dynamic")]
    Dynanic,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Amount {
    pub value: String,
    pub currency: Currency,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Block {
    pub block_identifier: BlockIdentifier,
    pub parent_block_identifier: BlockIdentifier,
    pub timestamp: u64,
    pub transactions: Vec<Transaction>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Currency {
    pub symbol: String,
    pub decimals: u8,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Error {
    pub code: u64,
    pub message: String,
    pub retriable: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<ErrorDetails>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Operation {
    pub operation_identifier: OperationIdentifier,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub related_operations: Option<Vec<OperationIdentifier>>,
    #[serde(rename = "type")]
    pub type_: OperationType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<OperationStatusType>, //TODO: sucess/faliure for now
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account: Option<AccountIdentifier>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub amount: Option<Amount>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::value::Value>,
}

#[derive(
    Clone,
    Debug,
    Deserialize,
    Serialize,
    PartialEq,
    strum::EnumIter,
    strum::AsStaticStr,
    strum_macros::EnumString,
)]
pub enum OperationType {
    System__CreateAccount,
    System__Assign,
    System__Transfer,
    //    System__CreateAccountWithSeed,
    System__CreateNonceAccount,
    System__AdvanceNonceAccount,
    System__WithdrawNonceAccount,
    System__AuthorizeNonceAccount,
    System__Allocate,
    //System__AllocateWithSeed,
    //System__AssignWithSeed,
    //System__TransferWithSeed,
    SplToken__InitializeMint,
    SplToken__InitializeAccount,
    SplToken__CreateToken,
    SplToken__CreateAccount,
    SplToken__Transfer,
    SplToken__Approve,
    SplToken__Revoke,
    //SplToken__SetAuthority,
    SplToken__MintTo,
    SplToken__Burn,
    SplToken__CloseAccount,
    SplToken__FreezeAccount,
    SplToken__ThawAccount,
    SplToken__TransferChecked,
    SplToken__CreateAssocAccount,

    Stake__CreateAccount,
    Stake__Delegate,
    Stake__Split,
    Stake__Merge,
    Stake__Authorize,
    Stake__Withdraw,
    Stake__Deactivate,
    Stake__SetLockup,

    Vote__CreateAccount,
    Vote__Authorize,
    //Vote__Vote,
    Vote__Withdraw,
    Vote__UpdateValidatorIdentity,
    Vote__UpdateCommission,
    //Vote__VoteSwitch,
    Unknown,
}
#[derive(
    Clone, Debug, Deserialize, Serialize, PartialEq, strum::EnumIter, strum_macros::EnumString,
)]
pub enum OperationStatusType {
    Success,
    Faliure,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SigningPayload {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account_identifier: Option<AccountIdentifier>,
    pub hex_bytes: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature_type: Option<SignatureType>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PublicKey {
    pub hex_bytes: String,
    pub curve_type: CurveType,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Signature {
    pub signing_payload: SigningPayload,
    pub public_key: PublicKey,
    pub signature_type: SignatureType,
    pub hex_bytes: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Transaction {
    pub transaction_identifier: TransactionIdentifier,
    pub operations: Vec<Operation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<UiTransactionStatusMeta>,
}

// Identifiers

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AccountIdentifier {
    pub address: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sub_account: Option<SubAccountIdentifier>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct BlockIdentifier {
    pub index: u64,
    pub hash: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct NetworkIdentifier {
    pub blockchain: String,
    pub network: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sub_network_identifier: Option<SubNetworkIdentifier>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct OperationIdentifier {
    pub index: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub network_index: Option<u64>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PartialBlockIdentifier {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub index: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hash: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SubAccountIdentifier {
    pub address: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SubNetworkIdentifier {
    pub network: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TransactionIdentifier {
    pub hash: String,
}

// Requests and Rseponses

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AccountBalanceRequest {
    pub network_identifier: NetworkIdentifier,
    pub account_identifier: AccountIdentifier,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub block_identifier: Option<PartialBlockIdentifier>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub currencies: Option<Vec<Currency>>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AccountBalanceResponse {
    pub block_identifier: BlockIdentifier,
    pub balances: Vec<Amount>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct BlockRequest {
    pub network_identifier: NetworkIdentifier,
    pub block_identifier: PartialBlockIdentifier,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct BlockResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub block: Option<Block>,
    //pub other_transactions: Vec<TransactionIdentifier>
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct BlockTransactionRequest {
    pub network_identifier: NetworkIdentifier,
    pub block_identifier: BlockIdentifier,
    pub transaction_identifier: TransactionIdentifier,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct BlockTransactionResponse {
    pub transaction: Transaction,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ConstructionCombineRequest {
    pub network_identifier: NetworkIdentifier,
    pub unsigned_transaction: String,
    pub signatures: Vec<Signature>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ConstructionCombineResponse {
    pub signed_transaction: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ConstructionDeriveRequest {
    pub network_identifier: NetworkIdentifier,
    pub public_key: PublicKey,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ConstructionDeriveResponse {
    pub account_identifier: AccountIdentifier,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ConstructionHashRequest {
    pub network_identifier: NetworkIdentifier,
    pub signed_transaction: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ConstructionMetadataRequest {
    pub network_identifier: NetworkIdentifier,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<MetadataOptions>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ConstructionMetadataResponse {
    pub metadata: ConstructionMetadata,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ConstructionParseRequest {
    pub network_identifier: NetworkIdentifier,
    pub signed: bool,
    pub transaction: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ConstructionParseResponse {
    pub operations: Vec<Operation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account_identifier_signers: Option<Vec<AccountIdentifier>>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ConstructionPayloadsRequest {
    pub network_identifier: NetworkIdentifier,
    pub operations: Vec<Operation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<ConstructionMetadata>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ConstructionPayloadsResponse {
    pub unsigned_transaction: String,
    pub payloads: Vec<Option<SigningPayload>>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ConstructionPreprocessRequest {
    pub network_identifier: NetworkIdentifier,
    pub operations: Vec<Operation>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ConstructionPreprocessResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<MetadataOptions>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ConstructionSubmitRequest {
    pub network_identifier: NetworkIdentifier,
    pub signed_transaction: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ConstructionSubmitResponse {
    pub transaction_identifier: TransactionIdentifier,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MempoolRequest {
    pub network_identifier: NetworkIdentifier,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MempoolResponse {
    pub transaction_identifiers: Vec<TransactionIdentifier>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MempoolTransactionRequest {
    pub network_identifier: NetworkIdentifier,
    pub transaction_identifier: TransactionIdentifier,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MempoolTransactionResponse {
    pub transaction: Transaction,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MetadataRequest {}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct NetworkListResponse {
    pub network_identifiers: Vec<NetworkIdentifier>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct NetworkOptionsResponse {
    pub version: Version,
    pub allow: Allow,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct NetworkRequest {
    pub network_identifier: NetworkIdentifier,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct NetworkStatusResponse {
    pub current_block_identifier: BlockIdentifier,
    pub current_block_timestamp: u64,
    pub genesis_block_identifier: BlockIdentifier,
    pub peers: Vec<Peer>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TransactionIdentifierResponse {
    pub transaction_identifier: TransactionIdentifier,
}

// Miscellaneous

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum CurveType {
    #[serde(rename = "secp256k1")]
    Secp256k1,
    #[serde(rename = "secp256r1")]
    Secp256r1,
    #[serde(rename = "edwards25519")]
    Edwards25519,
    #[serde(rename = "tweedle")]
    Tweedle,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct OperationStatus {
    pub status: OperationStatusType,
    pub successful: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Peer {
    pub peer_id: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum SignatureType {
    #[serde(rename = "ecdsa")]
    ECDSA,
    #[serde(rename = "ecdsa_recovery")]
    ECDSARecovery,
    #[serde(rename = "ed25519")]
    Ed25519,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Version {
    pub rosetta_version: String,
    pub node_version: String,
    pub middleware_version: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MetadataOptions {
    pub internal_operations: Vec<InternalOperation>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ErrorDetails {
    /// The detailed error
    pub error: String,
}

pub type OptionalInternalOperationMetadatas = Option<Vec<Option<InternalOperationMetadata>>>;
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ConstructionMetadata {
    pub blockhash: String,
    pub fee_calculator: FeeCalculator,
    pub internal_meta: OptionalInternalOperationMetadatas,
}
