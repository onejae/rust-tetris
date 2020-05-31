extern crate termion;

use lazy_static::lazy_static;
use std::io;
use std::io::{stdin, stdout, Write};
use std::{thread, time};
use termion::clear;
use termion::raw::IntoRawMode;
use termion::*;
use termion::{color, cursor, style};

// 2-demension array including boundaries
const boardSizeX: usize = 12;
const boardSizeY: usize = 26;

const blockSizeX: usize = 4;
const blockSizeY: usize = 4;

type BoardArray = [[u8; boardSizeX]; boardSizeY];
type BlockArray = [[u8; blockSizeX]; blockSizeY];

lazy_static! {
    static ref terminalSize: (u16, u16) = match termion::terminal_size() {
        Ok(s) => s,
        Err(e) => (80, 100),
    };
}

struct Board {
    data: BoardArray,
    moving_block : Option<Block>
}

impl Board {
    pub fn init(&mut self) {
        for i in 0..boardSizeY {
            self.data[i] = [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1];
        }
        self.data[0] = [1, 1, 1, 1, 1, 1, 1, 1, 1, 1u8, 1, 1];
        self.data[25] = [1, 1, 1, 1, 1, 1, 1, 1, 1, 1u8, 1, 1];
    }
    pub fn new() -> Board {
        Board {
            data: [[0; boardSizeX]; boardSizeY], moving_block: Option::None
        }
    }
    pub fn as_array(&mut self) -> &BoardArray {
        &self.data
    }
    pub fn add_block(&mut self, block: &Block) {
        // self.moving_block = Some(block);

        Board::refresh_data_with_block(self, 10, 0, block);
    }
    fn refresh_data_with_block(&mut self, x:u8, y:u8, block:&Block) {

    }
}

// #[derive(Default)]
enum BlockType {
    I,
    J,
    L,
    O,
    S,
    Z,
    T,
}

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
        let mut blockData: BlockArray = [[0u8; 4]; 4];

        match _type {
            BlockType::I => {
                blockData = [
                    [0u8, 0u8, 1u8, 0u8],
                    [0u8, 0u8, 1u8, 0u8],
                    [0u8, 0u8, 1u8, 0u8],
                    [0u8, 0u8, 1u8, 0u8],
                ]
            }
            BlockType::J => {
                blockData = [
                    [0u8, 0u8, 0u8, 0u8],
                    [0u8, 0u8, 1u8, 0u8],
                    [0u8, 0u8, 1u8, 0u8],
                    [0u8, 1u8, 1u8, 0u8],
                ]
            }
            BlockType::L => {
                blockData = [
                    [0u8, 0u8, 0u8, 0u8],
                    [0u8, 0u8, 1u8, 0u8],
                    [0u8, 0u8, 1u8, 0u8],
                    [0u8, 0u8, 1u8, 0u8],
                ]
            }
            BlockType::O => {
                blockData = [
                    [0u8, 0u8, 0u8, 0u8],
                    [0u8, 1u8, 1u8, 0u8],
                    [0u8, 1u8, 1u8, 0u8],
                    [0u8, 0u8, 0u8, 0u8],
                ]
            }
            BlockType::S => {
                blockData = [
                    [0u8, 0u8, 0u8, 0u8],
                    [0u8, 0u8, 1u8, 1u8],
                    [0u8, 1u8, 1u8, 0u8],
                    [0u8, 0u8, 0u8, 0u8],
                ]
            }
            BlockType::Z => {
                blockData = [
                    [0u8, 0u8, 0u8, 0u8],
                    [1u8, 1u8, 0u8, 0u8],
                    [0u8, 1u8, 1u8, 0u8],
                    [0u8, 0u8, 0u8, 0u8],
                ]
            }
            BlockType::T => {
                blockData = [
                    [1u8, 1u8, 1u8, 0u8],
                    [0u8, 1u8, 0u8, 0u8],
                    [0u8, 0u8, 0u8, 0u8],
                    [0u8, 0u8, 0u8, 0u8],
                ]
            }
        }
        Block {
            _type: _type,
            data: blockData,
        }
    }
}
// static mut boardLock: Global<Board> = Global::new();

// fn init() {
// println!("{}", clear::All);
// unsafe {
// let board = &mut boardLock.lock_mut().unwrap();
// board.init();
// println!(
// "{}Rust tetris",
// termion::cursor::Goto(terminalSize.0 / 2, 1)
// );
// }
// }

fn draw_game_board(board: &mut Board) {
    println!("{}", cursor::Goto(1, 1));
    // let board = gameBoard.board;
    let mut stdout = stdout().into_raw_mode().unwrap();

    let array = board.as_array();
    let red = 0;
    for row in array.iter() {
        for col in row.iter() {
            if *col == 1 {
                write!(stdout, "{}  ", color::Bg(color::White)).unwrap();
            } else {
                write!(stdout, "{}  ", color::Bg(color::Black)).unwrap();
            }
        }
        write!(stdout, "\n\r").unwrap();
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

    let sampleBlock = Block::new(BlockType::L);
    board.add_block(&sampleBlock);

    loop {
        draw_game_board(&mut board);
        thread::sleep(break_duration);
    }
}
