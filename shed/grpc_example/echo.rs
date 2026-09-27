//! The two halves of the example: a server built from the generated
//! service, and a client call made through a tonic channel.

use echo_proto::shed::grpc_example as pb;
use tonic::transport::Server;
use tonic::transport::server::TcpIncoming;

/// Answers `Echo` with the message it was sent.
#[derive(Debug, Default)]
pub struct EchoService;

#[tonic::async_trait]
impl pb::echo_server::Echo for EchoService {
    async fn echo(
        &self,
        request: tonic::Request<pb::EchoRequest>,
    ) -> Result<tonic::Response<pb::EchoResponse>, tonic::Status> {
        let message = request.into_inner().message;
        Ok(tonic::Response::new(pb::EchoResponse { message }))
    }
}

/// Serves the Echo service until `incoming` stops yielding connections.
pub async fn serve(incoming: TcpIncoming) -> anyhow::Result<()> {
    Server::builder()
        .add_service(pb::echo_server::EchoServer::new(EchoService))
        .serve_with_incoming(incoming)
        .await?;
    Ok(())
}

/// Calls `Echo` on `target`, an endpoint such as
/// `http://127.0.0.1:50051`.
pub async fn call(target: &str, message: &str) -> anyhow::Result<String> {
    let mut client = pb::echo_client::EchoClient::connect(target.to_owned()).await?;
    let request = pb::EchoRequest { message: message.to_owned() };
    let response = client.echo(request).await?;
    Ok(response.into_inner().message)
}
