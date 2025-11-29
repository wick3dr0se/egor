#![allow(dead_code)]

use std::{collections::HashMap, fs::read_to_string, path::Path};

use image::GenericImageView;
use serde::Deserialize;
use serde_json::from_str;

use egor::{
    math::{Rect, Vec2, vec2},
    render::{Color, Graphics},
};

#[derive(Deserialize, Debug)]
pub struct TiledObject {
    pub id: u32,
    pub name: String,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

#[derive(Deserialize, Debug)]
pub struct TiledLayer {
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub name: String,
    pub visible: bool,
    pub data: Option<Vec<u32>>,
    pub objects: Option<Vec<TiledObject>>,
}

#[derive(Deserialize, Debug)]
pub struct TiledTileset {
    pub firstgid: u32,
    pub image: Option<String>,
    pub tilecount: Option<u32>,
    pub tilewidth: Option<u32>,
    pub tileheight: Option<u32>,
}

#[derive(Deserialize, Debug)]
pub struct TiledMap {
    pub width: u32,
    pub height: u32,
    pub tilewidth: u32,
    pub tileheight: u32,
    pub layers: Vec<TiledLayer>,
    pub tilesets: Vec<TiledTileset>,
}

impl TiledMap {
    pub fn load(path: &str) -> Self {
        let raw = read_to_string(path).expect("read Tiled JSON");
        from_str(&raw).expect("parse Tiled JSON")
    }

    pub fn tile_size(&self) -> (f32, f32) {
        (self.tilewidth as f32, self.tileheight as f32)
    }

    pub fn dimensions(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    pub fn tile_to_world(&self, x: u32, y: u32) -> Vec2 {
        vec2(
            x as f32 * self.tilewidth as f32,
            y as f32 * self.tileheight as f32,
        )
    }

    /// iterate every non‑empty tile (x,y,gid) that overlaps `rect``
    pub fn visible_tiles(
        &self,
        layer: &TiledLayer,
        rect: &Rect,
    ) -> impl Iterator<Item = (u32, u32, u32)> {
        let (tw, th) = self.tile_size();
        let (lw, lh) = (layer.width.unwrap(), layer.height.unwrap());

        let min_x = (rect.min().x / tw).floor().clamp(0.0, (lw - 1) as f32) as u32;
        let max_x = (rect.max().x / tw).ceil().clamp(0.0, lw as f32) as u32;
        let min_y = (rect.min().y / th).floor().clamp(0.0, (lh - 1) as f32) as u32;
        let max_y = (rect.max().y / th).ceil().clamp(0.0, lh as f32) as u32;

        let data = layer.data.as_ref().unwrap();

        (min_y..max_y).flat_map(move |y| {
            (min_x..max_x).filter_map(move |x| {
                let gid = data[(y * lw + x) as usize];
                if gid == 0 { None } else { Some((x, y, gid)) }
            })
        })
    }
}

struct TilesetInfo {
    tex_id: usize,
    first_gid: u32,
    tile_w: u32,
    tile_h: u32,
    atlas_w: u32,
    atlas_h: u32,
    per_row: u32,
}

pub struct EgorMap {
    tiled: TiledMap,
    sets: HashMap<u32, TilesetInfo>, // key = first_gid
}

impl EgorMap {
    pub fn new(path: &str) -> Self {
        Self {
            tiled: TiledMap::load(path),
            sets: HashMap::new(),
        }
    }

    pub fn load(&mut self, gfx: &mut Graphics) {
        for ts in &self.tiled.tilesets {
            let (Some(img), Some(tw), Some(th)) = (&ts.image, ts.tilewidth, ts.tileheight) else {
                continue;
            };

            let bytes = std::fs::read(Path::new("assets").join(img)).expect("read atlas png");
            let tex_id = gfx.load_texture(&bytes);

            let (aw, ah) = image::load_from_memory(&bytes).unwrap().dimensions();

            self.sets.insert(
                ts.firstgid,
                TilesetInfo {
                    tex_id,
                    first_gid: ts.firstgid,
                    tile_w: tw,
                    tile_h: th,
                    atlas_w: aw,
                    atlas_h: ah,
                    per_row: aw / tw.max(1),
                },
            );
        }
    }

    pub fn render(&mut self, gfx: &mut Graphics) {
        let screen = gfx.screen_size();
        let view = gfx.camera().viewport(screen);
        let (tw, th) = self.tiled.tile_size().into();

        for layer in &self.tiled.layers {
            if !layer.visible || layer.data.is_none() {
                continue;
            }

            for (x, y, gid) in self.tiled.visible_tiles(layer, &view) {
                let (info, uv) = match self.lookup_gid(gid) {
                    Some(v) => v,
                    None => continue,
                };

                gfx.rect()
                    .at(self.tiled.tile_to_world(x, y))
                    .size(Vec2::new(tw, th))
                    .texture(info.tex_id)
                    .color(Color::WHITE)
                    .uv(uv);
            }
        }
    }

    // gid → (tileset, uv‑quad)
    fn lookup_gid(&self, gid: u32) -> Option<(&TilesetInfo, [[f32; 2]; 4])> {
        let (_, info) = self
            .sets
            .iter()
            .filter(|(fg, _)| gid >= **fg)
            .max_by_key(|(fg, _)| **fg)?;

        let local = gid - info.first_gid;
        let tx = (local % info.per_row) * info.tile_w;
        let ty = (local / info.per_row) * info.tile_h;

        let aw = info.atlas_w as f32;
        let ah = info.atlas_h as f32;

        let u0 = tx as f32 / aw;
        let v0 = ty as f32 / ah;
        let u1 = (tx + info.tile_w) as f32 / aw;
        let v1 = (ty + info.tile_h) as f32 / ah;

        Some((info, [[u0, v0], [u1, v0], [u1, v1], [u0, v1]]))
    }
}
