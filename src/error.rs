use std::io::Cursor;

use crate::types::{self, ErrorDetails};
use rocket::http::{hyper::StatusCode, Status};
use solana_client::client_error::ClientError;
use solana_sdk::{
    program_error::ProgramError, pubkey::ParsePubkeyError, signature::ParseSignatureError,
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ApiError {
    #[error("{0}")]
    //PlaceHolder for any errors
    PlaceHolderError(String),
    #[error("bad request")]
    BadRequest,
    #[error("programerror {0}")]
    ProgramError(#[from] ProgramError),
    #[error("curve not supported")]
    UnsupportedCurve,
    #[error("invalid signed transaction")]
    InvalidSignedTransaction,
    #[error("bad network")]
    BadNetwork,
    #[error("deserialization failed: {0}")]
    DeserializationFailed(String),
    //#[error("serialization failed: {0:?}")]
    //SerializationFailed(#[from] bcs::Error),
    #[error("bad operations {0}")]
    BadOperations(String),
    #[error("account not found")]
    AccountNotFound,
    #[error("system time error: {0:?}")]
    SystemTimeError(#[from] std::time::SystemTimeError),
    #[error("hex decoding failed: {0:?}")]
    HexDecodingFailed(#[from] hex::FromHexError),
    #[error("bad signature")]
    BadSignature,
    #[error("bad signature type")]
    BadSignatureType,
    #[error("bad transaction script")]
    BadTransactionScript,
    #[error("bad transaction payload")]
    BadTransactionPayload,
    #[error("bad coin")]
    BadCoin,
    #[error("bad siganture count")]
    BadSignatureCount,
    #[error("historic balances unsupported")]
    HistoricBalancesUnsupported,
    #[error("ParsePubkeyError")]
    ParsePubkeyError(#[from] ParsePubkeyError),
    #[error(transparent)]
    RpcClientError(#[from] ClientError),
    #[error("ParseSignatureError")]
    ParseSignatureError(#[from] ParseSignatureError),
    #[error("Base64DecodeError")]
    Base64DecodeError(#[from] base64::DecodeError),
}
impl ApiError {
    pub fn code(&self) -> u64 {
        match self {
            ApiError::PlaceHolderError(_) => 19,
            ApiError::BadRequest => 20,
            ApiError::UnsupportedCurve => 21,
            ApiError::InvalidSignedTransaction => 22,
            ApiError::BadNetwork => 40,
            ApiError::DeserializationFailed(_) => 50,
            //ApiError::SerializationFailed(_) => 60,
            ApiError::BadOperations(_) => 70,
            ApiError::AccountNotFound => 80,
            ApiError::SystemTimeError(_) => 90,
            ApiError::HexDecodingFailed(_) => 100,
            ApiError::BadSignature => 110,
            ApiError::BadSignatureType => 120,
            ApiError::BadTransactionScript => 130,
            ApiError::BadTransactionPayload => 140,
            ApiError::BadCoin => 150,
            ApiError::BadSignatureCount => 160,
            ApiError::HistoricBalancesUnsupported => 170,
            ApiError::RpcClientError(_) => 180,
            ApiError::ParsePubkeyError(_) => 190,
            ApiError::ParseSignatureError(_) => 200,
            ApiError::Base64DecodeError(_) => 210,
            ApiError::ProgramError(_) => 220,
        }
    }

    pub fn retriable(&self) -> bool {
        match self {
            ApiError::PlaceHolderError(_) => false,
            ApiError::BadRequest => false,
            ApiError::UnsupportedCurve => false,
            ApiError::InvalidSignedTransaction => false,
            ApiError::BadNetwork => false,
            ApiError::DeserializationFailed(_) => false,
            //ApiError::SerializationFailed(_) => false,
            ApiError::BadOperations(_) => false,
            ApiError::AccountNotFound => true,
            ApiError::SystemTimeError(_) => true,
            ApiError::HexDecodingFailed(_) => false,
            ApiError::BadSignature => false,
            ApiError::BadSignatureType => false,
            ApiError::BadTransactionScript => false,
            ApiError::BadTransactionPayload => false,
            ApiError::BadCoin => false,
            ApiError::BadSignatureCount => false,
            ApiError::HistoricBalancesUnsupported => false,
            ApiError::RpcClientError(_) => false,
            ApiError::ParsePubkeyError(_) => false,
            ApiError::ParseSignatureError(_) => false,
            ApiError::Base64DecodeError(_) => false,
            ApiError::ProgramError(_) => false,
        }
    }

    pub fn status_code(&self) -> StatusCode {
        match self {
            ApiError::PlaceHolderError(_) => StatusCode::InternalServerError,
            ApiError::BadRequest => StatusCode::BadRequest,
            ApiError::UnsupportedCurve => StatusCode::InternalServerError,
            ApiError::InvalidSignedTransaction => StatusCode::InternalServerError,
            ApiError::BadNetwork => StatusCode::BadRequest,
            ApiError::DeserializationFailed(_) => StatusCode::BadRequest,
            //ApiError::SerializationFailed(_) => StatusCode::BadRequest,
            ApiError::BadOperations(_) => StatusCode::BadRequest,
            ApiError::AccountNotFound => StatusCode::NotFound,
            ApiError::SystemTimeError(_) => StatusCode::InternalServerError,
            ApiError::HexDecodingFailed(_) => StatusCode::BadRequest,
            ApiError::BadSignature => StatusCode::BadRequest,
            ApiError::BadSignatureType => StatusCode::BadRequest,
            ApiError::BadTransactionScript => StatusCode::BadRequest,
            ApiError::BadTransactionPayload => StatusCode::BadRequest,
            ApiError::BadCoin => StatusCode::BadRequest,
            ApiError::BadSignatureCount => StatusCode::BadRequest,
            ApiError::HistoricBalancesUnsupported => StatusCode::BadRequest,
            ApiError::RpcClientError(_) => StatusCode::InternalServerError,
            ApiError::ParsePubkeyError(_) => StatusCode::InternalServerError,
            ApiError::ParseSignatureError(_) => StatusCode::InternalServerError,
            ApiError::Base64DecodeError(_) => StatusCode::InternalServerError,
            ApiError::ProgramError(_) => StatusCode::InternalServerError,
        }
    }

    pub fn message(&self) -> String {
        let full = format!("{}", self);
        let parts: Vec<_> = full.split(":").collect();
        parts[0].to_string()
    }

    pub(crate) fn details(&self) -> ErrorDetails {
        let error = format!("{}", self);
        ErrorDetails { error }
    }

    pub fn deserialization_failed(type_: &str) -> ApiError {
        ApiError::DeserializationFailed(type_.to_string())
    }

    pub(crate) fn all_errors() -> Vec<types::Error> {
        vec![
            types::Error {
                message: "bad block request".to_string(),
                code: 20,
                retriable: false,
                details: None,
            },
            types::Error {
                message: "bad network".to_string(),
                code: 40,
                retriable: false,
                details: None,
            },
            types::Error {
                message: "deserialization failed".to_string(),
                code: 50,
                retriable: false,
                details: None,
            },
            types::Error {
                message: "serialization failed".to_string(),
                code: 60,
                retriable: false,
                details: None,
            },
            types::Error {
                message: "bad transfer operations".to_string(),
                code: 70,
                retriable: false,
                details: None,
            },
            types::Error {
                message: "account not found".to_string(),
                code: 80,
                retriable: false,
                details: None,
            },
            types::Error {
                message: "system time error".to_string(),
                code: 90,
                retriable: true,
                details: None,
            },
            types::Error {
                message: "hex decoding failed".to_string(),
                code: 100,
                retriable: false,
                details: None,
            },
            types::Error {
                message: "bad signature".to_string(),
                code: 110,
                retriable: false,
                details: None,
            },
            types::Error {
                message: "bad signature type".to_string(),
                code: 120,
                retriable: false,
                details: None,
            },
            types::Error {
                message: "bad transaction script".to_string(),
                code: 130,
                retriable: false,
                details: None,
            },
            types::Error {
                message: "bad transaction payload".to_string(),
                code: 140,
                retriable: false,
                details: None,
            },
            types::Error {
                message: "bad coin".to_string(),
                code: 150,
                retriable: false,
                details: None,
            },
            types::Error {
                message: "bad signature count".to_string(),
                code: 160,
                retriable: false,
                details: None,
            },
            types::Error {
                message: "historic balances unsupported".to_string(),
                code: 170,
                retriable: false,
                details: None,
            },
        ]
    }

    pub fn into_error(self) -> types::Error {
        types::Error {
            message: self.message(),
            code: self.code(),
            retriable: self.retriable(),
            details: Some(self.details()),
        }
    }
}

impl<'r> rocket::response::Responder<'r> for ApiError {
    fn respond_to(self, request: &rocket::Request) -> Result<rocket::Response<'r>, Status> {
        Ok(rocket::Response::build()
            .header(rocket::http::ContentType::JSON)
            .status(Status::InternalServerError)
            .sized_body(Cursor::new(format!(
                "{}",
                serde_json::to_string(&self.into_error()).unwrap()
            )))
            .finalize())
    }
}
