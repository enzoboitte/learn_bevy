use bevy::prelude::*;
use bevy_asset_loader::asset_collection::*;

#[derive(AssetCollection, Resource)]
pub struct GameAssets {
    #[asset(texture_atlas_layout(tile_size_x = 48, tile_size_y = 48, columns = 10, rows = 4))]
    pub player: Handle<TextureAtlasLayout>,

    #[asset(path = "characters/player.png")]
    pub player_texture: Handle<Image>,
}