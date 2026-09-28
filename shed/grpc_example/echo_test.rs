//! Round trip through the generated client and server.

use tonic::transport::server::TcpIncoming;

#[tokio::test]
async fn echo_round_trip() {
    // Port 0 lets the kernel pick a free port, so parallel tests cannot
    // collide. `serve` takes an already-bound listener, so the client
    // cannot be refused: the connection waits in the accept queue until
    // the spawned task gets to it.
    let incoming = TcpIncoming::bind("127.0.0.1:0".parse().unwrap()).unwrap();
    let addr = incoming.local_addr().unwrap();
    let server = tokio::spawn(echo::serve(incoming));

    let target = format!("http://{addr}");
    let response = echo::call(&target, "hello").await.unwrap();
    assert_eq!(response, "hello");

    server.abort();
}
