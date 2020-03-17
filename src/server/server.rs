use std::sync::{Arc, Mutex};

use tokio::sync::mpsc;
use tonic::{Request, Response, Status, transport::Server};

use game_master::{Action, ActionResult, GameStateRequest, GameStateResponse};
use game_master::game_master_server::{GameMaster, GameMasterServer};

pub mod game_master {
    tonic::include_proto!("gamemaster");
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

pub struct GameServer {
    state_manager: Arc<Mutex<StateManager>>
}

impl GameServer {
    fn new(state_manager: Arc<Mutex<StateManager>>) -> GameServer {
        GameServer {state_manager}
    }
}

#[tonic::async_trait]
impl GameMaster for GameServer {
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

    type GameStateStreamingStream = mpsc::Receiver<Result<GameStateResponse, Status>>;

    async fn game_state_streaming(&self, _: Request<GameStateRequest>, ) -> Result<Response<Self::GameStateStreamingStream>, Status> {
        let (mut tx, rx) = mpsc::channel::<Result<GameStateResponse, Status>>(10);

        tokio::spawn(async move {
            let start : i32 = 0;
            for i in start..10 {
                let to_send = GameStateResponse{message: format!("Echo {}", i)};
                let result: Result<GameStateResponse, Status> = Ok(to_send);
                tx.send(result).await.unwrap();
            }

            println!(" /// done sending");
        });

        Ok(Response::new(rx))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse().unwrap();
    let state_manager = StateManager::new();
    let state_handle = Arc::new(Mutex::new(state_manager));
    let greeter = GameServer::new(state_handle);

    println!("GreeterServer listening on {}", addr);

    Server::builder()
        .add_service(GameMasterServer::new(greeter))
        .serve(addr)
        .await?;

    Ok(())
}
