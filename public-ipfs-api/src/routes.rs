use std::{
    boxed::Box,
    io::Cursor,
};

use actix_web::{
    error, 
    get, 
    post, 
    web, 
    Error, 
    HttpResponse
};

use futures::StreamExt;
use ipfs_api::IpfsClient;
use serde::Serialize;

const MAX_SIZE: usize = 262144;

#[derive(Serialize)]
struct IpfsResponse {
    hash: String,
    name: String,
    size: String,
}

#[get("")]
async fn index(_client: web::Data<IpfsClient>) -> HttpResponse {
    HttpResponse::Ok().body("it works as post")
}

#[post("")]
async fn test_upload(mut payload: web::Payload, client: web::Data<IpfsClient>) -> Result<HttpResponse, Error> {
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        if chunk.len() + body.len() > MAX_SIZE {
            return Err(error::ErrorBadRequest("overflow"));
        }
        body.extend_from_slice(&chunk);
    }
    let data = Cursor::new(body);
    let boxed = Box::new(data);
    match client.add(*boxed).await {
        Ok(res) => Ok(HttpResponse::Ok().json(IpfsResponse {
            hash: res.hash,
            name: res.name,
            size: res.size,
        })),
        Err(e) => Err(error::ErrorInternalServerError(format!(
            "Internal Server Error: {:?}",
            e
        )))
    }
}
