extern crate pancurses;

use std::error::Error;

use pancurses::{endwin, initscr, Input, noecho};
use tonic::transport::{Channel, Endpoint};

use game_master::Action;
use game_master::game_master_client::GameMasterClient;
use game_master::GameStateRequest;
use game_master::action::Spell;

pub mod game_master {
    tonic::include_proto!("gamemaster");
}

struct GameClient {
    game_master_client: GameMasterClient<Channel>,
}

impl GameClient {
    async fn new() -> Result<GameClient, Box<dyn Error>> {
        let channel = Endpoint::from_static("http://[::1]:50051")
            .connect()
            .await?;

        let greeter_client = GameMasterClient::new(channel.clone());
        Ok(GameClient{ game_master_client: greeter_client})
    }

    async fn send_action(&mut self, spell: Spell) {
        let request = tonic::Request::new(Action {
            spell: spell as i32,
        });

        let response = self.game_master_client.send_action(request).await.unwrap();

        println!("RESPONSE={:?}", response);
    }

    async fn subscribe_to_game_state_update(&mut self) {
        let request = GameStateRequest{message: "Hello echo".to_string()};
        let response = self.game_master_client.game_state_streaming(request).await.unwrap();

        let mut inbound = response.into_inner();

        while let Some(note) = inbound.message().await.unwrap() {
            println!("NOTE = {:?}", note);
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
