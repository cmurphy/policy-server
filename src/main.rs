use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};

use std::fs::File;
use std::io::prelude::*;

use deadpool::unmanaged::Pool;
use wapc::WapcHost;
use wasmtime_provider::WasmtimeEngineProvider;

fn load_file(path: &str) -> Vec<u8> {
    println!("{}", path);
    let mut f = File::open(path).unwrap();
    let mut buf = Vec::new();
    f.read_to_end(&mut buf).unwrap();
    buf
}

fn host_callback(
    id: u64,
    bd: &str,
    ns: &str,
    op: &str,
    payload: &[u8],
) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
    println!(
        "Guest {} invoked '{}->{}:{}' with payload of {}",
        id,
        bd,
        ns,
        op,
        ::std::str::from_utf8(payload).unwrap()
    );
    Ok(b"Host result".to_vec())
}


#[get("/")]
async fn hello(pool: web::Data<Pool<wapc::WapcHost>>) -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let module_bytes = load_file("test.wasm");

    let pool = Pool::<wapc::WapcHost>::new(10);
    for _ in 0..10 {
        let engine = WasmtimeEngineProvider::new(&module_bytes, None);
        let host = WapcHost::new(Box::new(engine), host_callback).expect("Failed to init waPC host");

        pool.add(host);
    }

    HttpServer::new(move || {
        App::new()
            .data(pool)
            .service(hello)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
