use std::collections::HashSet;
use std::net::{TcpStream, SocketAddr};
use std::time::Duration;

use sdl2::EventPump;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::{Canvas, TextureQuery, TextureCreator};
use sdl2::sys::Font;
use sdl2::ttf::{FontStyle, Sdl2TtfContext};
use sdl2::video::{Window, WindowContext};
use serde::{Serialize, Deserialize};
use tungstenite::Message;
use tungstenite::stream::MaybeTlsStream;
use tungstenite::protocol::WebSocket;

use uuid::Uuid;

const WINDOW_WIDTH: u32 = 800;
const WINDOW_HEIGTH: u32 = 800;
const BLOCK_SIZE: u32 = 5;
const FONT_PATH: &str = "./font.ttf";

fn calculate_object_right(object: &Block) -> i32 {
    BLOCK_SIZE as i32 * object.rect.w / BLOCK_SIZE as i32
}
fn draw_text(canvas: &mut Canvas<Window>, ttf_context: &Sdl2TtfContext, texture_creator: &mut TextureCreator<WindowContext>, text: &str, x:i32, y:i32, font_size:u16, color: Color){
    let mut font = ttf_context.load_font(FONT_PATH, 40).unwrap();
    font.set_style(sdl2::ttf::FontStyle::BOLD);
    
    let surface = font
        .render(text)
        .blended(color)
        .map_err(|e| e.to_string()).unwrap();
    let texture = texture_creator
        .create_texture_from_surface(&surface)
        .map_err(|e| e.to_string()).unwrap();

    let TextureQuery { width, height, .. } = texture.query();

    let target = Rect::new(
        x,
        y,
        font_size as u32 * 2,
        font_size as u32,
    );

    canvas.copy(&texture, None, Some(target)).unwrap();
    //canvas.present(); no need, we already updating canvas after render
    //println!("font rendered, {:?}", target);
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
    fn handle_score(&mut self, score: &mut (u32,u32));
    fn restart(&mut self);
}
impl Ball for Block{
    fn handle_wall(&mut self){
        if self.border.x <= 0 || self.border.x + BLOCK_SIZE as i32 * 10   >= WINDOW_WIDTH as i32 {
            self.velocity_x = -self.velocity_x;
        }
        //self.border.y + BLOCK_SIZE as i32 * 10 >= WINDOW_HEIGTH as i32
        if self.border.y <= 0  {
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
    fn handle_score(&mut self, score: &mut (u32,u32)){
        if(self.border.y() + self.border.height() as i32 <= 0){
            score.1 += 1;
            self.restart();
        }
        if(self.border.y() >= WINDOW_HEIGTH as i32){
            score.0 += 1;
            self.restart();
        }
    }
    fn restart(&mut self) {
        self.rect.x = 400;
        self.rect.y = 400;
        self.border.x = 400;
        self.border.y = 400;
        self.velocity_x = 0;
        self.velocity_y = 0;
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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RectRecv {
    x: i32,
    y: i32,
    w: u32,
    h: u32,
}



#[derive(Debug, Serialize, Deserialize, Clone)]
struct Client {
    addr: SocketAddr,
    block: BlockRecv,
    sent: bool,
    id: String
}


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BlockRecv {
    pub rect: RectRecv,
    pub border: RectRecv,
    pub velocity_x: i32,
    pub velocity_y: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GameRecv {
    score: (u32,u32),
    tick: u64,
    clients: Vec<Client>,
    is_started: bool,
    ball: BlockRecv,
    headers_sent: bool
}

fn sync_server_send(socket: &mut WebSocket<MaybeTlsStream<TcpStream>>, player_to_send: BlockRecv, game_recv: &mut GameRecv){
    
    let data_json = serde_json::to_string(&player_to_send).unwrap();
    let msg = Message::Text(data_json);
    
    socket.write_message(msg).unwrap();
    
}
fn sync_server_read(socket: &mut WebSocket<MaybeTlsStream<TcpStream>>) -> GameRecv{
    let msg= socket.read_message().unwrap();
    
    let json_value: GameRecv = serde_json::from_str(msg.to_string().as_str()).unwrap();
    json_value
}

fn await_new_tick(mut current_tick: u64, socket: &mut WebSocket<MaybeTlsStream<TcpStream>>){
    let mut new_tick = false;
    loop{

        if new_tick{
            break;
        }
        //read the game
        let game_recv = sync_server_read(socket);

        if current_tick < game_recv.tick{
            current_tick = game_recv.tick;
            new_tick = true;
        }
        
    }
    
}

pub fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let prev_keys: HashSet<Keycode> = HashSet::new();

    let window = video_subsystem
        .window("divine snake", WINDOW_WIDTH, WINDOW_HEIGTH)
        .position_centered()
        .opengl()
        .build()
        .unwrap();

    //tick 
    let mut tick: u64 = 0;
    //server
    let (mut socket, response) = tungstenite::connect(
        url::Url::parse("ws://localhost:9001").unwrap()
    ).expect("Can't connect");
    let addr: String = socket.read_message().unwrap().to_string();
    
    println!("addr : {}", addr);

    let mut canvas = window.into_canvas().present_vsync().build().unwrap();
    let ttf_context = sdl2::ttf::init().map_err(|e| e.to_string()).unwrap();
        

    let mut score = (0,0);
    let mut game_runs: bool = false;
    let mut info_text = "Press enter.".to_string();


    let mut texture_creator = canvas.texture_creator();
    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();
    canvas.present();
    let mut event_pump = sdl_context.event_pump().unwrap();


    let ball_rect = Rect::new(40, 40, BLOCK_SIZE*10, BLOCK_SIZE*10);
    let mut ball = Block::new(ball_rect, BLOCK_SIZE as i32 *5, BLOCK_SIZE as i32 *1, Color::WHITE);

    let player1_rect = Rect::new(750, 750, BLOCK_SIZE*24, BLOCK_SIZE*3);
    let mut player1 = Block::new(player1_rect, 0, 0, Color::WHITE);

    let enemy_rect = Rect::new(50, 50, BLOCK_SIZE*24, BLOCK_SIZE*3);
    let mut enemy = Block::new(enemy_rect, 0, 0, Color::WHITE);


    'running: loop {
       
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                Event::KeyDown { keycode: Some(Keycode::KpEnter), ..}
                => {
                    if ball.velocity_x == 0 && ball.velocity_y == 0 {
                        ball.velocity_x = BLOCK_SIZE as i32 * 5;
                        ball.velocity_y = BLOCK_SIZE as i32 * 1;
                    }
                    println!("damn");
                    if !game_runs {
                        game_runs = true;
                        score = (0,0);
                    }
                }
                _ => {}
            }
        }
        clear_screen(&mut canvas);
        //updating score
        if game_runs {
            info_text = format!("Score: {:?}", score);
        }
        // printing score
        draw_text(&mut canvas, &ttf_context, &mut texture_creator, &info_text, 0, 700, 80, Color::WHITE);
        if score.0 == 5 {
            game_runs = false;
            info_text = "Player 1 Won!".to_string();
            ball.restart();

        }
        else if score.1 == 5{
            game_runs = false;
            info_text = "Player 2 Won!".to_string();
            ball.restart();
        }
        let game_recv = sync_server_read(&mut socket);
        
        for client in game_recv.clients{
            if client.addr.to_string() == addr {
                player1.rect.x = client.block.rect.x;
                player1.border.x = client.block.border.x;

            }else{
                enemy.rect.x = client.block.rect.x;
                enemy.border.x = client.block.border.x;
            }
        }
        ball.rect.x = game_recv.ball.rect.x;
        ball.rect.y = game_recv.ball.rect.y;
        ball.border.x = game_recv.ball.border.x;
        ball.border.y = game_recv.ball.border.y;

        println!("new tick!");
        if game_runs {
            //player1.handle_movement(&mut event_pump, &prev_keys);
            player1.render(&mut canvas);
            
            enemy.render(&mut canvas);

            ball.render(&mut canvas);
        }

        //wait's new tick
        //await_new_tick(tick, &mut socket);
        
        
        canvas.present();
        //::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
      
    }
}
