extern crate piston_window;
extern crate ai_behavior;
extern crate sprite;
extern crate find_folder;

mod client;

pub mod game_master {
    tonic::include_proto!("gamemaster");
}

use std::rc::Rc;
use std::collections::HashSet;

use piston_window::*;
use sprite::*;
use ai_behavior::{
    Action,
    Sequence,
    Wait,
    WaitForever,
    While,
};
use uuid::Uuid;
use std::sync::Arc;
use gfx_device_gl::{CommandBuffer, Resources};
use std::path::PathBuf;
use client::GameClient;
use client::game_master::action::Spell;
use tokio::sync::Mutex;
use client::game_master::{LivingBeing};

struct Position { x: u32, y: u32}

impl Position {
    pub fn new(x: u32, y: u32) -> Self {
        Position { x, y }
    }
}

struct Size {w: u32, h: u32}

impl Size {
    pub fn new(w: u32, h: u32) -> Self {
        Size { w, h }
    }
    pub fn apply_scale(&self, scale: f64) -> Size {
        Size {w: (self.w as f64 * scale) as u32, h: (self.h as f64* scale) as u32}
    }
}

struct GLivingBeing {
    name: String,
    position: Position,
    scale: f64,
    actual_size: Size,
    health : u32,
    sprite_id: Uuid
}

struct SpriteDef {
    path: String,
    size: Size
}

impl SpriteDef {
    pub fn new(path: String, size: Size) -> Self {
        SpriteDef { path, size }
    }
}

impl GLivingBeing {
    const GREEN: [f32; 4] =  [0.0, 1.0, 0.0, 1.0];
    const YELLOW: [f32; 4] = [1.0, 1.0, 0.0, 1.0];
    const RED: [f32; 4] = [1.0, 0.0, 0.0, 1.0];
    const BLACK: [f32; 4] =  [0.0, 0.0, 0.0, 1.0];
    
    fn new(    name: String,
               sprite_def: SpriteDef,
               position: Position,
               scale: f64,
               health : u32,
               sprite_id: Uuid) -> GLivingBeing {
        GLivingBeing {name, position, scale, actual_size: sprite_def.size.apply_scale(scale),health, sprite_id}
    }
    
    fn render(&self, scene: &mut Scene<Texture<Resources>>, c: &Context, g: &mut G2d) {
        let sprite = scene.child_mut(self.sprite_id).unwrap();
        sprite.set_position(self.position.x as f64, self.position.y as f64);
        sprite.set_scale(self.scale, self.scale);
        
        let rect = math::margin_rectangle([self.position.x as f64, self.position.y as f64, 100.0, 10.0], 1.0);
        rectangle(GLivingBeing::RED, rect, c.transform, g);
        Rectangle::new_border(GLivingBeing::BLACK, 2.0).draw(rect, &c.draw_state, c.transform, g);
    }
}

struct SpriteLoader {
    texture_context: TextureContext<gfx_device_gl::Factory, Resources, CommandBuffer>,
    assets: PathBuf
}

impl SpriteLoader {
    fn new(window: &mut PistonWindow) -> SpriteLoader{
        let mut texture_context = TextureContext {
            factory: window.factory.clone(),
            encoder: window.factory.create_command_buffer().into()
        };
        let assets = find_folder::Search::ParentsThenKids(3, 3).for_folder("assets").unwrap();
        SpriteLoader{texture_context, assets}
    }
    
    fn load(&mut self, filename: &str) -> Sprite<Texture<Resources>> {
        let tex = Rc::new(Texture::from_path(
            &mut self.texture_context,
            self.assets.join(filename),
            Flip::None,
            &TextureSettings::new()
        ).unwrap());

        Sprite::from_texture(tex)
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    let game_client = Arc::new(GameClient::new().await?);

    let game_client_cloned = Arc::clone(&game_client);
    tokio::spawn(async move {
        game_client_cloned.subscribe_to_game_state_update().await;
    });

    let (width, height) = (1024, 768);
    let opengl = OpenGL::V3_2;
    let mut window: PistonWindow =
        WindowSettings::new("piston: sprite", (width, height))
            .exit_on_esc(true)
            .graphics_api(opengl)
            .build()
            .unwrap();
    let mut scene = Scene::new();
    
    let mut sprite_loader = SpriteLoader::new(&mut window);

   
    // let golem = GLivingBeing::new("Golem du chaos".to_string(),golem_sprite_def, Position::new(400, 500), 0.5, 100, golem_id);
    
    // Rust sprite
    let mut sprite = sprite_loader.load("rust.png");
    sprite.set_position(width as f64 / 2.0, height as f64 / 2.0);
    let id = scene.add_child(sprite);
    
    // Mage sprite
    let mut mage_sprite = sprite_loader.load("mage-idle1.png");
    mage_sprite.set_position(200.0, 200.0);
    let mage_id = scene.add_child(mage_sprite);


    // Run a sequence of animations.
    let seq = Sequence(vec![
        Action(Ease(EaseFunction::CubicOut, Box::new(ScaleTo(2.0, 0.5, 0.5)))),
        Action(Ease(EaseFunction::BounceOut, Box::new(MoveBy(1.0, 0.0, 100.0)))),
        Action(Ease(EaseFunction::ElasticOut, Box::new(MoveBy(2.0, 0.0, -100.0)))),
        Action(Ease(EaseFunction::BackInOut, Box::new(MoveBy(1.0, 0.0, -100.0)))),
        Wait(0.5),
        Action(Ease(EaseFunction::ExponentialInOut, Box::new(MoveBy(1.0, 0.0, 100.0)))),
        Action(Blink(1.0, 5)),
        While(Box::new(WaitForever), vec![
            Action(Ease(EaseFunction::QuadraticIn, Box::new(FadeOut(1.0)))),
            Action(Ease(EaseFunction::QuadraticOut, Box::new(FadeIn(1.0)))),
        ]),
    ]);
    scene.run(id, &seq);

    // This animation and the one above can run in parallel.
    let rotate = Action(Ease(EaseFunction::ExponentialInOut,
                             Box::new(RotateTo(2.0, 360.0))));
    scene.run(id, &rotate);

    //let red = [1.0, 0.0, 0.0, 1.0];
    //Rectangle::new(red).dr

  
    println!("Press any key to pause/resume the animation!");

    let mut living_being_ids = HashSet::new();
    let mut g_living_begins: Vec<GLivingBeing> = Vec::new();
    let mut new_g_living_begins = Vec::new();

    while let Some(e) = window.next() {
        
        
        
        let guard = game_client.get_living_beings().lock().await;


        let filter : Vec<&LivingBeing> = guard.iter().filter(|lb| !living_being_ids.contains(&lb.id)).collect::<Vec<&LivingBeing>>();
        for new_lb in filter {
            let golem_sprite_def = SpriteDef::new("Golem_01_Idle_000.png".to_string(), Size::new(720,480));
            let mut golem_sprite = sprite_loader.load(&golem_sprite_def.path);
            let golem_id = scene.add_child(golem_sprite);
            new_g_living_begins.push(GLivingBeing::new(
                new_lb.name.clone(), 
                golem_sprite_def, 
                Position { x: new_lb.position.as_ref().unwrap().x, y: new_lb.position.as_ref().unwrap().y }, 
                0.25, 
                new_lb.health, 
                golem_id));
            living_being_ids.insert(new_lb.id);
        }
        std::mem::drop(guard);
        
        scene.event(&e);

        window.draw_2d(&e, |c, mut g, _| {
            clear([1.0, 1.0, 1.0, 1.0], g);
            scene.draw(c.transform, g);

            let red = [1.0, 0.0, 0.0, 1.0];
            let black = [0.0, 0.0, 0.0, 1.0];
            let rect = math::margin_rectangle([20.0, 20.0, 100.0, 10.0], 1.0);
            rectangle(red, rect, c.transform, g);
            Rectangle::new_border(black, 2.0).draw(rect, &c.draw_state, c.transform, g);

            for glb in &new_g_living_begins {
                glb.render(&mut scene, &c, &mut g);
            }

            for lb in &g_living_begins {
                let obj = scene.child_mut(lb.sprite_id).unwrap();
                obj.set_position(lb.position.x as f64, lb.position.y as f64);
            }

            g_living_begins.append(&mut new_g_living_begins);
        });

        if let Some(Button::Keyboard(Key::D)) = e.press_args() {
            move_object(&mut scene, mage_id, 10.0, 0.0)
        }
        if let Some(Button::Keyboard(Key::Q)) = e.press_args() {
            move_object(&mut scene, mage_id, -10.0, 0.0)
        }
        if let Some(Button::Keyboard(Key::Z)) = e.press_args() {
            move_object(&mut scene, mage_id, 0.0, -10.0)
        }
        if let Some(Button::Keyboard(Key::S)) = e.press_args() {
            move_object(&mut scene, mage_id, 0.0, 10.0)
        }

        if let Some(Button::Keyboard(Key::D1)) = e.press_args() {
            game_client.send_action(Spell::Fireball).await;
        }
        if let Some(Button::Keyboard(Key::D2)) = e.press_args() {
            game_client.send_action(Spell::FrostBall).await;
        }



        // Some(Input::Character(c)) => {
        //     match c {
        //         '&' | '1' => {
        //             window.addstr("Fireball");
        //            
        //         }
        //         'é' | '2' => {
        //             window.addstr("Frostball");
        //             game_client.send_action(Spell::FrostBall).await;
        //         }
        //         _ => {}
        //     }

        if let Some(_) = e.press_args() {
            scene.toggle(id, &seq);
            scene.toggle(id, &rotate);
        }
    }

    Ok(())
}

fn move_object<T: piston_window::ImageSize>(scene : &mut Scene<T>, object_id: Uuid, delta_x: f64, delta_y: f64) {
    let obj = scene.child_mut(object_id).unwrap();
    obj.set_position(obj.get_position().0 + delta_x, obj.get_position().1 + delta_y);
}