
mod error;
mod processor;
mod routes;
mod middlewares;

use std::net::ToSocketAddrs;
use actix_web::{
    web,
    App,
    HttpServer,
    middleware,
    client::Client,
};

use mongodb::{
    self,
    options::ClientOptions,
};
use url::Url;
use processor::{
    RequestProcessor,
};
use middlewares::Authorized;

struct Container {
    processor: RequestProcessor,
}

impl Container {
    fn new(processor: RequestProcessor) -> Self {
        Container {
            processor,
        }
    }
}

pub struct State {
    container: Container,
}


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let mongodb_address = "mongodb://localhost:27017";
    let mongodb_name = "secure";
    let listen_address = "127.0.0.1:5003";
    let forward_address = "127.0.0.1";
    let forward_port = 5001;
    let authentication_address = "127.0.0.1";
    let authentication_port = 5002;

    let client_options = ClientOptions::parse(mongodb_address).await.unwrap();
    let client = mongodb::Client::with_options(client_options).unwrap();

    let database = client.database(mongodb_name);
    let requests = database.collection("requests");

    let authentication_url = Url::parse(&format!(
        "http://{}",
        (authentication_address.to_owned().as_str(), authentication_port)
            .to_socket_addrs()
            .unwrap()
            .next()
            .unwrap(),
    ))
    .unwrap();

    let forward_url = Url::parse(&format!(
        "http://{}",
        (forward_address, forward_port)
            .to_socket_addrs()
            .unwrap()
            .next()
            .unwrap(),
    ))
    .unwrap();

    HttpServer::new(move || {
        let container = Container::new(RequestProcessor::new(requests.clone()));

        App::new()
            .wrap(middleware::Logger::default())
            .data(State { container })
            .service(routes::get_all_requests)
            .service(routes::get_requests_by_key)
            .service(
                web::scope("/")
                    .data(Client::new())
                    .data(forward_url.clone())
                    .wrap(Authorized::new(&authentication_url))
                    .default_service(web::route().to(routes::forward)),
            )
    })
    .bind(listen_address)?
    .system_exit()
    .run()
    .await
}
