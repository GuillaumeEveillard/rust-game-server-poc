extern crate pancurses;

use pancurses::{initscr, endwin, Input, noecho};

use hello_world::greeter_client::GreeterClient;
use hello_world::HelloRequest;
use tonic::transport::{Endpoint, Channel};

pub mod hello_world {
    tonic::include_proto!("helloworld");
}

pub mod echo {
    tonic::include_proto!("grpc.examples.echo");
}
use echo::{echo_client::EchoClient, EchoRequest};
use std::error::Error;
use tonic::Status;

struct GameClient {
    greeter_client : GreeterClient<Channel>,
    echo_client :EchoClient<Channel>
}

impl GameClient {
    async fn new() -> Result<GameClient, Box<dyn Error>> {
        let channel = Endpoint::from_static("http://[::1]:50051")
            .connect()
            .await?;

        let mut greeter_client = GreeterClient::new(channel.clone());
        let mut echo_client = EchoClient::new(channel);
        Ok(GameClient{greeter_client, echo_client})
    }

    async fn say_hello(&mut self) {
        let request = tonic::Request::new(HelloRequest {
            name: "Tonic".into(),
        });

        let response = self.greeter_client.say_hello(request).await.unwrap();

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
