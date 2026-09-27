//! Calls `Echo` on the endpoint given as the first argument, sending the
//! message given as the second.

use std::env;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut args = env::args().skip(1);
    let target = args.next().unwrap_or("http://127.0.0.1:50051".to_owned());
    let message = args.next().unwrap_or("hello".to_owned());

    println!("{}", echo::call(&target, &message).await?);
    Ok(())
}
