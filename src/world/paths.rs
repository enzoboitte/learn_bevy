use bevy::prelude::*;

use crate::world::tiled::{MAP_HEIGHT, MAP_WIDTH, TILE_SIZE};

pub const EMPTY: i32 = -1;
pub const PATH: i32 = 1;

#[derive(Resource)]
pub struct PathMap {
    pub tiles: [[i32; MAP_WIDTH]; MAP_HEIGHT],
}

impl Default for PathMap {
    fn default() -> Self {
        Self::new()
    }
}

impl PathMap {
    pub const fn new() -> Self {
        Self {
            tiles: [[EMPTY; MAP_WIDTH]; MAP_HEIGHT],
        }
    }

    pub fn is_inside(x: usize, y: usize) -> bool {
        x < MAP_WIDTH && y < MAP_HEIGHT
    }

    pub fn get(&self, x: usize, y: usize) -> Option<i32> {
        Self::is_inside(x, y).then_some(self.tiles[y][x])
    }

    pub fn set(&mut self, x: usize, y: usize, value: i32) -> bool {
        if !Self::is_inside(x, y) {
            return false;
        }

        self.tiles[y][x] = value;
        true
    }

    pub fn set_path(&mut self, x: usize, y: usize) -> bool {
        self.set(x, y, PATH)
    }

    #[allow(dead_code)]
    pub fn remove_path(&mut self, x: usize, y: usize) -> bool {
        self.set(x, y, EMPTY)
    }

    pub fn has_path(&self, x: usize, y: usize) -> bool {
        self.get(x, y) == Some(PATH)
    }

    pub fn path_mask(&self, x: usize, y: usize) -> u8 {
        if !self.has_path(x, y) {
            return 0;
        }

        let mut mask = 0;

        if y > 0 && self.has_path(x, y - 1) {
            mask |= 1;
        }
        if x + 1 < MAP_WIDTH && self.has_path(x + 1, y) {
            mask |= 2;
        }
        if y + 1 < MAP_HEIGHT && self.has_path(x, y + 1) {
            mask |= 4;
        }
        if x > 0 && self.has_path(x - 1, y) {
            mask |= 8;
        }

        mask
    }

    pub fn tile_index(&self, x: usize, y: usize) -> Option<usize> {
        self.has_path(x, y)
            .then_some(PATH_TILE_BY_MASK[self.path_mask(x, y) as usize])
    }

    pub fn affected_tiles(x: usize, y: usize) -> [(usize, usize); 5] {
        [
            (x, y),
            (x, y.wrapping_sub(1)),
            (x.saturating_add(1), y),
            (x, y.saturating_add(1)),
            (x.wrapping_sub(1), y),
        ]
    }
}

#[derive(Component)]
pub struct PathTile {
    pub x: usize,
    pub y: usize,
}

pub const PATH_TILE_BY_MASK: [usize; 16] = [
    36, // 0: seul
    25, // 1: haut
    33, // 2: droite
    22, // 3: haut + droite
    3,  // 4: bas
    14, // 5: haut + bas
    0,  // 6: droite + bas
    11, // 7: haut + droite + bas
    35, // 8: gauche
    24, // 9: haut + gauche
    34, // 10: gauche + droite
    23, // 11: haut + gauche + droite
    2,  // 12: bas + gauche
    13, // 13: haut + bas + gauche
    1,  // 14: droite + bas + gauche
    55, // 15: plan plein
];

/*pub fn path_mask(map: &PathMap, x: usize, y: usize) -> u8 {
    map.path_mask(x, y)
}*/

pub fn world_to_tile(world_position: Vec2) -> Option<(usize, usize)> {
    let x = (world_position.x / TILE_SIZE + MAP_WIDTH as f32 / 2.0).floor();
    let y = (MAP_HEIGHT as f32 / 2.0 - world_position.y / TILE_SIZE).floor();

    if x < 0.0 || y < 0.0 {
        return None;
    }

    let tile = (x as usize, y as usize);
    PathMap::is_inside(tile.0, tile.1).then_some(tile)
}