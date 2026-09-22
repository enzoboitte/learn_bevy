use bevy::{asset, prelude::*};
use bevy_asset_loader::asset_collection::*;

#[derive(AssetCollection, Resource)]
pub struct GameAssets {
    #[asset(texture_atlas_layout(tile_size_x = 48, tile_size_y = 48, columns = 10, rows = 4))]
    pub player: Handle<TextureAtlasLayout>,
    #[asset(path = "characters/player.png")]
    pub player_texture: Handle<Image>,


    // water
    #[asset(texture_atlas_layout(tile_size_x = 16, tile_size_y = 16, columns = 4, rows = 1))]
    pub water_layout: Handle<TextureAtlasLayout>,
    #[asset(path = "tilesets/water.png")]
    pub water_texture: Handle<Image>,

    // grass
    #[asset(texture_atlas_layout(tile_size_x = 16, tile_size_y = 16, columns = 11, rows = 7))]
    pub grass_layout: Handle<TextureAtlasLayout>,
    #[asset(path = "tilesets/grass.png")]
    pub grass_texture: Handle<Image>,

    // biome
    #[asset(texture_atlas_layout(tile_size_x = 32, tile_size_y = 32, columns = 4, rows = 2, offset_x = 16))]
    pub big_biome_layout: Handle<TextureAtlasLayout>,
    #[asset(texture_atlas_layout(tile_size_x = 16, tile_size_y = 16, columns = 9, rows = 5))]
    pub small_biome_layout: Handle<TextureAtlasLayout>,
    #[asset(path = "tilesets/biome.png")]
    pub big_biome_texture: Handle<Image>,
}