use anyhow::Result;

use gfx::texture_def::{TextureDefBuilder, TextureIdx};
use util::image::{Components, ImageData};

pub struct GameDef {
  pub grid_tile_textures: Vec<TextureIdx>,
}

impl GameDef {
  pub fn new() -> Result<(GameDef, TextureDefBuilder)> {
    let mut texture_def_builder = TextureDefBuilder::new();
    let tex1 = texture_def_builder.add_texture(ImageData::from_encoded(include_bytes!("../../../../asset/wall_tile/dark.png"), Some(Components::Components4))?);
    let tex2 = texture_def_builder.add_texture(ImageData::from_encoded(include_bytes!("../../../../asset/wall_tile/light.png"), Some(Components::Components4))?);
    let tex3 = texture_def_builder.add_texture(ImageData::from_encoded(include_bytes!("../../../../asset/wall_tile/green.png"), Some(Components::Components4))?);
    let game_def = GameDef { grid_tile_textures: vec![tex1, tex2, tex3] };
    Ok((game_def, texture_def_builder))
  }
}
