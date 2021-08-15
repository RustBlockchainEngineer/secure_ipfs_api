use actix_web::{
    http::StatusCode, 
    HttpResponse, 
    ResponseError
};
use serde::Serialize;
use std::fmt::{
    Display, 
    Formatter, 
    Result as FmtResult
};


#[derive(thiserror::Error, Debug)]
pub enum ProxyError {
    #[error("invalid field in BSON document: {0}")]
    InvalidFieldError(#[from] bson::document::ValueAccessError),
    #[error("Operation failed: {source}")]
    MongoDBOperationError {
        #[from]
        source: mongodb::error::Error,
    },
}

#[derive(Debug, Serialize)]
pub struct JsonError {
    pub msg: String,
    pub status: u16,
    pub success: bool,
}

impl Display for JsonError {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        let err_json = serde_json::to_string(self).unwrap();
        write!(f, "{}", err_json)
    }
}

impl ResponseError for JsonError {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(StatusCode::from_u16(self.status).unwrap()).json2(self)
    }
}

impl From<ProxyError> for JsonError {
    fn from(err: ProxyError) -> Self {
        let status = match err {
            ProxyError::MongoDBOperationError { source: _ } | ProxyError::InvalidFieldError(_) => 500,
        };

        JsonError {
            msg: format!("{}", err.to_string()),
            status: status,
            success: false,
        }
    }
}
