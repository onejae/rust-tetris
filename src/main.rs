extern crate termion;

use lazy_static::lazy_static;
use std::io;
use std::io::{stdout, Write};
use std::sync::mpsc;
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

type BoardArray = [[u8; BOARD_SIZE_X]; BOARD_SIZE_Y];
type BlockArray = [[u8; BLOCK_SIZE_X]; BLOCK_SIZE_Y];

lazy_static! {
    static ref TERMINAL_SIZE: (u16, u16) = match termion::terminal_size() {
        Ok(s) => s,
        Err(e) => (80, 100),
    };
}

struct Board {
    data: BoardArray,
    moving_block: Option<Block>,
}

impl Board {
    pub fn init(&mut self) {
        for i in 0..BOARD_SIZE_Y {
            self.data[i] = [1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1];
        }
        self.data[0] = [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1u8, 1, 1];
        self.data[25] = [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1u8, 1, 1];
    }
    pub fn new() -> Board {
        Board {
            data: [[0; BOARD_SIZE_X]; BOARD_SIZE_Y],
            moving_block: Option::None,
        }
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
        let backup_board = self.data;

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

fn draw_object(board: &mut Board, out: &mut termion::raw::RawTerminal<std::io::Stdout>) {
    println!("{}", cursor::Goto(1, 1));
    println!("{}", cursor::Hide);

    let array = board.data;
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

fn clear_display() {
    println!("{}", clear::All);
}

fn main() {
    let mut board: Board = Board::new();
    clear_display();
    board.init();

    // game loop
    let break_duration = time::Duration::from_millis(10);

    let mut sample_block = Block::new(BlockType::J);
    let mut x = 6;
    let mut y = 1;
    board.set_with_block(x, 3, &sample_block).unwrap();

    // message channel
    let (tx, rx) = mpsc::channel();

    // rendering
    thread::spawn(move || loop {
        let mut out = stdout().into_raw_mode().unwrap();
        match rx.try_recv() {
            Ok(v) => {
                let backup_x = x;
                let backup_y = y;
                let mut block = sample_block;
                match v {
                    "movetoleft" => {
                        x = x - 1;
                    }
                    "movetoright" => {
                        x = x + 1;
                    }
                    "movedown" => {
                        y = y + 1;
                    }
                    "rotate" => {
                        block.rotate();
                    }
                    _ => {}
                }
                match board
                    .check_with_block(x as usize, y as usize, &block)
                    .unwrap()
                {
                    0 => {
                        board.set_with_block(x, y, &block).unwrap();
                        sample_block = block;
                    }
                    _ => {
                        x = backup_x;
                        y = backup_y;
                    }
                }
            }
            Err(e) => {}
        };

        draw_object(&mut board, &mut out);
        thread::sleep(break_duration);
    });

    let input_handler = thread::spawn(move || {
        for key in io::stdin().keys() {
            match key.unwrap() {
                Key::Char('q') => break,
                Key::Up => tx.send("rotate").unwrap(),
                Key::Down => tx.send("movedown").unwrap(),
                Key::Left => tx.send("movetoleft").unwrap(),
                Key::Right => tx.send("movetoright").unwrap(),
                Key::Char(' ') => tx.send("shot").unwrap(),
                _ => {}
            }
        }
    });

    let show_host = thread::spawn(move || {
        
    });

    input_handler.join().unwrap();
}
