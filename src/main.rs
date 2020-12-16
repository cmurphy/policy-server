use std::{convert::Infallible, net::SocketAddr};
use hyper::{Body, Request, Response, Server};
use hyper::service::{make_service_fn, service_fn};

use tokio::sync::mpsc;

use std::fs::File;
use std::io::prelude::*;

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


fn init_host() -> WapcHost {
    let module_bytes = load_file("test.wasm");
    let engine = WasmtimeEngineProvider::new(&module_bytes, None);
    WapcHost::new(Box::new(engine), host_callback).expect("Failed to init waPC host")
}

async fn handle(req: Request<Body>, mut tx: mpsc::Sender<String>) -> Result<Response<Body>, Infallible> {
    tx.send(String::from("ciao")).await.unwrap();
    Ok(Response::new("Hello, World!".into()))
}

#[tokio::main]
async fn main() {
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    //let wapc_host = init_host();

    let (tx, mut rx) = mpsc::channel(32);

    let wasm_task = tokio::spawn(async move {
        while let Some(cmd) = rx.recv().await {
            println!("I received {}", cmd);
            return;
        }
    });

    let make_svc = make_service_fn(|_conn| {
        let svc_tx = tx.clone();
        async move {
            Ok::<_, Infallible>(service_fn(move |req| {
                handle(req, svc_tx.clone())
            }))
        }
    });

    let server = Server::bind(&addr).serve(make_svc);

    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
    wasm_task.await.unwrap();
}