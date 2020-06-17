use crate::drawable::Drawable;
use lazy_static::lazy_static;
use rand::{
    distributions::{Distribution, Standard},
    Rng,
};
use std::slice;

pub const BLOCK_SIZE_X: usize = 4;
pub const BLOCK_SIZE_Y: usize = 4;

type BlockArray = [[i8; BLOCK_SIZE_X]; BLOCK_SIZE_Y];
lazy_static! {
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
pub enum BlockState {
    STACKED,
    DROPPING,
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
pub struct Block {
    pub data: BlockArray,
    _type: BlockType,
    rotation: u8,
    pub x: i8,
    pub y: i8,
    pub state: BlockState,
    pub color: u8,
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
        let color = rand::random::<u8>() % 5 + 1;

        let block_data = Block::get_block_with_color(_type as usize, rotation as usize, color);
        Block {
            _type: _type,
            data: block_data,
            rotation: rotation,
            state: BlockState::DROPPING,
            x: 6,
            y: 0 - Block::get_offset(_type as usize, rotation as usize) as i8 - 1,
            color: color,
        }
    }

    fn get_offset(_type: usize, rotation: usize) -> usize {
        let block_data: BlockArray = TERMINOS[_type as usize][rotation as usize];
        let mut offset = 0;
        for row in block_data.iter() {
            if *row == [0i8; BLOCK_SIZE_X] {
                offset = offset + 1;
            } else {
                return offset;
            }
        }
        return offset;
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
    }
}
