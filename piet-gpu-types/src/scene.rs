pub use self::scene::{BBox, SRGBColor, PietCircle, PietGlyph, PietItem};

piet_gpu! {
    #[rust_encode]
    mod scene {
        struct BBox {
            x0: u16,
            x1: u16,
            y0: u16,
            y1: u16,
        }

        struct SRGBColor {
            r: u8,
            g: u8,
            b: u8,
            a: u8,
        }

        struct PietGlyph {
            scene_bbox: BBox,
            atlas_bbox: BBox,
            color: SRGBColor,
        }

        struct PietCircle {
            scene_bbox: BBox,
            color: SRGBColor,
        }

        struct PietGroup {
            n: u32,
            first: u32,
            in_group_offset: [f32; 2],
            scene_bbox: BBox,
        }

        enum PietItem {
            Circle(PietCircle),
            Glyph(PietGlyph),
            Group(PietGroup),
        }
    }
}
