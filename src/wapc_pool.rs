use async_trait::async_trait;

use wapc::WapcHost;
use wasmtime_provider::WasmtimeEngineProvider;

pub(crate) struct WapcManager {
  pub module_bytes: Vec<u8>,
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

#[async_trait]
impl deadpool::managed::Manager<WapcHost, Box<dyn std::error::Error>> for WapcManager {
    async fn create(&self) -> Result<WapcHost, Box<dyn std::error::Error>> {
        let engine = WasmtimeEngineProvider::new(&self.module_bytes, None);
        Ok(WapcHost::new(Box::new(engine), host_callback)?)
    }
    async fn recycle(&self, conn: &mut WapcHost) -> deadpool::managed::RecycleResult<Box<dyn std::error::Error>> {
        Ok(())
    }
}