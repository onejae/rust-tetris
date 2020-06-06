extern crate termion;

use lazy_static::lazy_static;
use rand::{
    distributions::{Distribution, Standard},
    Rng,
};
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
use termion::{color, cursor};

// game board with boundaries
const BOARD_SIZE_X: usize = 16;
const BOARD_SIZE_Y: usize = 27;

const BLOCK_SIZE_X: usize = 4;
const BLOCK_SIZE_Y: usize = 4;

type BoardArray = Vec<[i8; BOARD_SIZE_X]>;
type BlockArray = [[i8; BLOCK_SIZE_X]; BLOCK_SIZE_Y];

lazy_static! {
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

use std::any::type_name;

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
            self.data
                .push([-1, -1, -1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -1, -1, -1]);
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
                if *col == 2 {
                    match cell {
                        0 | 2 | 4 => {}
                        -1 => {
                            return Ok(1);
                        } // wall
                        3 => return Ok(2),     // stacked blocks
                        _ => return Err(cell), // not possible
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
            self.data
                .push([-1, -1, -1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -1, -1, -1]);
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
        for row in self.data.iter_mut() {
            for cell in row.iter_mut() {
                if *cell == 2 || *cell == 4 {
                    *cell = 0;
                }
            }
        }

        // copy moving block onto board
        let mut y_ = y as usize;

        for row in block.data.iter() {
            let mut x_ = x as usize;
            for col in row.iter() {
                let cell = &mut self.data[y_][x_];
                if *col != 0 && *col != 4 {
                    *cell = *col;
                }
                x_ = x_ + 1;
            }

            y_ = y_ + 1;
        }

        Ok(0)
    }
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
    rotate: u8,
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
        let rotate: u8 = rand::random::<u8>() % 4;
        let block_data: BlockArray = TERMINOS[_type as usize][rotate as usize];
        Block {
            _type: _type,
            data: block_data,
            rotate: rotate,
        }
    }

    pub fn rotate(&mut self) {
        self.rotate = (self.rotate + 1) % 4;

        self.data = TERMINOS[self._type as usize][self.rotate as usize];
    }
}

fn draw_obj(
    objs: &impl Drawable,
    out: &mut termion::raw::RawTerminal<std::io::Stdout>,
    x: u16,
    mut y: u16,
) {
    println!("{}", cursor::Hide);

    let (w, h) = objs.get_size();

    for row in 0..h {
        println!("{}", cursor::Goto(x, y));
        for col in 0..w {
            let data = objs.get_data(row, col);
            match data {
                -5..=-1 => {
                    write!(out, "{}  ", color::Bg(color::White)).unwrap();
                }
                2 => {
                    write!(out, "{}  ", color::Bg(color::LightGreen)).unwrap();
                }
                _ => {
                    write!(out, "{}  ", color::Bg(color::Black)).unwrap();
                }
            }
        }
        y = y + 1;
        write!(out, "\n\r").unwrap();
    }
}

fn draw_score(score: u32, out: &mut termion::raw::RawTerminal<std::io::Stdout>) {
    println!("{}", cursor::Goto(40, 1));
    println!("{}", cursor::Hide);

    write!(out, "{}Score : {}", color::Bg(color::Black), score).unwrap();
}

fn clear_display() {
    println!("{}", clear::All);
}

enum BlockState {
    STACKED,
    DROPPING,
}

fn block_movement(v: &str, board: &mut Board, block: &mut Block) -> Result<BlockState, u8> {
    static mut X: u8 = 6;
    static mut Y: u8 = 0;

    unsafe {
        let mut new_x = X;
        let mut new_y = Y;
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
                if board
                    .check_with_block(X as usize, Y as usize + 1, &block_tmp)
                    .unwrap()
                    == 0
                {
                    new_y = new_y + 1;
                } else {
                    Y = 0;
                    return Ok(BlockState::STACKED);
                }
            }
            "rotate" => {
                block_tmp.rotate();
            }
            "drop" => {}
            _ => {}
        };
        match board
            .check_with_block(new_x as usize, new_y as usize, &block_tmp)
            .unwrap()
        {
            0 => {
                *block = block_tmp;
                board.set_with_block(new_x, new_y, &block).unwrap();
                X = new_x;
                Y = new_y;
            }
            _ => {}
        }
    }

    return Ok(BlockState::DROPPING);
}

fn main() {
    let mut board = Board::new();
    board.init();

    clear_display();

    let mut out = stdout().into_raw_mode().unwrap();

    let break_duration = time::Duration::from_millis(1);

    // message channel
    let (tx, rx) = mpsc::channel();

    let mut dropping_block = Block::new();
    let mut next_block = Block::new();

    let actor = thread::spawn(move || loop {
        if let Ok(ret) = rx.recv() {
            match ret {
                "break" => {
                    break;
                }
                _ => match block_movement(&ret, &mut board, &mut dropping_block).ok() {
                    Some(BlockState::STACKED) => {
                        dropping_block = next_block;
                        next_block = Block::new();
                    }
                    _ => {}
                },
            }
        };

        // check line completion
        let lines_completed = board.check_completion();

        // render game primitives
        draw_obj(&board, &mut out, 1, 1);
        draw_obj(&next_block, &mut out, 45, 10);
        draw_score(lines_completed * 100, &mut out);
    });

    let sender = tx.clone();
    let _input_handler = thread::spawn(move || {
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
    });

    let sender = tx.clone();
    let _ticker = thread::spawn(move || {
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

    actor.join().unwrap();
}
