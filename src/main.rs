extern crate termion;

use lazy_static::lazy_static;
use std::collections::HashMap;
use std::io;
use std::io::{stdout, Write};
use std::sync::mpsc;
use std::time::{SystemTime, UNIX_EPOCH};
use std::{thread, time};
use termion::clear;
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use termion::{color, cursor, raw};

mod drawable;
use drawable::Drawable;
mod block;
use block::{Block, BlockState};
mod board;
use board::Board;

// game board with boundaries

static mut SPEED_FACTOR: u128 = 1000;

fn set_speed_factor(f: u128) {
    unsafe {
        if f >= 0 {
            SPEED_FACTOR = f;
        }
    }
}

fn get_speed_factor() -> u128 {
    unsafe { SPEED_FACTOR }
}

lazy_static! {
    static ref COLOR_TABLE : HashMap<i8, u8> = {
        let mut map = HashMap::new();
        map.insert(-2, 0);          // black
        map.insert(-1, 252);          // red
        map.insert(0, 0);           // black
        map.insert(1, 82);           // green
        map.insert(2, 226);         // yellow
        map.insert(3, 27);          // blue
        map.insert(4, 129);         // violet
        map.insert(5, 197);         // white
        map
    };
    static ref TERMINAL_SIZE: (u16, u16) = termion::terminal_size().unwrap();
}
enum GameScene {
    INTRO,
    GAME,
    END,
}

fn draw_obj(
    objs: &impl Drawable,
    out: &mut raw::RawTerminal<std::io::Stdout>,
    x: isize,
    y: isize,
    mask: i8,
) {
    let (w, h) = objs.get_size();
    let mut y_ = y;

    for row in 0..h {
        let mut x_ = x;
        if y_ >= 1 {
            for col in 0..w {
                let data = objs.get_data(row, col);
                match data {
                    v if v == mask => {}
                    _ => {
                        let c = COLOR_TABLE.get(&data);
                        match c {
                            Some(v) => {
                                println!("{}", cursor::Goto(x_ as u16, y_ as u16));
                                println!("{}  ", color::Bg(color::AnsiValue(*v)));
                            }
                            None => {}
                        }
                    }
                }
                x_ = x_ + 2;
            }
        }
        y_ = y_ + 1;
    }
}

fn draw_gameover(_score: u32) {
    let (x, y) = (TERMINAL_SIZE.0 / 2 - 5, TERMINAL_SIZE.1 / 2);
    println!("{}Game Over", cursor::Goto(x, y));
    // println!("{}Score : {}", cursor::Goto(x, y + 2), score);
    println!("{}Enter : Play again", cursor::Goto(x - 3, 20));
    println!("{}Q : Quit", cursor::Goto(x - 3, 21));
}

fn draw_score(score: u32, out: &mut raw::RawTerminal<std::io::Stdout>) {
    println!("{}", cursor::Goto(40, 1));
    println!("{}", cursor::Hide);

    println!("{}Score : {}", color::Bg(color::Black), score);
}

fn draw_intro(out: &mut raw::RawTerminal<std::io::Stdout>) {
    let (x, y) = (TERMINAL_SIZE.0 / 2 - 10, TERMINAL_SIZE.1 / 2);
    println!("{}RusTetris (ver 0.1.0)", cursor::Goto(x, y));
    println!("{}Enter : Start", cursor::Goto(x + 2, 26));
    println!("{}Q : Quit", cursor::Goto(x + 2, 27));
}

fn draw_basics() {
    let (x, y) = (TERMINAL_SIZE.0, TERMINAL_SIZE.1);
    println!("{}RusTetris (ver 0.1.0)", cursor::Goto(40, 28));
}

fn clear_display() {
    println!("{}{}", color::Bg(color::Black), clear::All);
}

fn block_movement<'a>(v: &str, board: &mut board::Board, block: &'a mut Block) -> &'a Block {
    let mut block_tmp = *block;
    let mut state = BlockState::DROPPING;

    match v {
        "movetoleft" => {
            block_tmp.x = block_tmp.x - 1;
        }
        "movetoright" => {
            block_tmp.x = block_tmp.x + 1;
        }
        "movedown" => {
            block_tmp.y = block_tmp.y + 1;
        }
        "rotate" => {
            block_tmp.rotate();
        }
        "drop" => {
            let mut dropped = false;
            while !dropped {
                block_tmp.y = block_tmp.y + 1;
                match board.check_with_block(&block_tmp).unwrap() {
                    0 => {}
                    _ => dropped = true,
                }
            }

            block.y = block_tmp.y - 1;
            block.state = BlockState::STACKED;

            return block;
        }
        _ => {}
    };

    match board.check_with_block(&block_tmp).unwrap() {
        0 => {
            *block = block_tmp;
        }
        _ => {
            if v == "movedown" {
                state = BlockState::STACKED;
            }
        }
    }

    block.state = state;
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
            Key::Char('\n') => sender.send("go").unwrap(),
            _ => {}
        }
    }

    Ok(0)
}

fn screen_check() -> Result<u8, u8> {
    if TERMINAL_SIZE.0 < 80 || TERMINAL_SIZE.1 < 30 {
        return Err(1);
    }

    return Ok(0);
}

fn main() {
    clear_display();
    if screen_check().err() == Some(1) {
        println!("terminal size must be bigger than (80w x 30h)");
        return;
    }

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

                let ret = rx.recv().ok();
                if ret == Some("break") {
                    break;
                } else if ret == Some("go") {
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
                                board.set_with_block(&dropping_block).unwrap();
                                dropping_block = next_block;
                                next_block = Block::new();

                                // check if droppable
                                if board.check_with_block(&dropping_block).unwrap() != 0 {
                                    game_scene = GameScene::END;
                                } else {
                                    let current = get_speed_factor();
                                    if current >= 100 {
                                        set_speed_factor(current - 7);
                                    }
                                }
                            }
                            _ => {}
                        },
                    }
                };

                // check line completion
                let lines_completed = board.check_completion();
                score = score + lines_completed * 100;
                draw_obj(&board, &mut out, 1, 1, 0);
                draw_obj(&next_block, &mut out, 47, 5, -2);
                draw_obj(
                    &dropping_block,
                    &mut out,
                    1 + (dropping_block.x * 2) as isize,
                    1 + dropping_block.y as isize,
                    0,
                );
                println!("{}{} Next : ", cursor::Goto(39, 6), color::Bg(color::Black));
                draw_score(score, &mut out);
                draw_basics();
            }
            GameScene::END => {
                board.init();
                dropping_block = Block::new();
                next_block = Block::new();

                draw_gameover(score);
                let ret = rx.recv().ok();
                if ret == Some("break") {
                    break;
                } else if ret == Some("go") {
                    score = 0;
                    set_speed_factor(1000);
                    clear_display();
                    game_scene = GameScene::GAME;
                }
            }
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
        // let speed = 1000;

        loop {
            let now = SystemTime::now();

            if now.duration_since(start).unwrap().as_millis() > get_speed_factor() {
                sender.send("movedown").unwrap();
                start = now;
            }
            thread::sleep(break_duration);
        }
    });

    let _scene_manager = thread::spawn(move || {});

    input_handler.join().unwrap();
    actor.join().unwrap();
    clear_display();
}
