use crate::block::Block;
use crate::drawable::Drawable;
use std::slice;

const BOARD_SIZE_X: usize = 16;
const BOARD_SIZE_Y: usize = 27;

const ROWS: usize = 24;
const COLS: usize = 10;

type BoardArray = Vec<[i8; BOARD_SIZE_X]>;
pub struct Board {
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
        self.data.clear();
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
    // pub fn check_with_block(&mut self, x: isize, y: isize, block: &Block) -> Result<u8, i8> {
    pub fn check_with_block(&mut self, block: &Block) -> Result<u8, i8> {
        let mut y_ = block.y;
        for row in block.data.iter() {
            let mut x_ = block.x;
            for col in row.iter() {
                if y_ >= 0 && x_ >= 0 {
                    let cell = self.data[y_ as usize][x_ as usize];
                    if *col != 0 {
                        match cell {
                            -2 => {}
                            -1..=6 => return Ok(1), // wall
                            _ => return Err(cell),  // not possible
                        }
                    }
                }
                x_ = x_ + 1;
            }
            y_ = y_ + 1;
        }
        return Ok(0);
    }

    pub fn check_completion(&mut self) -> u32 {
        let mut counter = 0;

        // remove completed lines, skip the bottom
        self.data.retain(|&col| {
            let retained: bool = |col: &[i8; BOARD_SIZE_X]| -> bool {
                if col.iter().filter(|&r| *r > 0).count() == COLS {
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
            self.data.insert(
                0,
                [
                    -1, -1, -1, -2, -2, -2, -2, -2, -2, -2, -2, -2, -2, -1, -1, -1,
                ],
            );
        }

        counter
    }
    pub fn set_with_block(&mut self, block: &Block, color: i8) -> Result<u8, &'static str> {
        // copy moving block onto board
        let mut y_ = block.y;

        for row in block.data.iter() {
            if y_ >= 0 {
                let mut x_ = block.x;
                for col in row.iter() {
                    let cell = &mut self.data[y_ as usize][x_ as usize];
                    if *col != 0 {
                        *cell = color;
                    }
                    x_ = x_ + 1;
                }
            }

            y_ = y_ + 1;
        }

        Ok(0)
    }
}
