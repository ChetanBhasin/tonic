pub mod pb {
    tonic::include_proto!("grpc.examples.echo");
}

use pb::{EchoRequest, EchoResponse};
use std::{collections::VecDeque, net::SocketAddr};
use tokio::sync::mpsc;
use tonic::{transport::Server, Request, Response, Status, Streaming};

type EchoResult<T> = Result<Response<T>, Status>;
type Stream = VecDeque<Result<EchoResponse, Status>>;

#[derive(Debug)]
pub struct EchoServer {
    addr: SocketAddr,
}

#[tonic::async_trait]
impl pb::server::Echo for EchoServer {
    async fn unary_echo(&self, request: Request<EchoRequest>) -> EchoResult<EchoResponse> {
        let message = format!("{} (from {})", request.into_inner().message, self.addr);

        Ok(Response::new(EchoResponse { message }))
    }

    type ServerStreamingEchoStream = Stream;

    async fn server_streaming_echo(
        &self,
        _: Request<EchoRequest>,
    ) -> EchoResult<Self::ServerStreamingEchoStream> {
        Err(Status::unimplemented("not implemented"))
    }

    async fn client_streaming_echo(
        &self,
        _: Request<Streaming<EchoRequest>>,
    ) -> EchoResult<EchoResponse> {
        Err(Status::unimplemented("not implemented"))
    }

    type BidirectionalStreamingEchoStream = Stream;

    async fn bidirectional_streaming_echo(
        &self,
        _: Request<Streaming<EchoRequest>>,
    ) -> EchoResult<Self::BidirectionalStreamingEchoStream> {
        Err(Status::unimplemented("not implemented"))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addrs = ["[::1]:50051", "[::1]:50052"];

    let (tx, mut rx) = mpsc::unbounded_channel();

    for addr in &addrs {
        let addr = addr.parse()?;
        let mut tx = tx.clone();

        let server = EchoServer { addr };
        let serve = Server::builder().serve(addr, pb::server::EchoServer::new(server));

        tokio::spawn(async move {
            if let Err(e) = serve.await {
                eprintln!("Error = {:?}", e);
            }

            tx.try_send(()).unwrap();
        });
    }

    rx.recv().await;

    Ok(())
}
