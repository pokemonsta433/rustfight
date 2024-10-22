use macroquad::prelude::*;

// for storing move data in YAML
use serde::{Deserialize, Serialize};
use serde_yaml::{self};
// for locking FPS to 60
use std::time::Instant;
use std::time::Duration;
use spin_sleep;
// for move lists
use std::collections::HashMap;
use std::ops::Index;
// for bugfixing
use std::fmt;

fn center_text(text: &str){
    let size = measure_text(text, None, 100, 1.0);
    draw_text(text, SCREEN_WIDTH/2.0 - (size.width/2.0), SCREEN_HEIGHT/5.0, 100.0, RED);
}

fn draw_fps(text: &str){
    if DRAW_FPS{
        draw_text(text, SCREEN_WIDTH - 60.0, SCREEN_HEIGHT/5.0, 30.0, RED);
    }
}

fn get_hold_dir() -> i32{ //TODO: put the flip logic in here and keep forward/back logic out of the main code
    let dir_x: i32 = is_key_down(KeyCode::B) as i32 - is_key_down(KeyCode::F) as i32;
    let dir_y: i32 = is_key_down(KeyCode::Space) as i32 - is_key_down(KeyCode::P) as i32;
    let mut hold_dir = 0;
    if dir_x == 1{
        hold_dir = match dir_y {
            1 => 9,
            0 => 6,
            -1 => 3,
            _=> 0
        };
    }
    else if dir_x == 0{
        hold_dir = match dir_y {
            1 => 8,
            0 => 5,
            -1 => 2,
            _=> 0
        };
    }
    else if dir_x == -1{
        hold_dir = match dir_y {
            1 => 7,
            0 => 4,
            -1 => 1,
            _=> 0
        };
    }
    hold_dir
}

#[derive(PartialEq)]
pub enum Status{ //most actionable at top
    Idle,
    WalkingForward,
    WalkingBack,
    Crouch,
    Air,
    ForwardJump,
    NeutralJump,
    BackJump,
    Dashing,
    Startup,
    Attacking, //only have to draw hitboxes in this part :)
    Jumpsquat, // 3ish frames
    Block,
    EndLag,
    Hitstun
}

pub enum CharacterList{ //most actionable at top
    MothGirl,
    MushroomGirl
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Status::Idle => write!(f, "Idle"),
            Status::WalkingForward => write!(f, "WalkingForward"),
            Status::WalkingBack => write!(f, "WalkingBack"),
            Status::Crouch => write!(f, "Crouch"),
            Status::Air => write!(f, "Air"),
            Status::ForwardJump => write!(f, "ForwardJump"),
            Status::NeutralJump => write!(f, "NeutralJump"),
            Status::BackJump => write!(f, "BackJump"),
            Status::Dashing => write!(f, "Dashing"),
            Status::Startup => write!(f, "Startup"),
            Status::Attacking => write!(f, "Attacking"), //only have to draw hitboxes in this part :)
            Status::Jumpsquat => write!(f, "Jumpsquat"), // 3ish frames
            Status::Block => write!(f, "Block"),
            Status::EndLag => write!(f, "EndLag"),
            Status::Hitstun => write!(f, "Hitstun"),
        }
    }
}


#[derive(Debug, Serialize, Deserialize)]
struct MasterMoveList
{
    moth_girl: HashMap<String, String>,
    mushroom_girl: HashMap<String, String>
}

impl Index<CharacterList> for MasterMoveList {
    type Output = HashMap<String, String>;

    fn index(&self, char: CharacterList) -> &Self::Output{
        match char {
            CharacterList::MothGirl => &self.moth_girl,
            CharacterList::MushroomGirl => &self.mushroom_girl,
        }
    }
}

struct Character<'a>{
    name: &'a str,
    image_height: f32,
    image_width: f32,
    move_speed: f32,
}

struct Player<'a>{
    char: Character<'a>,
    pos: Vec2,
    vel: Vec2,
    current_texture: Texture2D,
    status: Status,
    frames_remaining: i32,
}

const MOTH_GIRL: Character= Character{
    name : "moth_girl",
    image_height : 118.0,
    image_width : 100.0,
    move_speed : 7.0,
};

const MUSHROOM_GIRL: Character = Character{
    name : "mushroom_girl",
    image_height : 160.0,
    image_width : 110.0,
    move_speed : 5.5,
};

// some game constants
pub const SCREEN_HEIGHT: f32 = 1080.0;
pub const SCREEN_WIDTH: f32 = 1730.0; //1920.0
pub const FRAME_RATE: f32 = 60.0; //fps
pub const JUMP_SPEED: f32 = 35.0;
pub const GRAVITY: f32 = 2.0;
pub const FLOOR_HEIGHT: f32 = SCREEN_HEIGHT - 20.0;
pub const DRAW_FPS: bool = true;

pub trait GetTexture{
    fn get_texture(&self) -> &Texture2D;
}

pub trait GetFeet {
    fn get_feet(&self) -> f32;
}

pub trait IsGrounded {
    fn is_grounded(&self) -> bool;
}

impl GetTexture for Player<'_>{
    fn get_texture(&self) -> &Texture2D {
        //load_texture(&("assets/".to_owned() + self.char.name + "_idle.png")).await.unwrap()
        // println!("{}", self);
        // load_texture("assets/moth_girl_idle.png").await.unwrap()
        &self.current_texture
    }
}

impl GetFeet for Player<'_>{
    fn get_feet(&self) -> f32 {
        self.char.image_height + self.pos.y
    }
}

impl IsGrounded for Player<'_>{
    fn is_grounded(&self) -> bool {
        self.get_feet() >= FLOOR_HEIGHT
    }
}

#[macroquad::main("rustfight")]
async fn main() {

    let mut p1: Player = Player{
        char: MOTH_GIRL,
        pos: vec2(0.0, 0.0),
        vel: vec2(0.0, 0.0),
        //current_texture: "baba", //load_texture(&("assets/".to_owned() + MOTH_GIRL.name + "_idle.png")).await.unwrap(),
        current_texture: load_texture("assets/moth_girl_idle.png").await.unwrap(),
        status: Status::Idle,
        frames_remaining: -1,
    };

    let mut p2: Player = Player{
        char: MOTH_GIRL,
        pos: vec2(0.0, 0.0),
        vel: vec2(0.0, 0.0),
        // current_texture: "baba", // load_texture(&("assets/".to_owned() + MOTH_GIRL.name + "_idle.png")).await.unwrap(),
        current_texture: load_texture("assets/moth_girl_idle.png").await.unwrap(),
        status: Status::Idle,
        frames_remaining: -1,
    };


    // Player Character Select
    p1.char = MOTH_GIRL;
    p2.char = MUSHROOM_GIRL;

    // load the textures for the characters

    let f = std::fs::File::open("assets/MOVES_LIST.yml").expect("Could not open file.");
    let moves: MasterMoveList = serde_yaml::from_reader(f).expect("Could not read values.");

    let p1_moves = moves.moth_girl;
    println!("{:?}", p1_moves);

    // p1.current_texture = load_texture(p1.char.standing_texture).await.unwrap();
    // p2.current_texture = load_texture(p2.char.standing_texture).await.unwrap();

    // position and velocity variables
    p1.pos = vec2(SCREEN_WIDTH/4.0 - p1.char.image_width/2., FLOOR_HEIGHT - p1.char.image_height);
    p2.pos = vec2(SCREEN_WIDTH/4.0 * 3. - p2.char.image_width/2., FLOOR_HEIGHT - p2.char.image_height);
    p1.vel = vec2(0.0, 0.0);
    p2.vel = vec2(0.0, 0.0);


    loop {
        // do some important calcs:
        let frame_start_time = Instant::now();
        let mut flip_p1 = p1.pos.x > p2.pos.x;


        //TODO: check input into a buffer (last)

        let hold_dir = get_hold_dir();

        // parse input -- TODO: hide some of this logic away in a function?
        if matches!(p1.status, Status::WalkingForward | Status::WalkingBack | Status::Idle){
            if hold_dir == 6{
                if !flip_p1{
                    p1.status = Status::WalkingForward;
                    p1.frames_remaining = 1;
                }
                else{
                    p1.status = Status::WalkingBack;
                    p1.frames_remaining = 1;
                }
            }
            else if hold_dir == 4{
                if !flip_p1{
                    p1.status = Status::WalkingBack;
                    p1.frames_remaining = 1;
                }
                else{
                    p1.status = Status::WalkingForward;
                    p1.frames_remaining = 1;
                }
            }
            else if matches!(hold_dir, 7|8|9){
                p1.status = Status::Jumpsquat;
                p1.frames_remaining = 3;
            }
            else if matches!(hold_dir, 1|2|3){
                p1.status = Status::Crouch;
                p1.frames_remaining = -1;
            }
        }
        else if p1.status == Status::Jumpsquat && p1.frames_remaining == 0{
            if is_key_down(KeyCode::F){
                if flip_p1{
                    p1.status = Status::ForwardJump;
                    p1.frames_remaining = 1;
                }
                else{
                    p1.status = Status::BackJump;
                    p1.frames_remaining = 1;
                }
            }
            else if is_key_down(KeyCode::B){
                if flip_p1{
                    p1.status = Status::BackJump;
                    p1.frames_remaining = 1;
                }
                else{
                    p1.status = Status::ForwardJump;
                    p1.frames_remaining = 1;
                }
            }
            else{
                p1.status = Status::NeutralJump;
                p1.frames_remaining = 1;
            }
        }
        else if p1.status == Status::Crouch{
            if !matches!(hold_dir, 1|2|3){
                p1.status = Status::Idle;
            }
            else{
            }
        }

        //calculate movement
        if p1.is_grounded(){
            if p1.status == Status::WalkingForward{
                if p1.frames_remaining == 0{
                    p1.status = Status::Idle;
                }
                if flip_p1{
                    p1.vel.x -= p1.char.move_speed;
                }
                else{
                    p1.vel.x += p1.char.move_speed;
                }
            }
            else if p1.status == Status::WalkingBack{
                if p1.frames_remaining == 0{
                    p1.status = Status::Idle;
                }
                if flip_p1{
                    p1.vel.x += p1.char.move_speed;
                }
                else{
                    p1.vel.x -= p1.char.move_speed;
                }
            }
            else if p1.status == Status::NeutralJump{
                p1.vel.y -= JUMP_SPEED;
                p1.status = Status::Air;
            }
            else if p1.status == Status::BackJump{
                p1.vel.y -= JUMP_SPEED;
                if flip_p1{
                    p1.vel.x += p1.char.move_speed;
                }
                else{
                    p1.vel.x -= p1.char.move_speed;
                }
                p1.status = Status::Air;
            }
            else if p1.status == Status::ForwardJump{
                p1.vel.y -= JUMP_SPEED;
                if flip_p1{
                    p1.vel.x -= p1.char.move_speed;
                }
                else{
                    p1.vel.x += p1.char.move_speed;
                }
                p1.status = Status::Air;
            }
            else if p1.status == Status::Air{
                p1.status = Status::Idle;
                p1.frames_remaining = -1;
            }
        }
        else{
            // all you do is fall, bitch
            p1.vel.y += GRAVITY;
        }

        if p2.is_grounded(){
        }
        else{
            // all you do is fall, bitch
            p2.vel.y += GRAVITY;
        }

        // calc next movement
        let mut p1_nextpos = p1.pos + p1.vel;
        let mut p2_nextpos = p2.pos + p2.vel;

        // only collide on the ground
        //if p1.is_grounded() && p2.is_grounded(){
        if !matches!(p1.status, Status::Air | Status::Jumpsquat) && p2.status != Status::Air{
            if flip_p1{ //p2 on left
                let overlap = p2_nextpos.x + p2.char.image_width - p1_nextpos.x;
                if overlap > 0.0{
                    p2_nextpos -= overlap/2.0;
                    p1_nextpos += overlap/2.0;
                }
            }
            else{ //p1 on left
                let overlap = p1_nextpos.x + p1.char.image_width - p2_nextpos.x;
                if overlap > 0.0{
                    p1_nextpos -= overlap/2.0;
                    p2_nextpos += overlap/2.0;
                }
            }
        }

        //tick movement
        p1.pos = p1_nextpos;
        p2.pos = p2_nextpos;

        //stop on boundaries
        if p1.get_feet() > FLOOR_HEIGHT{p1.pos.y = FLOOR_HEIGHT - p1.char.image_height}
        if p2.get_feet() > FLOOR_HEIGHT{p2.pos.y = FLOOR_HEIGHT - p2.char.image_height}
        if p1.pos.x < 0.0 { p1.pos.x = 0.0}
        if p2.pos.x < 0.0 { p2.pos.x = 0.0}
        if p1.pos.x + p1.char.image_width > SCREEN_WIDTH {p1.pos.x = SCREEN_WIDTH - p1.char.image_width}
        if p2.pos.x + p2.char.image_width > SCREEN_WIDTH {p2.pos.x = SCREEN_WIDTH - p2.char.image_width}


        // nicer to read if we just call this again
        if p1.is_grounded() {p1.vel.x = 0.0}
        if p2.get_feet() >= FLOOR_HEIGHT {p2.vel.x = 0.0}

        // draw
        clear_background(DARKGRAY);

        // test stuff
        // center_text(&p1.frames_remaining.to_string());
        // center_text(&p1.get_feet().to_string());

        flip_p1 = p1.pos.x > p2.pos.x;

        draw_texture_ex(
            &p1.current_texture,
            p1.pos.x,
            p1.pos.y,
            WHITE,
            DrawTextureParams{
                flip_x: flip_p1,
                ..Default::default()
            },
            );

        draw_texture_ex(
            &p2.current_texture,
            p2.pos.x,
            p2.pos.y,
            WHITE,
            DrawTextureParams{
                flip_x: !flip_p1,
                ..Default::default()
            },
            );

        // draw the floor
        draw_rectangle(0., FLOOR_HEIGHT, SCREEN_WIDTH, 1., GREEN);

        // draw some hurtbox indicators
        draw_rectangle(p1.pos.x, p1.pos.y, 10., 1., BLUE);
        draw_rectangle(p1.pos.x, p1.pos.y, 1., 10., BLUE);
        draw_rectangle(p1.pos.x + p1.char.image_width - 10., p1.pos.y + p1.char.image_height -1., 10., 1., BLUE);
        draw_rectangle(p1.pos.x + p1.char.image_width - 1., p1.pos.y + p1.char.image_height - 10., 1., 10., BLUE);

        draw_rectangle(p2.pos.x, p2.pos.y, 10., 1., RED);
        draw_rectangle(p2.pos.x, p2.pos.y, 1., 10., RED);
        draw_rectangle(p2.pos.x + p2.char.image_width - 10., p2.pos.y + p2.char.image_height -1., 10., 1., RED);
        draw_rectangle(p2.pos.x + p2.char.image_width - 1., p2.pos.y + p2.char.image_height - 10., 1., 10., RED);


        p1.frames_remaining -= 1;
        p2.frames_remaining -= 1;

        // lock to 60FPS
        let nanoseconds_per_frame = (1./FRAME_RATE) * 1000000000.0;
        let expected_time_per_frame = Duration::from_nanos(nanoseconds_per_frame as u64);
        let wait_time = expected_time_per_frame - frame_start_time.elapsed();
        spin_sleep::sleep(wait_time);
        let fps_this_frame = 1. / ((frame_start_time.elapsed().as_nanos()) as f32 / 1000000000.0);
        draw_fps((fps_this_frame as u64).to_string().as_str());

        center_text(p1.status.to_string().as_str());

        next_frame().await
    }
    }
