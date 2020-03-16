use tonic::{transport::Server, Request, Response, Status};

use hello_world::greeter_server::{Greeter, GreeterServer};
use hello_world::{HelloReply, HelloRequest};
use std::sync::mpsc::{channel, Sender};
use std::sync::{Mutex, Arc};

pub mod hello_world {
    tonic::include_proto!("helloworld");
}

struct StateManager {
    tx: Sender<u32>,
    counter: u32
}

impl StateManager {
    
    fn new() -> StateManager {
        let (tx, rx) = channel::<u32>();
        StateManager{tx, counter: 0}
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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse().unwrap();
    let state_manager = StateManager::new();
    let state_handle = Arc::new(Mutex::new(state_manager));
    let greeter = MyGreeter::new(state_handle);

    println!("GreeterServer listening on {}", addr);

    Server::builder()
        .add_service(GreeterServer::new(greeter))
        .serve(addr)
        .await?;

    Ok(())
}
