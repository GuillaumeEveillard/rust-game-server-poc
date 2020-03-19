use std::sync::{Arc};
use tokio::sync::Mutex;
use tokio::sync::mpsc;
use tonic::{Request, Response, Status, transport::Server};

use game_master::{Action, ActionResult, GameStateRequest, GameStateResponse, LivingBeing};
use game_master::game_master_server::{GameMaster, GameMasterServer};
use std::time::{Instant, Duration};

pub mod game_master {
    tonic::include_proto!("gamemaster");
}

struct StateManager {
    counter: u32,
    living_beings: Vec<LivingBeing>
}

impl StateManager {
    fn new() -> StateManager {
        let mut living_beings = Vec::new();
        living_beings.push(LivingBeing{id: 1, name: "Murloc du chaos".to_string(), health: 100});
        StateManager{counter: 0, living_beings}
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
        
        let mut state = self.state_manager.lock().await;
        // let mut state = lock.unwrap();
        state.inc();
        let c = state.counter;

        let reply = game_master::ActionResult {
            message: format!("Hello {}! Counter={}", request.into_inner().spell, c),
        };
        Ok(Response::new(reply))
    }

    type GameStateStreamingStream = mpsc::Receiver<Result<GameStateResponse, Status>>;

    async fn game_state_streaming(&self, _: Request<GameStateRequest>, ) -> Result<Response<Self::GameStateStreamingStream>, Status> {
        let (mut tx, rx) = mpsc::channel::<Result<GameStateResponse, Status>>(10);


        let sm = self.state_manager.clone();
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval_at(tokio::time::Instant::now(), Duration::from_secs(1));

            let start : u64 = 0;
            for i in start..10 {
                interval.tick().await;

                let state = sm.lock().await;
                let to_send = GameStateResponse{counter: i, living_beings: state.living_beings.clone()};
                let result: Result<GameStateResponse, Status> = Ok(to_send);
                tx.send(result).await.unwrap();
                
                tokio::task::yield_now().await;
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
