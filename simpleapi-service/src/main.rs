use actix_web::{
    self,
    web,
    HttpServer,
};
use mongodb::{
    options::ClientOptions,
    Client,
};
use processor::ApiKeyProcessor;

pub mod routes;
pub mod processor;

struct Container {
    key: ApiKeyProcessor,
}

impl Container {
    fn create(key:ApiKeyProcessor) -> Self {
        Container {
            key
        }
    }
}

struct State {
    container: Container
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let address = "127.0.0.1:5002";
    let mongodb_address = "mongodb://localhost:27017";
    let mongodb_name = "secure";
    let options = ClientOptions::parse(mongodb_address)
        .await
        .unwrap();
    let client = Client::with_options(options)
        .unwrap();
    let database = client.database(mongodb_name);
    let keys = database.collection("keys");

    print!("SimpleAPI keys Listening {} ...", address);

    HttpServer::new(move || {
        let container = Container::create(
            ApiKeyProcessor::create(keys.clone())
        );
        actix_web::App::new()
            .service(
                web::scope("/keys")
                .data(State{
                    container,
                })
                .service(routes::get_keys)
                .service(routes::get_key)
                .service(routes::create)
                .service(routes::update)
                .service(routes::delete),
            )
    })
    .bind(address)?
    .run()
    .await
}
