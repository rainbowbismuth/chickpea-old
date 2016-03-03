#![feature(custom_derive, plugin)]
#![plugin(serde_macros)]

extern crate image;
extern crate serde;
extern crate serde_json;

use std::fs::File;
use std::collections::HashMap;
use std::path::Path;
use image::{DynamicImage, GenericImage, ImageFormat};

#[derive(Clone, Serialize, Deserialize)]
pub struct TileSource {
    pub identifier: String,
    pub image_path: String,
}

pub const TILES_IN_FLOOR_SET: u32 = 16;

#[derive(Clone, Serialize, Deserialize)]
pub struct FloorSource {
    pub tile_source: String,
    pub floor_sets: Vec<FloorSetSource>,
}

impl FloorSource {
    pub fn num_tiles(&self) -> u32 {
        self.floor_sets.len() as u32 * TILES_IN_FLOOR_SET
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct FloorSetSource {
    pub identifier: String,
    pub location: (u32, u32),
}

pub const TILES_IN_WALL_SET: u32 = 13;

#[derive(Clone, Serialize, Deserialize)]
pub struct WallSource {
    pub tile_source: String,
    pub wall_sets: Vec<WallSetSource>,
}

impl WallSource {
    pub fn num_tiles(&self) -> u32 {
        self.wall_sets.len() as u32 * TILES_IN_WALL_SET
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct WallSetSource {
    pub identifier: String,
    pub location: (u32, u32),
}

#[derive(Clone, Serialize, Deserialize)]
pub struct TileSetModule {
    pub identifier: String,
    pub tile_size: (u32, u32),
    pub out_tile_set_path: String,
    pub out_image_path: String,
    pub tile_sources: Vec<TileSource>,
    pub floor_sources: Vec<FloorSource>,
    pub wall_sources: Vec<WallSource>,
}

impl TileSetModule {
    pub fn num_tiles(&self) -> u32 {
        let mut sum = 0;
        for floor_source in &self.floor_sources {
            sum += floor_source.num_tiles();
        }
        for wall_source in &self.wall_sources {
            sum += wall_source.num_tiles();
        }
        sum
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct CompiledTileSet {
    pub identifier: String,
    pub tile_size: (u32, u32),
    pub image_path: String,
    pub floor_sets: HashMap<String, CompiledFloorSet>,
    pub wall_sets: HashMap<String, CompiledWallSet>,
}

#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct CompiledFloorSet {
    pub numpad: [(u32, u32); 9],
    pub top_bottom: [(u32, u32); 3],
    pub left_right: [(u32, u32); 3],
    pub closed_center: (u32, u32),
}

#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct CompiledWallSet {
    pub circle: [(u32, u32); 6],
    pub center_point: (u32, u32),
    pub wall: (u32, u32),
    pub cross: [(u32, u32); 5],
}

#[derive(Debug)]
pub enum Error {
    Msg(&'static str),
    ImageError(image::ImageError),
    JsonError(serde_json::error::Error),
    IOError(std::io::Error),
}

impl From<image::ImageError> for Error {
    fn from(err: image::ImageError) -> Self {
        Error::ImageError(err)
    }
}

impl From<serde_json::error::Error> for Error {
    fn from(err: serde_json::error::Error) -> Self {
        Error::JsonError(err)
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::IOError(err)
    }
}

pub type TileSetResult<T> = Result<T, Error>;

struct TileSetCursor {
    img: DynamicImage,
    loc: (u32, u32),
    tile_size: (u32, u32),
}

impl TileSetCursor {
    fn new(dimensions: (u32, u32), tile_size: (u32, u32)) -> TileSetCursor {
        TileSetCursor {
            img: DynamicImage::new_rgba8(dimensions.0, dimensions.1),
            loc: (0, 0),
            tile_size: tile_size,
        }
    }

    fn add_tile(&mut self,
                from: &mut DynamicImage,
                tile_coordinates: (u32, u32))
                -> TileSetResult<(u32, u32)> {
        let (width, _) = self.img.dimensions();
        let x = tile_coordinates.0 * self.tile_size.0;
        let y = tile_coordinates.1 * self.tile_size.1;
        let sub = from.sub_image(x, y, self.tile_size.0, self.tile_size.1);

        let ok = self.img.copy_from(&sub, self.loc.0, self.loc.1);

        self.loc.0 += self.tile_size.0;
        if self.loc.0 + self.tile_size.0 > width {
            self.loc.0 = 0;
            self.loc.1 += self.tile_size.1;
        }

        match ok {
            true => Ok((self.loc.0, self.loc.1)),
            false => {
                println!("{:?}", width);
                println!("{:?}", self.loc);
                Err(Error::Msg("couldn't fit tile into image"))
            }
        }

    }
}

pub fn compile_tile_set_module(module_src: &str) -> TileSetResult<()> {
    let module: TileSetModule = {
        let mut reader = try!(File::open(module_src));
        try!(serde_json::de::from_reader(&mut reader))
    };

    let mut images = try!(load_tile_sources(&module.tile_sources));

    let total_tiles = module.num_tiles();

    let tile_set_image_size = {
        let root = (total_tiles as f64).sqrt().floor() as u32 + 1;
        assert!(root * root > total_tiles);
        (root * module.tile_size.0, root * module.tile_size.1)
    };

    let mut tile_set_cursor = TileSetCursor::new(tile_set_image_size, module.tile_size);

    let floor_sets = try!(compile_floor_sets(&module.floor_sources,
                                             &mut images,
                                             &mut tile_set_cursor));
    let wall_sets = try!(compile_wall_sets(&module.wall_sources,
                                           &mut images,
                                           &mut tile_set_cursor));

    let compiled_tile_set = CompiledTileSet {
        identifier: module.identifier.clone(),
        tile_size: module.tile_size,
        image_path: module.out_image_path.clone(),
        floor_sets: floor_sets,
        wall_sets: wall_sets,
    };

    {
        let mut buffer = try!(File::create(&module.out_tile_set_path));
        try!(serde_json::ser::to_writer(&mut buffer, &compiled_tile_set));
    }

    {
        let mut buffer = try!(File::create(&module.out_image_path));
        try!(tile_set_cursor.img.save(&mut buffer, ImageFormat::PNG));
    }

    Ok(())
}

fn load_tile_sources<'a>(tile_sources: &'a [TileSource])
                         -> TileSetResult<HashMap<&'a str, DynamicImage>> {
    let mut hmap = HashMap::new();
    for tile_source in tile_sources {
        let image = try!(image::open(Path::new(&tile_source.image_path)));
        hmap.insert(tile_source.identifier.as_ref(), image);
    }
    Ok(hmap)
}

fn compile_wall_sets(wall_sources: &[WallSource],
                     images: &mut HashMap<&str, DynamicImage>,
                     cursor: &mut TileSetCursor)
                     -> TileSetResult<HashMap<String, CompiledWallSet>> {
    let mut hmap = HashMap::new();
    for wall_source in wall_sources {
        let mut image = try!(images.get_mut(&wall_source.tile_source[..])
                                   .ok_or(Error::Msg("missing tile_source")));
        for wall_set in &wall_source.wall_sets {
            let (x, y) = wall_set.location;

            let c3 = try!(cursor.add_tile(&mut image, (x, y)));
            let c4 = try!(cursor.add_tile(&mut image, (x + 1, y)));
            let c5 = try!(cursor.add_tile(&mut image, (x + 2, y)));
            let w = try!(cursor.add_tile(&mut image, (x + 3, y)));
            let x1 = try!(cursor.add_tile(&mut image, (x + 4, y)));
            let c2 = try!(cursor.add_tile(&mut image, (x, y + 1)));
            let cp = try!(cursor.add_tile(&mut image, (x + 1, y + 1)));
            let x2 = try!(cursor.add_tile(&mut image, (x + 3, y + 1)));
            let x3 = try!(cursor.add_tile(&mut image, (x + 4, y + 1)));
            let x4 = try!(cursor.add_tile(&mut image, (x + 5, y + 1)));
            let c1 = try!(cursor.add_tile(&mut image, (x, y + 2)));
            let c6 = try!(cursor.add_tile(&mut image, (x + 2, y + 2)));
            let x5 = try!(cursor.add_tile(&mut image, (x + 4, y + 2)));

            let compiled_wall_set = CompiledWallSet {
                circle: [c1, c2, c3, c4, c5, c6],
                center_point: cp,
                wall: w,
                cross: [x1, x2, x3, x4, x5],
            };
            hmap.insert(wall_set.identifier.clone(), compiled_wall_set);
        }
    }
    Ok(hmap)
}

fn compile_floor_sets(floor_sources: &[FloorSource],
                      images: &mut HashMap<&str, DynamicImage>,
                      cursor: &mut TileSetCursor)
                      -> TileSetResult<HashMap<String, CompiledFloorSet>> {
    let mut hmap = HashMap::new();
    for floor_source in floor_sources {
        let mut image = try!(images.get_mut(&floor_source.tile_source[..])
                                   .ok_or(Error::Msg("missing tile_source")));
        for floor_set in &floor_source.floor_sets {
            let (x, y) = floor_set.location;

            let n7 = try!(cursor.add_tile(&mut image, (x, y)));
            let n8 = try!(cursor.add_tile(&mut image, (x + 1, y)));
            let n9 = try!(cursor.add_tile(&mut image, (x + 2, y)));
            let tb_top = try!(cursor.add_tile(&mut image, (x + 3, y)));
            let closed = try!(cursor.add_tile(&mut image, (x + 5, y)));
            let n4 = try!(cursor.add_tile(&mut image, (x, y + 1)));
            let n5 = try!(cursor.add_tile(&mut image, (x + 1, y + 1)));
            let n6 = try!(cursor.add_tile(&mut image, (x + 2, y + 1)));
            let tb_mid = try!(cursor.add_tile(&mut image, (x + 3, y + 1)));
            let lr_left = try!(cursor.add_tile(&mut image, (x + 4, y + 1)));
            let lr_mid = try!(cursor.add_tile(&mut image, (x + 5, y + 1)));
            let lr_right = try!(cursor.add_tile(&mut image, (x + 6, y + 1)));
            let n1 = try!(cursor.add_tile(&mut image, (x, y + 2)));
            let n2 = try!(cursor.add_tile(&mut image, (x + 1, y + 2)));
            let n3 = try!(cursor.add_tile(&mut image, (x + 2, y + 2)));
            let tb_bot = try!(cursor.add_tile(&mut image, (x + 3, y + 2)));

            let compiled_floor_set = CompiledFloorSet {
                numpad: [n1, n2, n3, n4, n5, n6, n7, n8, n9],
                top_bottom: [tb_top, tb_mid, tb_bot],
                left_right: [lr_left, lr_mid, lr_right],
                closed_center: closed,
            };
            hmap.insert(floor_set.identifier.clone(), compiled_floor_set);
        }
    }
    Ok(hmap)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        compile_tile_set_module("test_data/tile_set.json").expect("compilation failed");
    }
}
