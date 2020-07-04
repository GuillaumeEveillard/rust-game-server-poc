extern crate pancurses;

use std::error::Error;

use pancurses::{endwin, initscr, Input, noecho};
use tonic::transport::{Channel, Endpoint};

use game_master::Action;
use game_master::game_master_client::GameMasterClient;
use game_master::{LivingBeing};
use game_master::GameStateRequest;
use game_master::action::Spell;
use tokio::sync::Mutex;
use std::sync::Arc;

pub mod game_master {
    tonic::include_proto!("gamemaster");
}

pub struct GameClient {
    game_master_client: Mutex<GameMasterClient<Channel>>,
    living_beings: Mutex<Vec<LivingBeing>>
}

impl GameClient {
    pub async fn new() -> Result<GameClient, Box<dyn Error>> {
        let channel = Endpoint::from_static("http://[::1]:50051")
            .connect()
            .await?;

        let greeter_client = Mutex::new(GameMasterClient::new(channel.clone()));
        Ok(GameClient{ game_master_client: greeter_client, living_beings: Mutex::new(Vec::new())})
    }

    pub async fn send_action(&self, spell: Spell) {
        let request = tonic::Request::new(Action {
            spell: spell as i32,
        });

        let mut guard = self.game_master_client.lock().await;
        let response = guard.send_action(request).await.unwrap();
        std::mem::drop(guard);

        println!("RESPONSE={:?}", response);
    }

    pub async fn subscribe_to_game_state_update(&self) {
        let request = GameStateRequest{message: "Hello echo".to_string()};
        
        let mut guard = self.game_master_client.lock().await;
        let response = guard.game_state_streaming(request).await.unwrap();
        std::mem::drop(guard);

        let mut inbound = response.into_inner();

        while let Some(ref mut note) = inbound.message().await.unwrap() {
            println!("counter = {:?}", note.counter);
            println!("NOTE = {:?}", note);
            let mut guard = self.living_beings.lock().await;
            guard.clear();
            guard.append(&mut note.living_beings);
            std::mem::drop(guard);
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut game_client = GameClient::new().await?;

    game_client.subscribe_to_game_state_update().await;    
    let window = initscr();
    window.printw("Type things, press delete to quit\n");
    window.refresh();
    window.keypad(true);
    noecho();
    loop {
        match window.getch() {
            Some(Input::Character(c)) => { 
                match c {
                    '&' | '1' => {
                        window.addstr("Fireball");
                        game_client.send_action(Spell::Fireball).await;
                    }
                    'é' | '2' => {
                        window.addstr("Frostball");
                        game_client.send_action(Spell::FrostBall).await;
                    }
                    _ => {}
                }
            },
            Some(Input::KeyDC) => break,
            Some(input) => { window.addstr(&format!("{:?}", input)); },
            None => ()
        }
    }
    endwin();


    game_client.subscribe_to_game_state_update().await;
    
    Ok(())
}
