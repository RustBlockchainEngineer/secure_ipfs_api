use mongodb::{
    bson::doc,
    Collection,
    results::{
        InsertOneResult,
        UpdateResult,
        DeleteResult,
    }
};
use bson::{
        document::ValueAccessError, 
        Document,
        oid::ObjectId
};
use chrono::{
    prelude::*,
    Utc
} ;
use futures::StreamExt;
use uuid::Uuid;
use serde::{
    Deserialize, 
    Serialize
};
use actix_web::{
    http::StatusCode, 
    HttpResponse, 
    ResponseError
};
use std::fmt::{
    Display, 
    Formatter, 
    Result as FmtResult
};
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Key {
    #[serde(rename = "_id")]
    id: String,
    key: Uuid,
    create_time: DateTime<Utc>,
    update_time: DateTime<Utc>,
    enabled: bool,
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub struct NewKey {
    pub key: Uuid,
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub struct UpdateKey {
    pub key: Uuid,
    pub enabled: bool,
}

impl Key {
    pub fn convert_bson_to_key (bson_doc: &Document) -> Result<Self, ValueAccessError> {
        let key = Key {
            id: bson_doc.get_object_id("_id")?.to_hex(),
            key: Uuid::parse_str(bson_doc.get_str("key")?).expect("id parse error"),
            create_time: *bson_doc.get_datetime("create_time")?,
            update_time: *bson_doc.get_datetime("update_time")?,
            enabled: bson_doc.get_bool("enabled")?,
        };
        Ok(key)
    }
}

impl NewKey {
    pub fn create() -> Self {
        NewKey{
            key: Uuid::new_v4(),
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum SimpleApiError {
    #[error("invalid field in BSON document: {0}")]
    InvalidFieldError(#[from] bson::document::ValueAccessError),
    #[error("empty results")]
    MongoDBEmptyResult,
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

impl From<SimpleApiError> for JsonError {
    fn from(err: SimpleApiError) -> Self {
        let status = match err {
            SimpleApiError::MongoDBOperationError { source: _ } | SimpleApiError::InvalidFieldError(_) => 500,
            SimpleApiError::MongoDBEmptyResult => 404,
        };

        JsonError {
            msg: format!("{}", err.to_string()),
            status: status,
            success: false,
        }
    }
}


#[derive(Clone)]
pub struct ApiKeyProcessor {
    collection: Collection,
}

impl ApiKeyProcessor {
    pub fn create(collection: Collection) -> Self {
        ApiKeyProcessor {
            collection,
        }
    }

    pub async fn generate(&self, key: NewKey) -> Result<InsertOneResult, SimpleApiError> {
        let doc = doc! {
            "key": key.key.to_hyphenated().to_string(),
            "update_time": Utc::now(),
            "create_time": Utc::now(),
            "enabled": true,
        };
        let result = self.collection.insert_one(doc, None).await?;
        Ok(result)
    }
    pub async fn update(&self, key: UpdateKey) -> Result<UpdateResult, SimpleApiError> {
        let filter = doc! {
            "key": key.key.to_hyphenated().to_string(),
        };
        let doc = doc!{
            "$set": {
                "update_time": Utc::now(),
                "enabled": key.enabled,
            }
        };
        let result = self.collection.update_one(filter, doc, None).await?;
        Ok(result)
    }
    
    pub async fn get_all(&self) -> Result<Vec<Key>, SimpleApiError> {
        let mut cursor = self.collection.find(None, None).await?;
        let mut result: Vec<Key> = Vec::new();
        while let Some(doc) = cursor.next().await {
            result.push(Key::convert_bson_to_key(&doc?)?);
        }
        Ok(result)
    }
    pub async fn get_key(&self, key: &str) -> Result<Key, SimpleApiError> {
        let filter = doc! {
            "key": key.to_string(),
        };
        let doc = self.collection.find_one(filter, None).await?;
        let result = doc.ok_or(SimpleApiError::MongoDBEmptyResult)?;
        let apikey = Key::convert_bson_to_key(&result)?;

        Ok(apikey)
    }

    pub async fn get_key_from_id(&self, id: &ObjectId) -> Result<Key, SimpleApiError> {
        let filter = doc! {
            "_id": id,
        };
        let doc = self.collection.find_one(filter, None).await?;
        let result = doc.ok_or(SimpleApiError::MongoDBEmptyResult)?;
        let apikey = Key::convert_bson_to_key(&result)?;

        Ok(apikey)
    }
    pub async fn delete(&self, key: &str) -> Result<DeleteResult, SimpleApiError> {
        let filter = doc! {
            "key": key.to_string(),
        };
        let result = self.collection.delete_one(filter, None).await?;
        Ok(result)
    }
}