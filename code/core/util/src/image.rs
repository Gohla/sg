use std::ffi::CStr;
use std::os::raw::{c_char, c_int, c_void};
use std::slice::{
  from_raw_parts,
  from_raw_parts_mut,
};

use stb_image::stb_image::bindgen::{
  stbi_failure_reason,
  stbi_image_free,
  stbi_load_from_memory,
};
use thiserror::Error;

pub struct ImageData {
  pub dimensions: Dimensions,
  pub storage: Box<dyn Storage>,
}

#[derive(Debug, Error)]
pub enum ImageCreateError {
  #[error("Could not load image data from memory: unknown")]
  Unknown,
  #[error("Could not load image data from memory: {0:?}")]
  Reason(String),
}

impl ImageData {
  pub fn from_encoded(bytes: &[u8], required_components: Option<Components>) -> Result<ImageData, ImageCreateError> {
    let req_comp_num = required_components.map(Components::into);
    let req_comp = req_comp_num.unwrap_or(0) as c_int;
    let mut width = 0 as c_int;
    let mut height = 0 as c_int;
    let mut components = 0 as c_int;
    let ptr = unsafe {
      stbi_load_from_memory(
        bytes.as_ptr(),
        bytes.len() as c_int,
        &mut width,
        &mut height,
        &mut components,
        req_comp,
      )
    };
    if ptr.is_null() {
      let reason: *const c_char = unsafe { stbi_failure_reason() };
      return Err(if let Ok(s) = unsafe { CStr::from_ptr(reason) }.to_str() {
        ImageCreateError::Reason(s.to_string())
      } else {
        ImageCreateError::Unknown
      });
    }
    let dimensions = {
      let width = width as u32;
      let height = height as u32;
      let comp_num = req_comp_num.unwrap_or(components as u8);
      let components = comp_num.into();
      Dimensions { width, height, components }
    };
    let storage = {
      let ptr = ptr as *mut u8;
      let size = dimensions.num_bytes();
      Box::new(DecodedStorage { ptr, size })
    };
    Ok(ImageData { dimensions, storage })
  }

  pub fn from_vec(dimensions: Dimensions, data: Vec<u8>) -> ImageData {
    let storage = Box::new(VecStorage { data });
    ImageData { dimensions, storage }
  }


  pub fn size(&self) -> usize { self.dimensions.num_bytes() }
  pub fn data_slice(&self) -> &[u8] { self.storage.as_slice() }
  pub fn data_slice_mut(&mut self) -> &mut [u8] { self.storage.as_slice_mut() }
  pub fn data_ptr(&self) -> *const u8 { self.storage.as_ptr() }
  pub fn data_ptr_mut(&mut self) -> *mut u8 { self.storage.as_ptr_mut() }


  pub fn subdivide_into_tiles(&self, tile_width: u32, tile_height: u32) -> Vec<ImageData> {
    let dimensions = self.dimensions;
    let width = dimensions.width;
    assert_eq!(width % tile_width, 0, "Image of width {} is not divisible by tile width {}", width, tile_width);
    let height = dimensions.height;
    assert_eq!(height % tile_height, 0, "Image of height {} is not divisible by tile height {}", height, tile_height);
    let tile_dimensions = Dimensions { width: tile_width, height: tile_height, components: dimensions.components };

    let components: u8 = dimensions.components.into();
    let components = components as usize;
    let width = width as usize;
    let tile_width = tile_width as usize;
    let num_tiles_width = width / tile_width;
    let height = height as usize;
    let tile_height = tile_height as usize;
    let num_tiles_height = height / tile_height;

    let data = self.storage.as_slice();
    let mut tiles = {
      let num_tiles = num_tiles_width * num_tiles_height;
      let size_per_tile = tile_width * tile_height * components;
      vec![Vec::<u8>::with_capacity(size_per_tile); num_tiles]
    };

    for y in 0..height {
      for x in 0..width {
        let data_idx = (x + (y * width)) * components;
        let tile_idx = (x / tile_width) + ((y / tile_height) * num_tiles_height);
        //println!("x: {}, y: {}, data_idx: {}, tile_idx: {}", x, y, data_idx, tile_idx);
        for c in 0..components {
          let d: u8 = data[data_idx + c];
          let tile: &mut Vec<u8> = &mut tiles[tile_idx];
          tile.push(d);
        }
      }
    }

    let tiles = tiles;
    tiles
      .into_iter()
      .map(|data| ImageData::from_vec(tile_dimensions, data))
      .collect::<Vec<_>>()
  }
}


#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Components { Components1, Components2, Components3, Components4 }

impl From<Components> for u8 {
  fn from(components: Components) -> Self {
    match components {
      Components::Components1 => 1,
      Components::Components2 => 2,
      Components::Components3 => 3,
      Components::Components4 => 4,
    }
  }
}

impl From<u8> for Components {
  fn from(number: u8) -> Self {
    match number {
      1 => Components::Components1,
      2 => Components::Components2,
      3 => Components::Components3,
      4 => Components::Components4,
      _ => unreachable!("Cannot convert {} to a components enum value", number),
    }
  }
}

#[derive(Clone, Copy, Debug)]
pub struct Dimensions {
  pub width: u32,
  pub height: u32,
  pub components: Components,
}

impl Dimensions {
  pub fn new(width: u32, height: u32, components: Components) -> Dimensions { Dimensions { width, height, components } }
  pub fn num_bytes(&self) -> usize { self.width as usize * self.height as usize * u8::from(self.components) as usize }
  pub fn num_pixels(&self) -> u32 { self.width as u32 * self.height as u32 }
}


pub trait Storage {
  fn as_slice(&self) -> &[u8];
  fn as_slice_mut(&mut self) -> &mut [u8];
  fn as_ptr(&self) -> *const u8;
  fn as_ptr_mut(&mut self) -> *mut u8;
}


pub struct DecodedStorage {
  ptr: *mut u8,
  size: usize,
}

impl Storage for DecodedStorage {
  fn as_slice(&self) -> &[u8] { unsafe { from_raw_parts(self.ptr, self.size) } }
  fn as_slice_mut(&mut self) -> &mut [u8] { unsafe { from_raw_parts_mut(self.ptr, self.size) } }
  fn as_ptr(&self) -> *const u8 { self.ptr as *const u8 }
  fn as_ptr_mut(&mut self) -> *mut u8 { self.ptr }
}

impl Drop for DecodedStorage {
  fn drop(&mut self) { unsafe { stbi_image_free(self.ptr as *mut c_void); } }
}


pub struct VecStorage {
  data: Vec<u8>,
}

impl Storage for VecStorage {
  fn as_slice(&self) -> &[u8] { &self.data }
  fn as_slice_mut(&mut self) -> &mut [u8] { &mut self.data }
  fn as_ptr(&self) -> *const u8 { self.data.as_ptr() }
  fn as_ptr_mut(&mut self) -> *mut u8 { self.data.as_mut_ptr() }
}
