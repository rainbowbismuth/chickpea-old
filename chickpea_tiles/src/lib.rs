// chickpea, A small tile-based game project
// Copyright (C) 2016 Emily A. Bellows
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <http://www.gnu.org/licenses/>.

#![feature(custom_derive, plugin)]
#![plugin(serde_macros)]

extern crate image;
extern crate serde;
extern crate serde_json;

use std::fs::File;
use std::collections::HashMap;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use image::{DynamicImage, GenericImage, ImageFormat};

#[derive(Clone, Serialize, Deserialize)]
pub struct TileSource {
    pub image_path: String,
    pub tile_size: [usize; 2],
}

pub type OutputTileFormat = BTreeMap<String, usize>;

#[derive(Clone, Serialize, Deserialize)]
pub struct InputTileFormat {
    pub fmt: String,
    pub parts: BTreeMap<String, Vec<[usize; 2]>>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct TileSetSource {
    pub tile_size: [usize; 2],
    pub groups: Vec<TileSetSourceGroup>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct TileSetSourceGroup {
    pub from: String,
    pub fmt: String,
    pub items: Vec<TileSetSourceItem>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct TileSetSourceItem {
    pub id: String,
    pub loc: [usize; 2],
}

pub fn num_tiles(fmt: &OutputTileFormat) -> usize {
    fmt.values().fold(0, |x, y| x + y)
}

#[derive(Clone, Serialize, Deserialize)]
pub struct TileSet {
    pub tile_size: [usize; 2],
    pub image_path: String,
    pub fmts: HashMap<String, TileSetItems>,
}

pub type TileSetItems = HashMap<String, Vec<[usize; 2]>>;

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
    loc: [usize; 2],
    tile_size: [usize; 2],
}

impl TileSetCursor {
    fn new(dimensions: [usize; 2], tile_size: [usize; 2]) -> TileSetCursor {
        TileSetCursor {
            img: DynamicImage::new_rgba8(dimensions[0] as u32, dimensions[1] as u32),
            loc: [0, 0],
            tile_size: tile_size,
        }
    }

    fn add_tile(&mut self,
                from: &mut DynamicImage,
                tile_coordinates: [usize; 2])
                -> TileSetResult<[usize; 2]> {
        let width = self.img.dimensions().0 as usize;
        let x = tile_coordinates[0] * self.tile_size[0];
        let y = tile_coordinates[1] * self.tile_size[1];
        let sub = from.sub_image(x as u32, y as u32, self.tile_size[0] as u32, self.tile_size[1] as u32);

        let ok = self.img.copy_from(&sub, self.loc[0] as u32, self.loc[1] as u32);

        self.loc[0] += self.tile_size[0];
        if self.loc[0] + self.tile_size[0] > width {
            self.loc[0] = 0;
            self.loc[1] += self.tile_size[1];
        }

        match ok {
            true => Ok([self.loc[0], self.loc[1]]),
            false => {
                Err(Error::Msg("couldn't fit tile into image"))
            }
        }

    }
}

fn load<T: serde::Deserialize>(mut path: PathBuf) -> TileSetResult<T> {
    path.set_extension("json");
    let reader = try!(File::open(path));
    let t: T = try!(serde_json::de::from_reader(reader));
    Ok(t)
}

pub fn compile_tile_set(src_folder: &Path,
                        tile_set_source_path: &Path,
                        target: &Path,
                        tile_set_target_path: &Path) -> TileSetResult<()> {
    let tss: TileSetSource = try!(load(src_folder.join(tile_set_source_path)));
    let mut fmts = HashMap::<String, TileSetItems>::new();
    let mut total_tiles = 0;

    for group in &tss.groups {
        let from: TileSource = try!(load(src_folder.join(&group.from)));
        let ifmt: InputTileFormat = try!(load(src_folder.join(&group.fmt)));
        let ofmt: OutputTileFormat = try!(load(src_folder.join(&ifmt.fmt)));

        if tss.tile_size != from.tile_size {
            return Err(Error::Msg("inconsistent tile size"));
        }

        for (part, num) in &ofmt {
            if ifmt.parts[part].len() != *num {
                return Err(Error::Msg("input & output format don't match"));
            }
        }

        total_tiles += num_tiles(&ofmt) * group.items.len();
    }

    let root = (total_tiles as f64).sqrt().floor() as usize + 1;
    let dimensions = [root * tss.tile_size[0], root * tss.tile_size[1]];
    let mut cursor = TileSetCursor::new(dimensions, tss.tile_size);

    for group in &tss.groups {
        let from: TileSource = try!(load(src_folder.join(&group.from)));
        let ifmt: InputTileFormat = try!(load(src_folder.join(&group.fmt)));

        let mut src_img = try!(image::open(src_folder.join(&from.image_path)));

        for item in &group.items {
            let (x, y) = (item.loc[0], item.loc[1]);
            let mut out_pxs = Vec::<[usize; 2]>::new();
            for tile in ifmt.parts.values().flat_map(|c| c.iter()) {
                let px = try!(cursor.add_tile(&mut src_img, [x + tile[0], y + tile[1]]));
                out_pxs.push(px);
            }

            let mut m = fmts.entry(ifmt.fmt.clone()).or_insert(HashMap::new());
            match m.insert(item.id.clone(), out_pxs) {
                Some(_) => return Err(Error::Msg("duplicate item")),
                _ => { }
            };
        }
    }

    //TODO: FIXXXXXX
    let img_path = {
        let mut p = target.join(tile_set_target_path);
        p.set_extension("png");
        p
    };

    let ts_path = {
        let mut p = target.join(tile_set_target_path);
        p.set_extension("json");
        p
    };

    let ts = TileSet {
        tile_size: tss.tile_size,
        image_path: String::from(img_path.to_str().unwrap()),
        fmts: fmts,
    };

    {
        let mut writer = try!(File::create(ts_path));
        try!(serde_json::ser::to_writer(&mut writer, &ts));
    }

    {
        let mut writer = try!(File::create(img_path));
        try!(cursor.img.save(&mut writer, ImageFormat::PNG));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn it_works() {
        compile_tile_set(&Path::new("test_data/src"),
                         &Path::new("tile_set_sources/morning"),
                         &Path::new("test_data/target"),
                         &Path::new("tile_sets/morning")).expect("compilation failed");
    }
}
