use crate::constants::CELL_SIZE;
use crate::glam::{IVec2, UVec2, Vec2};
use crate::uvec2;

pub fn grid_coord(pos: Vec2) -> UVec2 {
    uvec2(
        (pos.x / CELL_SIZE).floor() as u32,
        (pos.y / CELL_SIZE).floor() as u32,
    )
}

pub fn hash(grid_coord: UVec2, world_size: Vec2) -> u32 {
    let cells_per_row = (world_size.x / CELL_SIZE).floor() as u32;

    grid_coord.y * cells_per_row + grid_coord.x
}

pub fn neighbours() -> [(i32, i32); 9] {
    [
        (0, 0),
        (-1, 0),
        (1, 0),
        (0, -1),
        (0, 1),
        (1, 1),
        (-1, -1),
        (1, -1),
        (-1, 1),
    ]
}
pub fn find_cell_start(lookups: Vec<(usize, usize)>) {}
