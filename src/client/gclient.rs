extern crate ai_behavior;
extern crate find_folder;
extern crate piston_window;
extern crate sprite;

use std::collections::HashMap;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::Arc;

use ai_behavior::{Action, Behavior, Sequence, Wait, WaitForever, While};
use gfx_device_gl::{CommandBuffer, Device, Resources};
use piston_window::*;
use sprite::*;
use uuid::Uuid;

use client::game_master::action::Spell;
use client::game_master::living_being::Class;
use client::GameClient;
use gfx::texture::ResourceDesc;
use std::collections::hash_map::Entry;

mod client;

pub mod game_master {
    tonic::include_proto!("gamemaster");
}

struct Position {
    x: u32,
    y: u32,
}

impl Position {
    pub fn new(x: u32, y: u32) -> Self {
        Position { x, y }
    }
}

struct Size {
    w: u32,
    h: u32,
}

impl Size {
    pub fn new(w: u32, h: u32) -> Self {
        Size { w, h }
    }
    pub fn apply_scale(&self, scale: f64) -> Size {
        Size {
            w: (self.w as f64 * scale) as u32,
            h: (self.h as f64 * scale) as u32,
        }
    }
}

struct GLivingBeing {
    name: String,
    position: Position,
    scale: f64,
    actual_size: Size,
    health: u32,
    sprite_id: Uuid,
    has_been_rendered: bool,
}

struct SpriteDef {
    path: String,
    size: Size,
}

impl SpriteDef {
    pub fn new(path: String, size: Size) -> Self {
        SpriteDef { path, size }
    }
}

impl GLivingBeing {
    const GREEN: [f32; 4] = [0.0, 1.0, 0.0, 1.0];
    const YELLOW: [f32; 4] = [1.0, 1.0, 0.0, 1.0];
    const RED: [f32; 4] = [1.0, 0.0, 0.0, 1.0];
    const BLACK: [f32; 4] = [0.0, 0.0, 0.0, 1.0];

    fn new(
        name: String,
        sprite_def: SpriteDef,
        position: Position,
        scale: f64,
        health: u32,
        sprite_id: Uuid,
        has_been_rendered: bool,
    ) -> GLivingBeing {
        GLivingBeing {
            name,
            position,
            scale,
            actual_size: sprite_def.size.apply_scale(scale),
            health,
            sprite_id,
            has_been_rendered,
        }
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
    assets: PathBuf,
}

impl SpriteLoader {
    fn new(window: &mut PistonWindow) -> SpriteLoader {
        let texture_context = TextureContext {
            factory: window.factory.clone(),
            encoder: window.factory.create_command_buffer().into(),
        };
        let assets = find_folder::Search::ParentsThenKids(3, 3).for_folder("assets").unwrap();
        SpriteLoader {
            texture_context,
            assets,
        }
    }

    fn load(&mut self, filename: &str) -> Sprite<Texture<Resources>> {
        let tex = Rc::new(
            Texture::from_path(
                &mut self.texture_context,
                self.assets.join(filename),
                Flip::None,
                &TextureSettings::new(),
            )
            .unwrap(),
        );

        Sprite::from_texture(tex)
    }
}

struct Game {
    game_client: Arc<GameClient>,
    window: PistonWindow,
    scene: Scene<Texture<Resources>>,
    sprite_loader: SpriteLoader,
    g_living_begins: HashMap<u64, GLivingBeing>,
    player_sprint_id: Option<Uuid>,
}

impl Game {
    async fn new() -> Game {
        let game_client = Arc::new(GameClient::new("GGYE").await.unwrap());

        let game_client_cloned = Arc::clone(&game_client);
        tokio::spawn(async move {
            game_client_cloned.subscribe_to_game_state_update().await;
        });

        let (width, height) = (1024, 768);
        let opengl = OpenGL::V3_2;
        let mut window: PistonWindow = WindowSettings::new("piston: sprite", (width, height))
            .exit_on_esc(true)
            .graphics_api(opengl)
            .build()
            .unwrap();

        let sprite_loader = SpriteLoader::new(&mut window);

        Game {
            game_client,
            window,
            scene: Scene::new(),
            sprite_loader,
            g_living_begins: HashMap::new(),
            player_sprint_id: Option::None,
        }
    }

    async fn game_loop(&mut self) {
        while let Some(e) = self.window.next() {
            self.process_state().await;
            self.draw(&e);
            self.handle_input(e).await
        }
    }

    async fn process_state(&mut self) {
        let guard = self.game_client.get_living_beings().lock().await;
        for new_lb in guard.iter() {
            match self.g_living_begins.entry(new_lb.id) {
                Entry::Occupied(mut e) => {
                    println!("Updating {} ", new_lb.id);
                    e.get_mut().position = Position {
                        x: new_lb.position.as_ref().unwrap().x,
                        y: new_lb.position.as_ref().unwrap().y,
                    };
                }
                Entry::Vacant(e) => {
                    println!("Creating {} ", new_lb.id);
                    let sprite_def = if new_lb.class == Class::Golem as i32 {
                        SpriteDef::new("Golem_01_Idle_000.png".to_string(), Size::new(720, 480))
                    } else {
                        SpriteDef::new("mage-idle1.png".to_string(), Size::new(128, 128))
                    };
                    let scale = if new_lb.class == Class::Golem as i32 { 0.25 } else { 1.0 };
                    let mut sprite = self.sprite_loader.load(&sprite_def.path);
                    let sprite_id = self.scene.add_child(sprite);
                    if (self.game_client.player_id == new_lb.id) {
                        self.player_sprint_id = Option::Some(sprite_id)
                    }
                    let glb = GLivingBeing::new(
                        new_lb.name.clone(),
                        sprite_def,
                        Position {
                            x: new_lb.position.as_ref().unwrap().x,
                            y: new_lb.position.as_ref().unwrap().y,
                        },
                        scale,
                        new_lb.health,
                        sprite_id,
                        false,
                    );
                    e.insert(glb);
                }
            }
        }

        std::mem::drop(guard);
    }

    fn draw(&mut self, e: &Event) {
        self.scene.event(e);
        let g_living_begins = &self.g_living_begins;
        let mut scene = &mut self.scene;

        let x = |c: Context, g: &mut G2d<'_>, _: &mut Device| {
            clear([1.0, 1.0, 1.0, 1.0], g);
            scene.draw(c.transform, g);

            let red = [1.0, 0.0, 0.0, 1.0];
            let black = [0.0, 0.0, 0.0, 1.0];
            let rect = math::margin_rectangle([20.0, 20.0, 100.0, 10.0], 1.0);
            rectangle(red, rect, c.transform, g);
            Rectangle::new_border(black, 2.0).draw(rect, &c.draw_state, c.transform, g);

            for (_, glb) in g_living_begins {
                if glb.has_been_rendered {
                    let obj = scene.child_mut(glb.sprite_id).unwrap();
                    obj.set_position(glb.position.x as f64, glb.position.y as f64);
                } else {
                    glb.render(&mut scene, &c, g);
                }
            }
        };
        let window = &mut self.window;
        window.draw_2d(e, x);
    }

    async fn handle_input(&mut self, e: Event) {
        if let Some(Button::Keyboard(Key::D)) = e.press_args() {
            move_object(&mut self.scene, self.player_sprint_id.unwrap(), 10.0, 0.0)
        }
        if let Some(Button::Keyboard(Key::Q)) = e.press_args() {
            move_object(&mut self.scene, self.player_sprint_id.unwrap(), -10.0, 0.0)
        }
        if let Some(Button::Keyboard(Key::Z)) = e.press_args() {
            move_object(&mut self.scene, self.player_sprint_id.unwrap(), 0.0, -10.0)
        }
        if let Some(Button::Keyboard(Key::S)) = e.press_args() {
            move_object(&mut self.scene, self.player_sprint_id.unwrap(), 0.0, 10.0)
        }

        if let Some(Button::Keyboard(Key::D1)) = e.press_args() {
            self.game_client.send_action(Spell::Fireball).await;
        }
        if let Some(Button::Keyboard(Key::D2)) = e.press_args() {
            self.game_client.send_action(Spell::FrostBall).await;
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut game = Game::new().await;
    game.game_loop().await;
    Ok(())
}

fn move_object<T: piston_window::ImageSize>(scene: &mut Scene<T>, object_id: Uuid, delta_x: f64, delta_y: f64) {
    let obj = scene.child_mut(object_id).unwrap();
    obj.set_position(obj.get_position().0 + delta_x, obj.get_position().1 + delta_y);
}
