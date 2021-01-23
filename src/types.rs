use serde::{Deserialize, Serialize};
use solana_sdk::{fee_calculator::FeeCalculator, hash::Hash};
use solana_transaction_status::UiTransactionStatusMeta;

// Objects

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Allow {
    pub operation_statuses: Vec<OperationStatus>,
    pub operation_types: Vec<OperationType>,
    pub errors: Vec<Error>,
    pub historical_balance_lookup: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp_start_index: Option<u64>,
    pub call_methods: Vec<String>,
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
    pub status: Option<OperationStatusType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account: Option<AccountIdentifier>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub amount: Option<Amount>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::value::Value>,
}

#[derive(
    Clone, Debug, Deserialize, Serialize, PartialEq, strum::EnumIter, strum_macros::EnumString,
)]
pub enum OperationType {
    System__Transfer,

    SplToken__Transfer,
    SplToken__TransferChecked,
    SplToken__CreateAssocAccount,
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
    pub payloads: Vec<SigningPayload>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ConstructionPreprocessRequest {
    pub network_identifier: NetworkIdentifier,
    pub operations: Vec<Operation>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ConstructionPreprocessResponse {
    pub options: MetadataOptions,
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
pub struct MetadataOptions {}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ErrorDetails {
    /// The detailed error
    pub error: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ConstructionMetadata {
    pub blockhash: String,
    pub fee_calculator: FeeCalculator,
}
