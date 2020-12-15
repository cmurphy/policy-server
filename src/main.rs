use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};

use std::fs::File;
use std::io::prelude::*;


mod wapc_pool;
use wapc_pool::WapcManager;


fn load_file(path: &str) -> Vec<u8> {
    println!("{}", path);
    let mut f = File::open(path).unwrap();
    let mut buf = Vec::new();
    f.read_to_end(&mut buf).unwrap();
    buf
}

#[get("/")]
async fn hello(pool: web::Data<WapcManager>) -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

type Pool = deadpool::managed::Pool<WapcManager, Box<dyn std::error::Error>>;


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let module_bytes = load_file("test.wasm");
    let mgr = WapcManager{module_bytes: module_bytes};
    let pool = Pool::new(mgr, 10);

    HttpServer::new(move || {
        App::new::data(pool.clone())
            .service(hello)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
