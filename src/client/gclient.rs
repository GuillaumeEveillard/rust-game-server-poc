extern crate piston_window;
extern crate ai_behavior;
extern crate sprite;
extern crate find_folder;

use std::rc::Rc;

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
use gfx_device_gl::{CommandBuffer, Resources};
use std::path::PathBuf;

struct Position { x: u32, y: u32}

struct GLivingBeing {
    name: String,
    position: Position,
    health : u32,
    sprite_id: Uuid
}

impl GLivingBeing {
    fn render(&self, window: &mut PistonWindow, scene: &mut Scene<Texture<Resources>>) {
        let sprite = scene.child_mut(self.sprite_id).unwrap();
        sprite.set_position(self.position.x as f64, self.position.y as f64);
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

fn main() {
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

    while let Some(e) = window.next() {
        scene.event(&e);

        window.draw_2d(&e, |c, g, _| {
            clear([1.0, 1.0, 1.0, 1.0], g);
            scene.draw(c.transform, g);

            let red = [1.0, 0.0, 0.0, 1.0];
            let black = [0.0, 0.0, 0.0, 1.0];
            let rect = math::margin_rectangle([20.0, 20.0, 100.0, 10.0], 1.0);
            rectangle(red, rect, c.transform, g);
            Rectangle::new_border(black, 2.0).draw(rect, &c.draw_state, c.transform, g);
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

        if let Some(_) = e.press_args() {
            scene.toggle(id, &seq);
            scene.toggle(id, &rotate);
        }
    }
}

fn move_object<T: piston_window::ImageSize>(scene : &mut Scene<T>, object_id: Uuid, delta_x: f64, delta_y: f64) {
    let obj = scene.child_mut(object_id).unwrap();
    obj.set_position(obj.get_position().0 + delta_x, obj.get_position().1 + delta_y);
}