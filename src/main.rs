extern crate termion;

use lazy_static::lazy_static;
use rand::{
    distributions::{Distribution, Standard},
    Rng,
};
use std::collections::HashMap;
use std::io;
use std::io::{stdout, Write};
use std::slice;
use std::sync::mpsc;
use std::time::{SystemTime, UNIX_EPOCH};
use std::{thread, time};
use termion::clear;
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use termion::{color, cursor, raw};

// game board with boundaries
const BOARD_SIZE_X: usize = 16;
const BOARD_SIZE_Y: usize = 27;

const BLOCK_SIZE_X: usize = 4;
const BLOCK_SIZE_Y: usize = 4;

type BoardArray = Vec<[i8; BOARD_SIZE_X]>;
type BlockArray = [[i8; BLOCK_SIZE_X]; BLOCK_SIZE_Y];

lazy_static! {
    static ref COLOR_TABLE : HashMap<i8, u8> = {
        let mut map = HashMap::new();
        map.insert(-2, 0);          // black
        map.insert(-1, 1);          // red
        map.insert(0, 0);           // black
        map.insert(1, 2);           // green
        map.insert(2, 226);         // yellow
        map.insert(3, 27);          // blue
        map.insert(4, 129);         // violet
        map.insert(5, 231);         // white
        map
    };
    static ref TERMINAL_SIZE: (u16, u16) = termion::terminal_size().unwrap();
    static ref TERMINOS: [[[[i8; BLOCK_SIZE_X]; BLOCK_SIZE_Y]; 4]; 7] = [
        [
            [[0, 0, 2, 0], [0, 0, 2, 0], [0, 0, 2, 0], [0, 0, 2, 0]],
            [[0, 0, 0, 0], [0, 0, 0, 0], [2, 2, 2, 2], [0, 0, 0, 0]],
            [[0, 0, 2, 0], [0, 0, 2, 0], [0, 0, 2, 0], [0, 0, 2, 0]],
            [[0, 0, 0, 0], [0, 0, 0, 0], [2, 2, 2, 2], [0, 0, 0, 0]]
        ],
        [
            [[0, 2, 0, 0], [0, 2, 0, 0], [2, 2, 0, 0], [0, 0, 0, 0]],
            [[2, 0, 0, 0], [2, 2, 2, 0], [0, 0, 0, 0], [0, 0, 0, 0]],
            [[0, 2, 2, 0], [0, 2, 0, 0], [0, 2, 0, 0], [0, 0, 0, 0]],
            [[0, 0, 0, 0], [2, 2, 2, 0], [0, 0, 2, 0], [0, 0, 0, 0]]
        ],
        [
            [[0, 2, 0, 0], [0, 2, 0, 0], [0, 2, 2, 0], [0, 0, 0, 0]],
            [[0, 0, 0, 0], [2, 2, 2, 0], [2, 0, 0, 0], [0, 0, 0, 0]],
            [[2, 2, 0, 0], [0, 2, 0, 0], [0, 2, 0, 0], [0, 0, 0, 0]],
            [[0, 0, 2, 0], [2, 2, 2, 0], [0, 0, 0, 0], [0, 0, 0, 0]]
        ],
        [[[0, 2, 2, 0], [0, 2, 2, 0], [0, 0, 0, 0], [0, 0, 0, 0]]; 4],
        [
            [[0, 0, 0, 0], [0, 2, 2, 0], [2, 2, 0, 0], [0, 0, 0, 0]],
            [[2, 0, 0, 0], [2, 2, 0, 0], [0, 2, 0, 0], [0, 0, 0, 0]],
            [[0, 0, 0, 0], [0, 2, 2, 0], [2, 2, 0, 0], [0, 0, 0, 0]],
            [[2, 0, 0, 0], [2, 2, 0, 0], [0, 2, 0, 0], [0, 0, 0, 0]]
        ],
        [
            [[0, 0, 0, 0], [2, 2, 0, 0], [0, 2, 2, 0], [0, 0, 0, 0]],
            [[0, 2, 0, 0], [2, 2, 0, 0], [2, 0, 0, 0], [0, 0, 0, 0]],
            [[0, 0, 0, 0], [2, 2, 0, 0], [0, 2, 2, 0], [0, 0, 0, 0]],
            [[0, 2, 0, 0], [2, 2, 0, 0], [2, 0, 0, 0], [0, 0, 0, 0]]
        ],
        [
            [[0, 0, 0, 0], [2, 2, 2, 0], [0, 2, 0, 0], [0, 0, 0, 0]],
            [[0, 2, 0, 0], [2, 2, 0, 0], [0, 2, 0, 0], [0, 0, 0, 0]],
            [[0, 2, 0, 0], [2, 2, 2, 0], [0, 0, 0, 0], [0, 0, 0, 0]],
            [[0, 2, 0, 0], [0, 2, 2, 0], [0, 2, 0, 0], [0, 0, 0, 0]]
        ]
    ];
}

pub trait Drawable {
    type E;

    fn get_obj(&self) -> slice::Iter<Self::E>;
    fn get_size(&self) -> (usize, usize);
    fn get_data(&self, x: usize, y: usize) -> i8;
}

struct Board {
    data: BoardArray,
}

impl Drawable for Board {
    type E = [i8; BOARD_SIZE_X];
    fn get_obj(&self) -> slice::Iter<[i8; BOARD_SIZE_X]> {
        self.data.iter()
    }

    fn get_size(&self) -> (usize, usize) {
        (BOARD_SIZE_X, BOARD_SIZE_Y)
    }

    fn get_data(&self, x: usize, y: usize) -> i8 {
        self.data[x][y]
    }
}

impl Board {
    pub fn init(&mut self) {
        for _i in 0..BOARD_SIZE_Y {
            self.data.push([
                -1, -1, -1, -2, -2, -2, -2, -2, -2, -2, -2, -2, -2, -1, -1, -1,
            ]);
        }
        self.data[25] = [-1; 16];
        self.data[26] = [-1; 16];
    }
    pub fn new() -> Board {
        Board { data: Vec::new() }
    }

    // see if possible move
    fn check_with_block(&mut self, x: usize, y: usize, block: &Block) -> Result<u8, i8> {
        let mut y_ = y;
        for row in block.data.iter() {
            let mut x_ = x;
            for col in row.iter() {
                let cell = self.data[y_][x_];
                if *col != 0 {
                    match cell {
                        -2 => {}
                        -1..=6 => return Ok(1), // wall
                        _ => return Err(cell),  // not possible
                    }
                }
                x_ = x_ + 1;
            }
            y_ = y_ + 1;
        }
        return Ok(0);
    }

    fn check_completion(&mut self) -> u32 {
        let mut counter = 0;

        // remove completed lines, skip the bottom
        self.data.retain(|&col| {
            let retained: bool = |col: &[i8; BOARD_SIZE_X]| -> bool {
                if col.iter().position(|&r| r <= 0) == Option::None {
                    false
                } else {
                    true
                }
            }(&col);
            if retained == false {
                counter = counter + 1;
            }

            retained
        });

        // add new lines
        for _i in 0..counter {
            self.data.push([
                -1, -1, -1, -2, -2, -2, -2, -2, -2, -2, -2, -2, -2, -1, -1, -1,
            ]);
        }

        counter
    }
    fn set_with_block(&mut self, x: u8, y: u8, block: &Block) -> Result<u8, &'static str> {
        // cell value represents...
        // 0 : empty space
        // 1 : wall
        // 2 : moving
        // 3 : stacked

        // clean up moving cells
        // for row in self.data.iter_mut() {
        //     for cell in row.iter_mut() {
        //         if *cell == 2 || *cell == 4 {
        //             *cell = -2;
        //         }
        //     }
        // }

        // copy moving block onto board
        let mut y_ = y as usize;

        for row in block.data.iter() {
            let mut x_ = x as usize;
            for col in row.iter() {
                let cell = &mut self.data[y_][x_];
                if *col != 0 {
                    *cell = *col;
                }
                x_ = x_ + 1;
            }

            y_ = y_ + 1;
        }

        Ok(0)
    }
}

enum GameScene {
    INTRO,
    GAME,
    END,
}

#[derive(Copy, Clone)]
enum BlockType {
    I,
    J,
    L,
    O,
    S,
    Z,
    T,
}

impl Distribution<BlockType> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> BlockType {
        match rng.gen_range(0, 7) {
            0 => BlockType::I,
            1 => BlockType::J,
            2 => BlockType::L,
            3 => BlockType::O,
            4 => BlockType::S,
            5 => BlockType::Z,
            _ => BlockType::T,
        }
    }
}

#[derive(Copy, Clone)]
struct Block {
    data: BlockArray,
    _type: BlockType,
    rotation: u8,
    x: u8,
    y: u8,
    state: BlockState,
    color: u8,
}

impl Drawable for Block {
    type E = [i8; BLOCK_SIZE_X];
    //
    fn get_obj(&self) -> slice::Iter<[i8; BLOCK_SIZE_X]> {
        self.data.iter()
    }

    fn get_data(&self, x: usize, y: usize) -> i8 {
        self.data[x][y]
    }

    fn get_size(&self) -> (usize, usize) {
        (BLOCK_SIZE_X, BLOCK_SIZE_Y)
    }
}

impl Block {
    pub fn new() -> Block {
        let _type = rand::random();
        let rotation: u8 = rand::random::<u8>() % 4;
        // let mut block_data: BlockArray = TERMINOS[_type as usize][rotation as usize];
        let color = rand::random::<u8>() % 5 + 1;

        // let block_data:&BlockArray = block_data;
        let block_data = Block::get_block_with_color(_type as usize, rotation as usize, color);
        Block {
            _type: _type,
            data: block_data,
            rotation: rotation,
            state: BlockState::DROPPING,
            x: 6,
            y: 0,
            color: color,
        }
    }

    fn get_block_with_color(_type: usize, rotation: usize, color: u8) -> BlockArray {
        let mut block_data: BlockArray = TERMINOS[_type as usize][rotation as usize];

        for r in block_data.iter_mut() {
            for c in &mut r.iter_mut() {
                if *c == 2 {
                    *c = color as i8;
                }
            }
        }

        block_data
    }

    pub fn rotate(&mut self) {
        self.rotation = (self.rotation + 1) % 4;

        self.data =
            Block::get_block_with_color(self._type as usize, self.rotation as usize, self.color);

        // self.data = TERMINOS[self._type as usize][self.rotate as usize];

        // for r in self.data.iter_mut() {
        //     for c in &mut r.iter_mut() {
        //         if *c == 2 {
        //             *c = self.color as i8;
        //         }
        //     }
        // }
    }
}

fn draw_obj(
    objs: &impl Drawable,
    out: &mut raw::RawTerminal<std::io::Stdout>,
    x: u16,
    mut y: u16,
    mask: i8,
) {
    let (w, h) = objs.get_size();

    for row in 0..h {
        println!("{}", cursor::Goto(x, y));
        for col in 0..w {
            let data = objs.get_data(row, col);
            match data {
                // -1 => {
                //     write!(out, "{}  ", color::Bg(color::AnsiValue(1))).unwrap();
                // }
                // 2 => {
                //     write!(out, "{}  ", color::Bg(color::Yellow)).unwrap();
                // }
                v if v == mask => {
                    write!(out, "{}", cursor::Right(2)).unwrap();
                }
                // -2 => {
                // write!(out, "{}  ", color::Bg(color::Black)).unwrap();
                // }
                _ => {
                    // println!("vlue is {}", &data);
                    let c = COLOR_TABLE.get(&data);
                    match c {
                        Some(v) => {
                            write!(out, "{}  ", color::Bg(color::AnsiValue(*v))).unwrap();
                        }
                        None => {
                            println!("none is {}", data);
                        }
                    }
                }
            }
        }
        y = y + 1;
        write!(out, "\n\r").unwrap();
    }
}

fn draw_score(score: u32, out: &mut raw::RawTerminal<std::io::Stdout>) {
    println!("{}", cursor::Goto(40, 1));
    println!("{}", cursor::Hide);

    write!(out, "{}Score : {}", color::Bg(color::Black), score).unwrap();
}

fn draw_intro(out: &mut raw::RawTerminal<std::io::Stdout>) {
    println!("{}", clear::All);
    let (x, y) = (TERMINAL_SIZE.0 / 2 - 10, TERMINAL_SIZE.1 / 2);
    println!("{}RusTetris (ver 0.1.0)", cursor::Goto(x, y));
    println!("{}Spacebar : Start", cursor::Goto(x + 2, y + 2));
    println!("{}Q : Quit", cursor::Goto(x + 2, y + 3));
}

fn clear_display() {
    println!("{}{}", color::Bg(color::Black), clear::All);
}

#[derive(Copy, Clone)]
enum BlockState {
    STACKED,
    DROPPING,
}

fn block_movement<'a>(v: &str, board: &mut Board, block: &'a mut Block) -> &'a Block {
    let mut new_x = block.x;
    let mut new_y = block.y;
    let mut block_tmp = *block;
    match v {
        "movetoleft" => {
            new_x = new_x - 1;
        }
        "movetoright" => {
            new_x = new_x + 1;
        }
        "movedown" => {
            // see if touch the ground
            match board
                .check_with_block(block.x as usize, block.y as usize + 1, &block_tmp)
                .unwrap()
            {
                0 => {
                    new_y = new_y + 1;
                }
                _ => {
                    block.state = BlockState::STACKED;
                    return block;
                }
            }
        }
        "rotate" => {
            block_tmp.rotate();
        }
        "drop" => {
            let mut dropped = false;
            let mut i = 1;
            while !dropped {
                match board
                    .check_with_block(block.x as usize, block.y as usize + i, &block_tmp)
                    .unwrap()
                {
                    0 => {
                        i = i + 1;
                        new_y = new_y + 1;
                    }
                    _ => dropped = true,
                }
            }
        }
        _ => {}
    };
    match board
        .check_with_block(new_x as usize, new_y as usize, &block_tmp)
        .unwrap()
    {
        0 => {
            *block = block_tmp;
            block.x = new_x;
            block.y = new_y;
        }
        _ => {}
    }

    block.state = BlockState::DROPPING;
    return block;
}

fn read_input(sender: mpsc::Sender<&str>) -> Result<u8, u8> {
    for key in io::stdin().keys() {
        match key.unwrap() {
            Key::Char('q') => {
                sender.send("break").unwrap();
                break;
            }
            Key::Up => sender.send("rotate").unwrap(),
            Key::Down => sender.send("movedown").unwrap(),
            Key::Left => sender.send("movetoleft").unwrap(),
            Key::Right => sender.send("movetoright").unwrap(),
            Key::Char(' ') => sender.send("drop").unwrap(),
            _ => {}
        }
    }

    Ok(0)
}

fn main() {
    let mut board = Board::new();
    board.init();

    let mut out = stdout().into_raw_mode().unwrap();

    let break_duration = time::Duration::from_millis(1);

    // message channel
    let (tx, rx) = mpsc::channel();

    let mut dropping_block = Block::new();
    let mut next_block = Block::new();

    let mut score: u32 = 0;
    let mut game_scene = GameScene::INTRO;

    let actor = thread::spawn(move || loop {
        match game_scene {
            GameScene::INTRO => {
                draw_intro(&mut out);

                if rx.recv().ok() == Some("drop") {
                    clear_display();
                    game_scene = GameScene::GAME;
                }
            }
            GameScene::GAME => {
                if let Ok(ret) = rx.recv() {
                    match ret {
                        "break" => {
                            break;
                        }
                        _ => match block_movement(&ret, &mut board, &mut dropping_block).state {
                            BlockState::STACKED => {
                                board
                                    .set_with_block(
                                        dropping_block.x,
                                        dropping_block.y,
                                        &dropping_block,
                                    )
                                    .unwrap();
                                dropping_block = next_block;
                                next_block = Block::new();
                            }
                            _ => {}
                        },
                    }
                };

                // check line completion
                let lines_completed = board.check_completion();
                score = score + lines_completed * 100;
                draw_obj(&board, &mut out, 1, 1, 0);
                draw_obj(&next_block, &mut out, 45, 10, -2);
                draw_obj(
                    &dropping_block,
                    &mut out,
                    1 + (dropping_block.x * 2) as u16,
                    1 + dropping_block.y as u16,
                    0,
                );
                draw_score(score, &mut out);
            }
            GameScene::END => {}
        }
        thread::sleep(break_duration);
    });

    let sender = tx.clone();
    let input_handler = thread::spawn(move || {
        read_input(sender).unwrap();
    });

    let sender = tx.clone();
    thread::spawn(move || {
        let mut start = SystemTime::now();
        let speed = 1000;

        loop {
            let now = SystemTime::now();

            if now.duration_since(start).unwrap().as_millis() > speed {
                sender.send("movedown").unwrap();
                start = now;
            }
            thread::sleep(break_duration);
        }
    });

    let _scene_manager = thread::spawn(move || {});

    input_handler.join().unwrap();
    actor.join().unwrap();
}
