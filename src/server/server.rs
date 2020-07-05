use std::sync::{Arc};
use tokio::sync::Mutex;
use tokio::sync::mpsc;
use tonic::{Request, Response, Status, transport::Server};

use game_master::{Action, ActionResult, GameStateRequest, GameStateResponse, LivingBeing, Position};
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
        living_beings.push(LivingBeing{id: 1, name: "Murloc du chaos".to_string(), health: 100, position: Some(Position{x: 50, y: 50})});
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

impl LivingBeing {
    fn update_health(&mut self, diff: i32) {
        self.health = std::cmp::max(0, self.health as i32 + diff) as u32; //il y a un risque d'overflow ici  
    }
    
    fn move_it(&mut self, x_diff: i32, y_diff: i32) {
        let old_position = self.position.as_ref().unwrap();
        self.position = Some(Position{x: (old_position.x as i32 + x_diff) as u32,y: (old_position.y as i32 + y_diff) as u32});
    }
}

#[tonic::async_trait]
impl GameMaster for GameServer {
    async fn send_action(&self, request: Request<Action>, ) -> Result<Response<ActionResult>, Status> {
        println!("Got a request from {:?}", request.remote_addr());
        
        let action = request.get_ref();
        let spell = action.spell;

        let mut state = self.state_manager.lock().await;
        
        match spell {
            Fireball => { 
                println!("Fireballlllllllll !");
                let mut mob = state.living_beings.get_mut(0).unwrap();
                mob.update_health(-30);
            }
            FrostBall => {
                println!("Frostball");
                let mut mob = state.living_beings.get_mut(0).unwrap();
                mob.update_health(-20);
            }
        }
      
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
            let mut interval = tokio::time::interval_at(tokio::time::Instant::now(), Duration::from_secs(5));

            let start : u64 = 0;
            for i in start..20 {
                interval.tick().await;

                let mut state = sm.lock().await;
                for lb in &mut state.living_beings {
                    lb.move_it(10, 10);
                }
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
