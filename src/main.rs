extern crate termion;

use lazy_static::lazy_static;
use std::io;
use std::io::{stdout, Write};
use std::sync::mpsc;
use std::time::{SystemTime, UNIX_EPOCH};
use std::{thread, time};
use termion::clear;
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use termion::{color, cursor};

// 2-demension array including boundaries
const BOARD_SIZE_X: usize = 16;
const BOARD_SIZE_Y: usize = 26;

const BLOCK_SIZE_X: usize = 4;
const BLOCK_SIZE_Y: usize = 4;

type BoardArray = Vec<[u8; BOARD_SIZE_X]>;
type BlockArray = [[u8; BLOCK_SIZE_X]; BLOCK_SIZE_Y];

lazy_static! {
    static ref TERMINAL_SIZE: (u16, u16) = match termion::terminal_size() {
        Ok(s) => s,
        Err(e) => (80, 100),
    };
}

struct Board {
    data: BoardArray,
}

impl Board {
    pub fn init(&mut self) {
        for i in 0..BOARD_SIZE_Y {
            self.data
                .push([1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1]);
        }
        self.data[25] = [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1];
    }
    pub fn new() -> Board {
        Board { data: Vec::new() }
    }

    // see if possible move
    fn check_with_block(&mut self, x: usize, y: usize, block: &Block) -> Result<u8, u8> {
        let mut y_ = y;
        for row in block.data.iter() {
            let mut x_ = x;
            for col in row.iter() {
                let cell = self.data[y_][x_];
                if (*col == 2) {
                    match cell {
                        0 | 2 => {}
                        1 => return Ok(1),     // wall
                        3 => return Ok(2),     // stacked blocks
                        _ => return Err(cell), // not possible
                    }
                }
                x_ = x_ + 1;
            }
            y_ = y_ + 1;
        }
        Ok(0)
    }

    fn check_completion(&mut self) -> u32 {
        let mut counter = 0;

        // remove completed lines, skip the bottom
        self.data.retain(|&col| {
            let retained: bool = |col: &[u8; BOARD_SIZE_X]| -> bool {
                if col.iter().position(|&r| r == 0) == Option::None {
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
        for i in 0..counter {
            self.data
                .push([1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1]);
        }

        counter
    }
    fn set_with_block(
        &mut self,
        x: u8,
        y: u8,
        block: &Block,
    ) -> Result<&'static str, &'static str> {
        // cell value represents...
        // 0 : empty space
        // 1 : wall
        // 2 : moving
        // 3 : stacked

        // clean up moving cells
        for row in self.data.iter_mut() {
            for cell in row.iter_mut() {
                if *cell == 2 {
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
                if *col != 0 {
                    *cell = *col;
                }
                x_ = x_ + 1;
            }

            y_ = y_ + 1;
        }

        Ok("ok")
    }
}

// #[derive(Default)]
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

#[derive(Copy, Clone)]
struct Block {
    data: BlockArray,
    _type: BlockType,
}

impl Default for Block {
    fn default() -> Block {
        Block::new(BlockType::I)
    }
}

impl Block {
    pub fn new(_type: BlockType) -> Block {
        let mut blockData: BlockArray = [[0u8; BLOCK_SIZE_X]; BLOCK_SIZE_Y];

        match _type {
            BlockType::I => {
                blockData = [
                    [0u8, 0u8, 2u8, 0u8],
                    [0u8, 0u8, 2u8, 0u8],
                    [0u8, 0u8, 2u8, 0u8],
                    [0u8, 0u8, 2u8, 0u8],
                ]
            }
            BlockType::J => {
                blockData = [
                    [0u8, 0u8, 0u8, 0u8],
                    [0u8, 0u8, 2u8, 0u8],
                    [0u8, 0u8, 2u8, 0u8],
                    [0u8, 2u8, 2u8, 0u8],
                ]
            }
            BlockType::L => {
                blockData = [
                    [0u8, 0u8, 0u8, 0u8],
                    [0u8, 0u8, 2u8, 0u8],
                    [0u8, 0u8, 2u8, 0u8],
                    [0u8, 0u8, 2u8, 2u8],
                ]
            }
            BlockType::O => {
                blockData = [
                    [0u8, 0u8, 0u8, 0u8],
                    [0u8, 2u8, 2u8, 0u8],
                    [0u8, 2u8, 2u8, 0u8],
                    [0u8, 0u8, 0u8, 0u8],
                ]
            }
            BlockType::S => {
                blockData = [
                    [0u8, 0u8, 0u8, 0u8],
                    [0u8, 0u8, 2u8, 2u8],
                    [0u8, 2u8, 2u8, 0u8],
                    [0u8, 0u8, 0u8, 0u8],
                ]
            }
            BlockType::Z => {
                blockData = [
                    [0u8, 0u8, 0u8, 0u8],
                    [2u8, 2u8, 0u8, 0u8],
                    [0u8, 2u8, 2u8, 0u8],
                    [0u8, 0u8, 0u8, 0u8],
                ]
            }
            BlockType::T => {
                blockData = [
                    [0u8, 0u8, 0u8, 0u8],
                    [0u8, 2u8, 2u8, 2u8],
                    [0u8, 0u8, 2u8, 0u8],
                    [0u8, 0u8, 0u8, 0u8],
                ]
            }
        }
        Block {
            _type: _type,
            data: blockData,
        }
    }

    pub fn rotate(&mut self) {
        let mut ret: BlockArray = [[0u8; BLOCK_SIZE_X]; BLOCK_SIZE_Y];
        for i in (0..BLOCK_SIZE_Y) {
            for j in (0..BLOCK_SIZE_X) {
                ret[j][BLOCK_SIZE_X - i - 1] = self.data[i][j];
            }
        }

        self.data = ret;
    }
}

fn draw_board(board: &mut Board, out: &mut termion::raw::RawTerminal<std::io::Stdout>) {
    println!("{}", cursor::Goto(1, 1));
    println!("{}", cursor::Hide);

    let array = &board.data;
    for row in array.iter() {
        for col in row.iter() {
            match *col {
                1 => {
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
        write!(out, "\n\r").unwrap();
    }
}

fn draw_score(score: u32, out: &mut termion::raw::RawTerminal<std::io::Stdout>) {
    println!("{}", cursor::Goto(40, 1));
    println!("{}", cursor::Hide);

    println!("{}Score : {}", color::Bg(color::Black), score);
}

fn clear_display() {
    println!("{}", clear::All);
}

fn block_movement(rx: &mpsc::Receiver<&str>, board: &mut Board, block: &Block) {
    static mut x: u8 = 6;
    static mut y: u8 = 0;

    unsafe {
        match rx.try_recv() {
            Ok(v) => {
                let mut new_x = x;
                let mut new_y = y;
                let mut block_tmp = *block;
                match v {
                    "movetoleft" => {
                        new_x = new_x - 1;
                    }
                    "movetoright" => {
                        new_x = new_x + 1;
                    }
                    "movedown" => {
                        new_y = new_y + 1;
                    }
                    "rotate" => {
                        block_tmp.rotate();
                    }
                    "break" => {
                        // break;
                    }
                    _ => {}
                }
                match board
                    .check_with_block(new_x as usize, new_y as usize, &block)
                    .unwrap()
                {
                    0 => {
                        board.set_with_block(x, y, &block).unwrap();
                        x = new_x;
                        y = new_y;
                    }
                    _ => {}
                }
            }
            Err(e) => {}
        };
    }
}

fn main() {
    let mut board: Board = Board::new();
    board.init();

    clear_display();

    let mut out = stdout().into_raw_mode().unwrap();
    // game loop
    let break_duration = time::Duration::from_millis(10);

    let mut sample_block = Block::new(BlockType::J);
    let mut x = 6;
    let mut y = 1;
    board.set_with_block(x, 3, &sample_block).unwrap();

    // message channel
    let (tx, rx) = mpsc::channel();

    let actor = thread::spawn(move || loop {
        // handle input
        block_movement(&rx, &mut board, &sample_block);

        // check line completion
        let lines_completed = board.check_completion();

        // render game primitives
        draw_board(&mut board, &mut out);
        draw_score(lines_completed * 100, &mut out);

        thread::sleep(break_duration);
    });

    let sender = tx.clone();
    let input_handler = thread::spawn(move || {
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
                Key::Char(' ') => sender.send("shot").unwrap(),
                _ => {}
            }
        }
    });

    let sender = tx.clone();
    let ticker = thread::spawn(move || {
        let mut start = SystemTime::now();
        let speed = 1000;

        loop {
            let now = SystemTime::now();

            if now.duration_since(start).unwrap().as_millis() > speed {
                sender.send("movedown").unwrap();
                start = now;
            }
        }
    });

    input_handler.join().unwrap();
    actor.join().unwrap();
}
