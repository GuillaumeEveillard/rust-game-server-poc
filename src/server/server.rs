use tonic::{transport::Server, Request, Response, Status};

use game_master::game_master_server::{GameMaster, GameMasterServer};
use game_master::{Action, ActionResult};
use std::sync::{Mutex, Arc};
use echo::{
    echo_server::{Echo, EchoServer},
    EchoRequest, EchoResponse,
};

use tokio::sync::mpsc;

pub mod game_master {
    tonic::include_proto!("gamemaster");
}

pub mod echo {
    tonic::include_proto!("grpc.examples.echo");
}

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
impl GameMaster for MyGreeter {
    async fn send_action(&self, request: Request<Action>, ) -> Result<Response<ActionResult>, Status> {
        println!("Got a request from {:?}", request.remote_addr());
        
        let lock = self.state_manager.lock();
        let mut state = lock.unwrap();
        state.inc();
        let c = state.counter;

        let reply = game_master::ActionResult {
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
        .add_service(GameMasterServer::new(greeter))
        .add_service(echo)
        .serve(addr)
        .await?;

    Ok(())
}
