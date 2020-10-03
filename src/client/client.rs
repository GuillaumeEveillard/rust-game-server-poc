extern crate pancurses;

use std::error::Error;

use pancurses::{endwin, initscr, noecho, Input};
use tonic::transport::{Channel, Endpoint};

use game_master::action::Spell;
use game_master::game_master_client::GameMasterClient;
use game_master::Action;
use game_master::GameStateRequest;
use game_master::LivingBeing;
use game_master::NewPlayerRequest;
use tokio::sync::Mutex;

pub mod game_master {
    tonic::include_proto!("gamemaster");
}

pub struct GameClient {
    game_master_client: Mutex<GameMasterClient<Channel>>,
    living_beings: Mutex<Vec<LivingBeing>>,
    pub player_id: u64,
}

impl GameClient {
    pub async fn new(player_name: &str) -> Result<GameClient, Box<dyn Error>> {
        let channel = Endpoint::from_static("http://[::1]:50051").connect().await?;

        let mut game_master_client = GameMasterClient::new(channel.clone());

        let player_id = game_master_client
            .new_player(NewPlayerRequest {
                player_name: player_name.to_string(),
            })
            .await
            .unwrap()
            .get_ref()
            .id;

        Ok(GameClient {
            game_master_client: Mutex::new(game_master_client),
            living_beings: Mutex::new(Vec::new()),
            player_id: player_id,
        })
    }

    pub async fn send_action(&self, spell: Spell) {
        let request = tonic::Request::new(Action { spell: spell as i32 });

        let mut guard = self.game_master_client.lock().await;
        let response = guard.send_action(request).await.unwrap();
        std::mem::drop(guard);

        println!("RESPONSE={:?}", response);
    }

    pub async fn subscribe_to_game_state_update(&self) {
        let request = GameStateRequest {
            message: "Hello echo".to_string(),
        };

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

    pub fn get_living_beings(&self) -> &Mutex<Vec<LivingBeing>> {
        &self.living_beings
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let game_client = GameClient::new("GGYE").await?;

    game_client.subscribe_to_game_state_update().await;
    let window = initscr();
    window.printw("Type things, press delete to quit\n");
    window.refresh();
    window.keypad(true);
    noecho();
    loop {
        match window.getch() {
            Some(Input::Character(c)) => match c {
                '&' | '1' => {
                    window.addstr("Fireball");
                    game_client.send_action(Spell::Fireball).await;
                }
                'é' | '2' => {
                    window.addstr("Frostball");
                    game_client.send_action(Spell::FrostBall).await;
                }
                _ => {}
            },
            Some(Input::KeyDC) => break,
            Some(input) => {
                window.addstr(&format!("{:?}", input));
            }
            None => (),
        }
    }
    endwin();

    game_client.subscribe_to_game_state_update().await;

    Ok(())
}
