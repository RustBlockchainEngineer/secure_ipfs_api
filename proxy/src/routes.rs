use actix_web::{
    get, 
    web, 
    Error, 
    HttpRequest, 
    HttpResponse,
    client::Client,
};
use url::Url;
use serde_json::json;
use super::error::JsonError;
use super::processor::*;

pub async fn forward(
    req: HttpRequest,
    body: web::Bytes,
    forward_url: web::Data<Url>,
    app_data: web::Data<crate::State>,
    client: web::Data<Client>,
) -> Result<HttpResponse, Error> {
    let request = NewRequest::from_http_request(&req).unwrap();
    let result = app_data.container.processor.create(request).await;

    match result {
        Ok(_) => println!("Request logged"),
        Err(e) => println!("Error logging request: {}", e),
    };

    println!("Processing ...");
    let mut new_url = forward_url.as_ref().clone();
    new_url.set_path(req.uri().path());
    new_url.set_query(req.uri().query());

    println!("Forwarded request URL: {:?}", new_url);
    
    let forwarded_req = client
        .request_from(new_url.as_str(), req.head())
        .no_decompress();
    let forwarded_req = if let Some(addr) = req.head().peer_addr {
        forwarded_req.header("x-forwarded-for", format!("{}", addr.ip()))
    } else {
        forwarded_req
    };

    let mut res = forwarded_req.send_body(body).await.map_err(Error::from)?;

    let mut client_resp = HttpResponse::build(res.status());
    
    for (header_name, header_value) in res.headers().iter().filter(|(h, _)| *h != "connection") {
        client_resp.header(header_name.clone(), header_value.clone());
    }

    Ok(client_resp.body(res.body().await?))
}

#[get("/requests")]
pub async fn get_all_requests(
    app_data: web::Data<crate::State>,
) -> Result<HttpResponse, JsonError> {
    let result = app_data.container.processor.get_all().await;
    match result {
        Ok(requests) => Ok(HttpResponse::Ok().json(json!({
            "status": 200,
            "sucess": true,
            "payload": requests,
        }))),
        Err(e) => Err(e.into()),
    }
}

#[get("/requests/{key}")]
pub async fn get_requests_by_key(
    key: web::Path<String>,
    app_data: web::Data<crate::State>,
) -> Result<HttpResponse, JsonError> {
    let result = app_data.container.processor.get_by_key(&key).await;
    match result {
        Ok(requests) => Ok(HttpResponse::Ok().json(json!({
            "status": 200,
            "success": true,
            "payload": requests,
        }))),
        Err(e) => Err(e.into()),
    }
}
