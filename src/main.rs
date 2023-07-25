use std::collections::HashSet;
use std::time::Duration;

use sdl2::EventPump;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;

const WINDOW_WIDTH: u32 = 800;
const WINDOW_HEIGTH: u32 = 800;
const BLOCK_SIZE: u32 = 5;

fn calculate_object_right(object: &Block) -> i32 {
    BLOCK_SIZE as i32 * object.rect.w / BLOCK_SIZE as i32
}

trait Pedal{
    fn handle_movement(&mut self, event_pump: &mut EventPump, prev_keys: &HashSet<Keycode>);
}
impl Pedal for Block{
    fn handle_movement(&mut self, event_pump: &mut EventPump, prev_keys: &HashSet<Keycode>) {
        let keys = event_pump
            .keyboard_state()
            .pressed_scancodes()
            .filter_map(Keycode::from_scancode)
            .collect();

        let new_keys = &keys - prev_keys;
        let block_right = calculate_object_right(&self);
        if !keys.is_empty(){
            for key in new_keys{
                if key == Keycode::Left && !(self.border.x <= 0) {
                    self.rect.set_x(self.rect.x() - BLOCK_SIZE as i32 * 2);
                    self.border.set_x(self.rect.x());

                }
                if key == Keycode::Right && !(self.border.x + block_right >= WINDOW_WIDTH as i32){
                    self.rect.set_x(self.rect.x() + BLOCK_SIZE as i32 * 2);
                    self.border.set_x(self.rect.x());
                }
            }
        }
        
    }
}
trait Ball{
    fn handle_wall(&mut self);
    fn react_object(&mut self, object: &Block)-> bool;
}
impl Ball for Block{
    fn handle_wall(&mut self){
        if self.border.x <= 0 || self.border.x + BLOCK_SIZE as i32 * 10   >= WINDOW_WIDTH as i32 {
            self.velocity_x = -self.velocity_x;
        }
        if self.border.y <= 0 ||self.border.y + BLOCK_SIZE as i32 * 10 >= WINDOW_HEIGTH as i32 {
            self.velocity_y = -self.velocity_y;
        }
    } 
    fn react_object(&mut self, object:&Block)->bool{
        let block_right = calculate_object_right(&self);
        if (self.border.y == object.border.y() || self.border.y + BLOCK_SIZE as i32 * 10 == object.border.y()) && (self.border.x >= object.border.x() && self.border.x <= object.border.x() + calculate_object_right(object)) {
            self.velocity_y = -self.velocity_y;
            return true;
        }       
        false 
    }
}
struct Block {
    rect: Rect,
    border: Rect,
    velocity_x: i32,
    velocity_y: i32,
    rect_color: Color,
    border_color:  Color
}
impl Block{
    fn new(
        rect: Rect, velocity_x: i32, velocity_y: i32, rect_color: Color
    ) -> Self{
        let border_color = Color::RED;
        let border = Rect::new(rect.x(), rect.y(), rect.w as u32, rect.h as u32);
        Self { rect: rect, border: border, velocity_x: velocity_x, velocity_y: velocity_y, rect_color: rect_color, border_color: border_color }
    }

    fn render(&mut self, canvas: &mut sdl2::render::Canvas<Window>, ){
        canvas.set_draw_color(self.rect_color);
        canvas.fill_rect(self.rect).unwrap();
        canvas.set_draw_color(self.border_color);
        canvas.draw_rect(self.border).unwrap();
        canvas.present();
    }
   
    fn update_position(&mut self){
        
        self.rect.x += self.velocity_x;
        self.rect.y += self.velocity_y;
        self.border.x += self.velocity_x;
        self.border.y += self.velocity_y;
    }

    

}
fn clear_screen(canvas: &mut sdl2::render::Canvas<Window>){
    canvas.set_draw_color(Color::BLACK);
    canvas.clear();
    canvas.present();
}

pub fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let mut prev_keys: HashSet<Keycode> = HashSet::new();


    let window = video_subsystem
        .window("divine snake", WINDOW_WIDTH, WINDOW_HEIGTH)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();

    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();
    canvas.present();
    let mut event_pump = sdl_context.event_pump().unwrap();
    
    let ball_rect = Rect::new(40, 40, BLOCK_SIZE*10, BLOCK_SIZE*10);
    let mut ball = Block::new(ball_rect, BLOCK_SIZE as i32 *2, BLOCK_SIZE as i32 *1, Color::WHITE);

    let player1_rect = Rect::new(750, 750, BLOCK_SIZE*24, BLOCK_SIZE*3);
    let mut player1 = Block::new(player1_rect, 0, 0, Color::WHITE);
    
    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                _ => {}
            }
        }
        clear_screen(&mut canvas);

        


        player1.handle_movement(&mut event_pump, &prev_keys);
        player1.update_position();
        player1.render(&mut canvas);

        ball.handle_wall();
        ball.react_object(&player1);
        
        ball.update_position();
        ball.render(&mut canvas);
        
        canvas.present();
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
      
    }
}
