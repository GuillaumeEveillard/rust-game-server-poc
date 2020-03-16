use tonic::{transport::Server, Request, Response, Status};

use hello_world::greeter_server::{Greeter, GreeterServer};
use hello_world::{HelloReply, HelloRequest};
use std::sync::{Mutex, Arc};
use echo::{
    echo_server::{Echo, EchoServer},
    EchoRequest, EchoResponse,
};
use tonic::codegen::Stream;
use std::pin::Pin;
use tokio::sync::mpsc;

pub mod hello_world {
    tonic::include_proto!("helloworld");
}

pub mod echo {
    tonic::include_proto!("grpc.examples.echo");
}

type ResponseStream = Pin<Box<dyn Stream<Item = Result<EchoResponse, Status>> + Send + Sync>>;

struct StateManager {
    counter: u32
}

impl StateManager {
    
    fn new() -> StateManager {
        StateManager{counter: 0}
    }
    
    fn inc(&mut self) {
        self.counter = self.counter + 1;
    }
}

pub struct MyGreeter {
    state_manager: Arc<Mutex<StateManager>>
}

impl MyGreeter {
    fn new(state_manager: Arc<Mutex<StateManager>>) -> MyGreeter {
        MyGreeter{state_manager}
    }
}

#[tonic::async_trait]
impl Greeter for MyGreeter {
    async fn say_hello(&self, request: Request<HelloRequest>, ) -> Result<Response<HelloReply>, Status> {
        println!("Got a request from {:?}", request.remote_addr());
        
        let lock = self.state_manager.lock();
        let mut state = lock.unwrap();
        state.inc();
        let c = state.counter;

        let reply = hello_world::HelloReply {
            message: format!("Hello {}! Counter={}", request.into_inner().name, c),
        };
        Ok(Response::new(reply))
    }
}

#[derive(Default)]
pub struct MyEcho;

#[tonic::async_trait]
impl Echo for MyEcho {

    type ServerStreamingEchoStream = mpsc::Receiver<Result<EchoResponse, Status>>;

    async fn server_streaming_echo(
        &self,
        _: Request<EchoRequest>,
    ) -> Result<Response<Self::ServerStreamingEchoStream>, Status> {
        let (mut tx, rx) = mpsc::channel::<Result<EchoResponse, Status>>(10);

        tokio::spawn(async move {
            let start : i32 = 0;
            for i in start..10 {
                let to_send = EchoResponse{message: format!("Echo {}", i)};
                let result: Result<EchoResponse, Status> = Ok(to_send);
                tx.send(result).await.unwrap();
            }

            println!(" /// done sending");
        });

        Ok(Response::new(rx))
    }

    // type BidirectionalStreamingEchoStream = ResponseStream;
    
    // async fn bidirectional_streaming_echo(&self, _: Request<tonic::Streaming<EchoRequest>>, ) -> Result<Response<Self::BidirectionalStreamingEchoStream>, Status> {
    //     let r: Result<Response<Self::BidirectionalStreamingEchoStream>, Status> = Result::Err(Status::aborted("paf"));
    //     r
    // }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse().unwrap();
    let state_manager = StateManager::new();
    let state_handle = Arc::new(Mutex::new(state_manager));
    let greeter = MyGreeter::new(state_handle);

    let echo = EchoServer::new(MyEcho::default());

    println!("GreeterServer listening on {}", addr);

    Server::builder()
        .add_service(GreeterServer::new(greeter))
        .add_service(echo)
        .serve(addr)
        .await?;

    Ok(())
}
