use std::slice;
pub trait Drawable {
    type E;

    fn get_obj(&self) -> slice::Iter<Self::E>;
    fn get_size(&self) -> (usize, usize);
    fn get_data(&self, x: usize, y: usize) -> i8;
}
