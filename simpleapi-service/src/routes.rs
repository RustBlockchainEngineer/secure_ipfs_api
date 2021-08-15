use actix_web::{
    web, 
    HttpResponse,
    get, 
    post, 
    put, 
    delete, 
};
use serde_json::json;
use super::processor::{
    NewKey,
    UpdateKey,
};
use super::processor::JsonError;


#[get("")]
async fn get_keys(app_data: web::Data<crate::State>) -> Result<HttpResponse, JsonError> {
    let result = app_data.container.key.get_all().await;
    match result {
        Ok(keys) => Ok(HttpResponse::Ok().json(json!({
            "status": 200,
            "sucess": true,
            "payload": keys,
        }))),
        Err(e) => Err(e.into()),
    }
}

#[get("/{key}")]
async fn get_key(
    key: web::Path<String>,
    app_data: web::Data<crate::State>,
) -> Result<HttpResponse, JsonError> {
    let result = app_data.container.key.get_key(&key).await;
    match result {
        Ok(apikey) => Ok(HttpResponse::Ok().json(json!({
            "status": 200,
            "success": true,
            "payload": apikey,
        }))),
        Err(e) => Err(e.into()),
    }
}


#[post("")]
async fn create(app_data: web::Data<crate::State>) -> Result<HttpResponse, JsonError> {
    let apikey = NewKey::create();
    let result = app_data.container.key.generate(apikey).await;
    match result {
        Ok(inserted) => {
            let id = inserted.inserted_id.as_object_id().ok_or(JsonError {
                msg: format!("Insert failed"),
                status: 500,
                success: false,
            })?;
            let apikey = app_data.container.key.get_key_from_id(&id).await;
            match apikey {
                Ok(key) => Ok(HttpResponse::Ok().json(json!({
                    "status": 200,
                    "success": true,
                    "payload": key,
                }))),
                Err(e) => Err(e.into()),
            }
        }
        Err(e) => Err(e.into()),
    }
}

#[put("")]
async fn update(
    apikey: web::Json<UpdateKey>,
    app_data: web::Data<crate::State>,
) -> Result<HttpResponse, JsonError> {
    let apikey = apikey.into_inner();
    let result = app_data.container.key.update(apikey).await;
    match result {
        Ok(_) => {
            // Result does not return an upserted_id, so we play nice by fetching by key
            // And returning the changed object
            let key = apikey.key.to_hyphenated().to_string();
            let apikey = app_data.container.key.get_key(&key).await;
            match apikey {
                Ok(key) => Ok(HttpResponse::Ok().json(json!({
                    "status": 200,
                    "success": true,
                    "payload": key,
                }))),
                Err(e) => Err(e.into()),
            }
        }
        Err(e) => Err(e.into()),
    }
}

#[delete("/{key}")]
async fn delete(
    key: web::Path<String>,
    app_data: web::Data<crate::State>,
) -> Result<HttpResponse, JsonError> {
    let result = app_data.container.key.delete(&key).await;
    match result {
        Ok(res) => {
            if res.deleted_count == 0 {
                Err(JsonError {
                    msg: format!("Key not found: {}", key),
                    status: 404,
                    success: false,
                })
            } else {
                Ok(HttpResponse::Ok().json(json!({
                    "status": 200,
                    "success": true,
                    "payload": {
                        "count": res.deleted_count,
                    },
                })))
            }
        }
        Err(e) => Err(e.into()),
    }
}
