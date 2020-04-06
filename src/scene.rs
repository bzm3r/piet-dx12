// Copyright © 2019 piet-dx12 developers.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

extern crate byteorder;
extern crate font_rs;
extern crate kurbo;
extern crate rand;

use kurbo::{Circle, Rect, Shape};
use std::convert::TryFrom;

use piet_gpu_types::encoder::{Encode, Encoder};
use piet_gpu_types::scene::{BBox, PietCircle, PietGlyph, PietItem, SRGBColor};

fn rect_to_bbox(bbox: &Rect) -> BBox {
    // TODO: should more attention be paid to f64 to u16 conversion?
    BBox {
        x0: bbox.x0 as u16,
        x1: bbox.x1 as u16,
        y0: bbox.y0 as u16,
        y1: bbox.y1 as u16,
    }
}

fn bytes_to_color(color: &[u8; 4]) -> SRGBColor {
    SRGBColor {
        r: color[0],
        g: color[1],
        b: color[2],
        a: color[3],
    }
}

pub struct PlacedGlyph {
    pub atlas_bbox: Rect,
    pub placed_bbox: Rect,
}

pub struct Scene {
    pub num_items: u32,
    pub item_bboxes: Encoder,
    pub items: Encoder,
}

impl Scene {
    pub fn new_empty() -> Scene {
        Scene {
            num_items: 0,
            item_bboxes: Encoder::new(),
            items: Encoder::new(),
        }
    }

    pub fn append_circle(&mut self, circle: Circle, color: [u8; 4]) {
        let scene_bbox = rect_to_bbox(&circle.bounding_box());
        scene_bbox.encode(&mut self.item_bboxes);

        let c = PietCircle {
            scene_bbox,
            color: bytes_to_color(&color),
        };
        let item = PietItem::Circle(c);
        item.encode(&mut self.items);

        self.num_items += 1;
    }

    pub fn append_glyph(&mut self, scene_bbox: Rect, atlas_bbox: Rect, color: [u8; 4]) {
        let scene_bbox = rect_to_bbox(&scene_bbox);
        scene_bbox.encode(&mut self.item_bboxes);

        let g = PietGlyph {
            scene_bbox,
            atlas_bbox: rect_to_bbox(&atlas_bbox),
            color: bytes_to_color(&color),
        };
        let item = PietItem::Glyph(g);
        item.encode(&mut self.items);

        self.num_items += 1;
    }

    pub fn add_text(
        &mut self,
        screen_x_offset: u16,
        screen_y_offset: u16,
        placed_glyphs: &[PlacedGlyph],
        color: [u8; 4],
    ) {
        for pg in placed_glyphs.iter() {
            let scene_bbox = Rect {
                x0: pg.placed_bbox.x0 + (screen_x_offset as f64),
                x1: pg.placed_bbox.x1 + (screen_x_offset as f64),
                y0: pg.placed_bbox.y0 + (screen_y_offset as f64),
                y1: pg.placed_bbox.y1 + (screen_y_offset as f64),
            };
            // println!("scene | x0: {}, x1: {}, y0: {}, y1: {}", scene_bbox.x0, scene_bbox.x1, scene_bbox.y0, scene_bbox.y1);
            // println!("atlas | x0: {}, x1: {}, y0: {}, y1: {}", pg.atlas_bbox.x0, pg.atlas_bbox.x1, pg.atlas_bbox.y0, pg.atlas_bbox.y1);
            self.append_glyph(scene_bbox, pg.atlas_bbox, color);
        }
    }
}
