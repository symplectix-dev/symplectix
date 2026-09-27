//! Serves `shed.grpc_example.Echo` on the address given as the first
//! argument, `127.0.0.1:50051` by default.

use std::env;

use tonic::transport::server::TcpIncoming;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let addr = env::args().nth(1).unwrap_or("127.0.0.1:50051".to_owned());
    let incoming = TcpIncoming::bind(addr.parse()?)?;
    println!("serving shed.grpc_example.Echo on {}", incoming.local_addr()?);
    echo::serve(incoming).await?;
    Ok(())
}
