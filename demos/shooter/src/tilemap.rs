#![allow(dead_code)]

use std::collections::HashMap;

use image::GenericImageView;
use serde::Deserialize;
use serde_json::from_str;

use egor::{
    math::{Rect, Vec2, vec2},
    render::{Color, Graphics, Texture},
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
    pub fn load(json_data: &str) -> Self {
        from_str(json_data).expect("parse Tiled JSON")
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

    /// iterate every non‑empty tile (x,y,gid) that overlaps `rect`
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
    tex: Texture,
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
    pub fn new(json_data: &str) -> Self {
        Self {
            tiled: TiledMap::load(json_data),
            sets: HashMap::new(),
        }
    }

    pub fn load_tileset(&mut self, gfx: &mut Graphics, bytes: &[u8], name: &str) {
        for ts in &self.tiled.tilesets {
            let (Some(img), Some(tw), Some(th)) = (&ts.image, ts.tilewidth, ts.tileheight) else {
                continue;
            };

            // Match by filename only (ignore path)
            let img_name = img.rsplit('/').next().unwrap_or(img);
            if img_name != name {
                continue;
            }

            let tex = gfx.load_texture(bytes);
            let (aw, ah) = image::load_from_memory(bytes).unwrap().dimensions();

            self.sets.insert(
                ts.firstgid,
                TilesetInfo {
                    tex,
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
        let (tw, th) = self.tiled.tile_size();

        for layer in &self.tiled.layers {
            if !layer.visible || layer.data.is_none() {
                continue;
            }

            // darken the walls so the floor is the bright, walkable surface
            let tint = if layer.name == "walls" {
                Color::new([0.34, 0.33, 0.32, 1.0])
            } else {
                Color::WHITE
            };

            for (x, y, gid) in self.tiled.visible_tiles(layer, &view) {
                let (info, uv) = match self.lookup_gid(gid) {
                    Some(v) => v,
                    None => continue,
                };

                gfx.rect()
                    .at(self.tiled.tile_to_world(x, y))
                    .size(Vec2::new(tw, th))
                    .texture(&info.tex)
                    .color(tint)
                    .uv(uv);
            }
        }
    }

    /// Center of a named spawn object, in world coordinates
    pub fn spawn(&self, name: &str) -> Option<Vec2> {
        self.tiled
            .layers
            .iter()
            .filter_map(|l| l.objects.as_ref())
            .flatten()
            .find(|o| o.name == name)
            .map(|o| vec2(o.x + o.width * 0.5, o.y + o.height * 0.5))
    }

    /// Map size in tiles
    pub fn dimensions(&self) -> (u32, u32) {
        self.tiled.dimensions()
    }

    /// Size of one tile in pixels
    pub fn tile_size(&self) -> (f32, f32) {
        self.tiled.tile_size()
    }

    /// World-space size of one tile
    pub fn tile_px(&self) -> Vec2 {
        let (tw, th) = self.tiled.tile_size();
        vec2(tw, th)
    }

    /// Whether the tile under this world point blocks movement
    pub fn solid_at(&self, world: Vec2) -> bool {
        let (tw, th) = self.tiled.tile_size();
        self.solid_tile((world.x / tw).floor() as i32, (world.y / th).floor() as i32)
    }

    /// Whether this tile index blocks movement, treating outside the map as solid
    pub fn solid_tile(&self, tx: i32, ty: i32) -> bool {
        let Some(layer) = self
            .tiled
            .layers
            .iter()
            .find(|l| l.name == "walls" && l.data.is_some())
        else {
            return false;
        };

        let lw = layer.width.unwrap() as i32;
        let lh = layer.height.unwrap() as i32;
        if tx < 0 || ty < 0 || tx >= lw || ty >= lh {
            return true;
        }

        layer.data.as_ref().unwrap()[(ty * lw + tx) as usize] != 0
    }

    /// Whether a square of half-extent `half` around `center` overlaps solid tiles
    pub fn overlaps_wall(&self, center: Vec2, half: f32) -> bool {
        self.first_solid_overlap(center, half).is_some()
    }

    /// Move a square with half-extent `half` by `delta`, stopping at solid tiles
    ///
    /// Substepped so a fast mover cannot tunnel through a thin wall.
    pub fn slide(&self, center: Vec2, half: f32, delta: Vec2) -> Vec2 {
        let steps = (delta.length() / 8.0).ceil().max(1.0) as u32;
        let step = delta / steps as f32;

        (0..steps).fold(center, |pos, _| self.slide_step(pos, half, step))
    }

    fn slide_step(&self, mut center: Vec2, half: f32, delta: Vec2) -> Vec2 {
        let (tw, th) = self.tiled.tile_size();

        center.x += delta.x;
        if let Some((tx, _)) = self.first_solid_overlap(center, half) {
            center.x = if delta.x > 0.0 {
                tx as f32 * tw - half
            } else {
                (tx + 1) as f32 * tw + half
            };
        }

        center.y += delta.y;
        if let Some((_, ty)) = self.first_solid_overlap(center, half) {
            center.y = if delta.y > 0.0 {
                ty as f32 * th - half
            } else {
                (ty + 1) as f32 * th + half
            };
        }

        center
    }

    /// First solid tile overlapped by the square around `center`
    fn first_solid_overlap(&self, center: Vec2, half: f32) -> Option<(i32, i32)> {
        let (tw, th) = self.tiled.tile_size();
        let min = center - Vec2::splat(half);
        let max = center + Vec2::splat(half - 0.0001);

        for ty in (min.y / th).floor() as i32..=(max.y / th).floor() as i32 {
            for tx in (min.x / tw).floor() as i32..=(max.x / tw).floor() as i32 {
                if self.solid_tile(tx, ty) {
                    return Some((tx, ty));
                }
            }
        }
        None
    }

    // gid → (tileset, uv‑quad)
    fn lookup_gid(&self, gid: u32) -> Option<(&TilesetInfo, [f32; 4])> {
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

        Some((info, [u0, v0, u1, v1]))
    }
}
