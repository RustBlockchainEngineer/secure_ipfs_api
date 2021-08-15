use actix_web::{
    self,
    web,
    HttpServer
};
use ipfs_api::IpfsClient;

pub mod routes;

#[actix_web::main]
async fn main() -> std::io::Result<()>{

    let ip_address = "127.0.0.1:5001";

    print!("listening {}", ip_address);

    HttpServer::new(|| {
        actix_web::App::new().data(IpfsClient::default())
        .service(
            web::scope("/")
            .service(routes::index)
            .service(routes::test_upload)
        )
    })
    .bind(ip_address)?
    .run()
    .await
}
