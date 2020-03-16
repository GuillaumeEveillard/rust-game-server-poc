extern crate pancurses;

use pancurses::{initscr, endwin, Input, noecho};

use game_master::game_master_client::GameMasterClient;
use game_master::Action;
use tonic::transport::{Endpoint, Channel};

pub mod game_master {
    tonic::include_proto!("gamemaster");
}

pub mod echo {
    tonic::include_proto!("grpc.examples.echo");
}
use echo::{echo_client::EchoClient, EchoRequest};
use std::error::Error;

struct GameClient {
    game_master_client: GameMasterClient<Channel>,
    echo_client :EchoClient<Channel>
}

impl GameClient {
    async fn new() -> Result<GameClient, Box<dyn Error>> {
        let channel = Endpoint::from_static("http://[::1]:50051")
            .connect()
            .await?;

        let greeter_client = GameMasterClient::new(channel.clone());
        let echo_client = EchoClient::new(channel);
        Ok(GameClient{ game_master_client: greeter_client, echo_client})
    }

    async fn say_hello(&mut self) {
        let request = tonic::Request::new(Action {
            name: "Tonic".into(),
        });

        let response = self.game_master_client.send_action(request).await.unwrap();

        println!("RESPONSE={:?}", response);
    }

    async fn listen_to_echo(&mut self) {
        let request = EchoRequest{message: "Hello echo".to_string()};
        let response = self.echo_client.server_streaming_echo(request).await.unwrap();

        let mut inbound = response.into_inner();

        while let Some(note) = inbound.message().await.unwrap() {
            println!("NOTE = {:?}", note);
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut game_client = GameClient::new().await?;

    println!("heu");
    
    let window = initscr();
    window.printw("Type things, press delete to quit\n");
    window.refresh();
    window.keypad(true);
    noecho();
    loop {
        match window.getch() {
            Some(Input::Character(c)) => { 
                match c {
                    '&' | '1' => {window.addstr("Fireball");}
                    'é' | '2' => {window.addstr("Frostball");}
                    _ => {}
                }
            },
            Some(Input::KeyDC) => break,
            Some(input) => { window.addstr(&format!("{:?}", input)); },
            None => ()
        }
    }
    endwin();

    game_client.say_hello().await;
    println!("heu");
    game_client.listen_to_echo().await;
    println!("heu");
    
    Ok(())
}
