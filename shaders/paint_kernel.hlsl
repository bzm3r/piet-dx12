// Copyright © 2019 piet-dx12 developers.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

ByteAddressBuffer per_tile_command_list: register(t2);

cbuffer SceneConstants: register(b0) {
    uint num_items;
};

cbuffer GpuStateConstants : register(b1)
{
    uint max_items_scene;
	uint tile_side_length;
    uint num_tiles_x;
    uint num_tiles_y;
};

Texture2D<float> glyph_atlas : register(t3);
RWTexture2D<float4> canvas : register(u1);

inline uint extract_8bit_value(uint bit_shift, uint package) {
    uint mask = 255;
    uint result = (package >> bit_shift) & mask;

    return result;
}

inline uint extract_16bit_value(uint bit_shift, uint package) {
    uint mask = 65535;
    uint result = (package >> bit_shift) & mask;

    return result;
}

typedef uint BBoxRef;
typedef uint BBoxPackedRef;
typedef uint SRGBColorRef;
typedef uint SRGBColorPackedRef;
typedef uint PietGlyphRef;
typedef uint PietGlyphPackedRef;
typedef uint PietCircleRef;
typedef uint PietCirclePackedRef;
typedef uint PietItemRef;

struct BBoxPacked {
    uint x_min_x_max;
    uint y_min_y_max;
};

inline BBoxPacked BBoxPacked_read(ByteAddressBuffer buf, BBoxPackedRef ref) {
    BBoxPacked result;

    uint x_min_x_max = buf.Load(ref);
    result.x_min_x_max = x_min_x_max;

    uint y_min_y_max = buf.Load(ref + 4);
    result.y_min_y_max = y_min_y_max;

    return result;
}

inline uint BBoxPacked_x_min_x_max(ByteAddressBuffer buf, BBoxPackedRef ref) {
    uint x_min_x_max = buf.Load(ref);
    return x_min_x_max;
}

inline uint BBoxPacked_y_min_y_max(ByteAddressBuffer buf, BBoxPackedRef ref) {
    uint y_min_y_max = buf.Load(ref + 4);
    return y_min_y_max;
}

inline uint BBoxPacked_unpack_x_min(uint x_min_x_max) {
    uint result;

    result = extract_16bit_value(16, x_min_x_max);
    return result;
}

inline uint BBoxPacked_unpack_x_max(uint x_min_x_max) {
    uint result;

    result = extract_16bit_value(0, x_min_x_max);
    return result;
}

inline uint BBoxPacked_unpack_y_min(uint y_min_y_max) {
    uint result;

    result = extract_16bit_value(16, y_min_y_max);
    return result;
}

inline uint BBoxPacked_unpack_y_max(uint y_min_y_max) {
    uint result;

    result = extract_16bit_value(0, y_min_y_max);
    return result;
}

struct BBox {
    uint x_min;
    uint x_max;
    uint y_min;
    uint y_max;
};

inline BBox BBoxPacked_unpack(BBoxPacked packed_form) {
    BBox result;

    result.x_min = BBoxPacked_unpack_x_min(packed_form.x_min_x_max);
    result.x_max = BBoxPacked_unpack_x_max(packed_form.x_min_x_max);
    result.y_min = BBoxPacked_unpack_y_min(packed_form.y_min_y_max);
    result.y_max = BBoxPacked_unpack_y_max(packed_form.y_min_y_max);

    return result;
}

struct SRGBColorPacked {
    uint r_g_b_a;
};

inline SRGBColorPacked SRGBColorPacked_read(ByteAddressBuffer buf, SRGBColorPackedRef ref) {
    SRGBColorPacked result;

    uint r_g_b_a = buf.Load(ref);
    result.r_g_b_a = r_g_b_a;

    return result;
}

inline uint SRGBColorPacked_r_g_b_a(ByteAddressBuffer buf, SRGBColorPackedRef ref) {
    uint r_g_b_a = buf.Load(ref);
    return r_g_b_a;
}

inline uint SRGBColorPacked_unpack_r(uint r_g_b_a) {
    uint result;

    result = extract_8bit_value(24, r_g_b_a);
    return result;
}

inline uint SRGBColorPacked_unpack_g(uint r_g_b_a) {
    uint result;

    result = extract_8bit_value(16, r_g_b_a);
    return result;
}

inline uint SRGBColorPacked_unpack_b(uint r_g_b_a) {
    uint result;

    result = extract_8bit_value(8, r_g_b_a);
    return result;
}

inline uint SRGBColorPacked_unpack_a(uint r_g_b_a) {
    uint result;

    result = extract_8bit_value(0, r_g_b_a);
    return result;
}

struct SRGBColor {
    uint r;
    uint g;
    uint b;
    uint a;
};

inline SRGBColor SRGBColorPacked_unpack(SRGBColorPacked packed_form) {
    SRGBColor result;

    result.r = SRGBColorPacked_unpack_r(packed_form.r_g_b_a);
    result.g = SRGBColorPacked_unpack_g(packed_form.r_g_b_a);
    result.b = SRGBColorPacked_unpack_b(packed_form.r_g_b_a);
    result.a = SRGBColorPacked_unpack_a(packed_form.r_g_b_a);

    return result;
}

struct PietGlyphPacked {
    uint tag;
    BBoxPacked scene_bbox;
    BBoxPacked atlas_bbox;
    SRGBColorPacked color;
};

inline PietGlyphPacked PietGlyphPacked_read(ByteAddressBuffer buf, PietGlyphPackedRef ref) {
    PietGlyphPacked result;

    BBoxPacked scene_bbox = BBoxPacked_read(buf, ref + 4);
    result.scene_bbox = scene_bbox;

    BBoxPacked atlas_bbox = BBoxPacked_read(buf, ref + 12);
    result.atlas_bbox = atlas_bbox;

    SRGBColorPacked color = SRGBColorPacked_read(buf, ref + 20);
    result.color = color;

    return result;
}

inline BBoxPacked PietGlyphPacked_scene_bbox(ByteAddressBuffer buf, PietGlyphPackedRef ref) {
    BBoxPacked scene_bbox = BBoxPacked_read(buf, ref + 4);
    return scene_bbox;
}

inline BBoxPacked PietGlyphPacked_atlas_bbox(ByteAddressBuffer buf, PietGlyphPackedRef ref) {
    BBoxPacked atlas_bbox = BBoxPacked_read(buf, ref + 12);
    return atlas_bbox;
}

inline SRGBColorPacked PietGlyphPacked_color(ByteAddressBuffer buf, PietGlyphPackedRef ref) {
    SRGBColorPacked color = SRGBColorPacked_read(buf, ref + 20);
    return color;
}

struct PietGlyph {
    BBox scene_bbox;
    BBox atlas_bbox;
    SRGBColor color;
};

inline PietGlyph PietGlyphPacked_unpack(PietGlyphPacked packed_form) {
    PietGlyph result;

    result.scene_bbox = BBoxPacked_unpack(packed_form.scene_bbox);
    result.atlas_bbox = BBoxPacked_unpack(packed_form.atlas_bbox);
    result.color = SRGBColorPacked_unpack(packed_form.color);

    return result;
}

struct PietCirclePacked {
    uint tag;
    BBoxPacked scene_bbox;
    SRGBColorPacked color;
};

inline PietCirclePacked PietCirclePacked_read(ByteAddressBuffer buf, PietCirclePackedRef ref) {
    PietCirclePacked result;

    BBoxPacked scene_bbox = BBoxPacked_read(buf, ref + 4);
    result.scene_bbox = scene_bbox;

    SRGBColorPacked color = SRGBColorPacked_read(buf, ref + 12);
    result.color = color;

    return result;
}

inline BBoxPacked PietCirclePacked_scene_bbox(ByteAddressBuffer buf, PietCirclePackedRef ref) {
    BBoxPacked scene_bbox = BBoxPacked_read(buf, ref + 4);
    return scene_bbox;
}

inline SRGBColorPacked PietCirclePacked_color(ByteAddressBuffer buf, PietCirclePackedRef ref) {
    SRGBColorPacked color = SRGBColorPacked_read(buf, ref + 12);
    return color;
}

struct PietCircle {
    BBox scene_bbox;
    SRGBColor color;
};

inline PietCircle PietCirclePacked_unpack(PietCirclePacked packed_form) {
    PietCircle result;

    result.scene_bbox = BBoxPacked_unpack(packed_form.scene_bbox);
    result.color = SRGBColorPacked_unpack(packed_form.color);

    return result;
}

struct PietItem {
    uint tag;
    uint body[5];
};
inline uint PietItem_tag(ByteAddressBuffer buf, PietItemRef ref) {
    uint result = buf.Load(ref);
    return result;
}

inline void PietItem_read_into(ByteAddressBuffer src, uint src_ref, RWByteAddressBuffer dst, uint dst_ref) {
    uint4 group0 = src.Load4(src_ref);
    dst.Store4(dst_ref, group0);
}

#define BBOX_SIZE 8
#define SRGBCOLOR_SIZE 4
#define PIET_ITEM_SIZE 24
#define PietItem_Circle 0
#define PietItem_Glyph 1


bool bbox_interiors_intersect(BBox bbox0, BBox bbox1) {
    bool x_intersection = (bbox0.x_max >= bbox1.x_min && bbox1.x_max >= bbox0.x_min);
    bool y_intersection = (bbox0.y_max >= bbox1.y_min && bbox1.y_max >= bbox0.y_min);

    bool intersection = x_intersection && y_intersection;

    return intersection;
}

BBox generate_tile_bbox(uint2 tile_coord) {
    uint tile_x_ix = tile_coord.x;
    uint tile_y_ix = tile_coord.y;

    uint left = tile_side_length*tile_x_ix;
    uint top = tile_side_length*tile_y_ix;
    uint right = left + tile_side_length;
    uint bot = top + tile_side_length;

    BBox result;
    result.x_min = left;
    result.x_max = right;
    result.y_min = top;
    result.y_max = bot;
    return result;
}


bool is_pixel_in_bbox(uint2 pixel_pos, BBox bbox) {
    uint px = pixel_pos.x;
    uint py = pixel_pos.y;

    // use of explicit result to avoid the following warning:
    // warning X4000: use of potentially uninitialized variable
    bool result = 0;

    uint left = bbox.x_min;
    uint right = bbox.x_max;
    uint top = bbox.y_min;
    uint bot = bbox.y_max;

    if (left <= px && px <= right) {
        if (top <= py && py <= bot) {
            result = 1;
        }
    }

    return result;
}

float circle_alpha(uint2 pixel_pos, BBox circle_bbox, float color_alpha) {
    uint2 circle_center = {lerp(circle_bbox.x_min, circle_bbox.x_max, 0.5), lerp(circle_bbox.y_min, circle_bbox.y_max, 0.5)};
    float radius = (circle_bbox.x_max - circle_bbox.x_min)*0.5;
    float d = distance(pixel_pos, circle_center);
    float position_alpha = clamp(radius - d, 0.0, 1.0);

    float pixel_alpha = color_alpha*position_alpha;

    return pixel_alpha;
}

float glyph_alpha(uint2 pixel_pos, BBox scene_bbox, BBox atlas_bbox, float color_alpha) {
    uint2 atlas_pixel_pos = {atlas_bbox.x_min + (pixel_pos.x - scene_bbox.x_min), atlas_bbox.y_min + (pixel_pos.y - scene_bbox.y_min)};
    float glyph_alpha = glyph_atlas[atlas_pixel_pos];

    float pixel_alpha = color_alpha*glyph_alpha;

    return pixel_alpha;
}

float4 blend_pd_over(float4 bg, float4 fg) {
    return lerp(bg, float4(fg.rgb, 1.0), fg.a);
}

#define NUM_CMD_OFFSET 4

[numthreads(16, 16, 1)]
void paint_items(uint3 Gid: SV_GroupID, uint3 DTid : SV_DispatchThreadID) {
    float4 bg = {0.0, 0.0, 0.0, 0.0};
    float4 fg = {0.0, 0.0, 0.0, 0.0};

    uint2 pixel_pos = DTid.xy;

    uint tile_ix = Gid.y*num_tiles_x + Gid.x;
    uint cmd_list_size = NUM_CMD_OFFSET + num_items*PIET_ITEM_SIZE;
    uint cmd_init_offset = cmd_list_size*tile_ix;
    uint num_cmd = per_tile_command_list.Load(cmd_init_offset);
    uint cmd_item_start = cmd_init_offset + NUM_CMD_OFFSET;

    /*
    if (num_items == 2) {
        fg.g = 1.0;
        fg.a = 1.0;
    } else {
        fg.r = 1.0;
        fg.a = 1.0;
    }
    */

    /*
    for (uint i = 0; i < num_cmd; i++) {
        uint item_offset = cmd_item_start + PIET_ITEM_SIZE*i;
        uint tag = PietItem_tag(per_tile_command_list, item_offset);

        PietCirclePacked packed_circle = PietCirclePacked_read(per_tile_command_list, cmd_item_start);
        SRGBColor color = SRGBColorPacked_unpack(packed_circle.color);
        BBox scene_bbox = BBoxPacked_unpack(packed_circle.scene_bbox);

        if (tag == PietItem_Circle) {
            fg.g = 1.0;
            fg.a = 0.5;
        } else {
            fg.r = 1.0;
            fg.a = 0.5;
        }

        bg = blend_pd_over(bg, fg);
    }
    */

    for (uint i = 0; i < num_cmd; i++) {
        uint item_offset = cmd_item_start + PIET_ITEM_SIZE*i;
        uint tag = PietItem_tag(per_tile_command_list, item_offset);

        if (tag == PietItem_Circle) {
            BBoxPacked packed_scene_bbox = PietCirclePacked_scene_bbox(per_tile_command_list, item_offset);
            BBox scene_bbox = BBoxPacked_unpack(packed_scene_bbox);
            SRGBColorPacked packed_color = PietCirclePacked_color(per_tile_command_list, item_offset);
            SRGBColor color = SRGBColorPacked_unpack(packed_color);

            fg.r = color.r/255.0;
            fg.g = color.g/255.0;
            fg.b = color.b/255.0;
            fg.a = color.a/255.0;
            fg.a = circle_alpha(pixel_pos, scene_bbox, color.a/255.0);
            bg = blend_pd_over(bg, fg);
        } else if (tag == PietItem_Glyph) {
            BBoxPacked packed_scene_bbox = PietGlyphPacked_scene_bbox(per_tile_command_list, item_offset);
            BBox scene_bbox = BBoxPacked_unpack(packed_scene_bbox);

            if (is_pixel_in_bbox(pixel_pos, scene_bbox)) {
                BBoxPacked packed_atlas_bbox = PietGlyphPacked_atlas_bbox(per_tile_command_list, item_offset);
                BBox atlas_bbox = BBoxPacked_unpack(packed_atlas_bbox);

                SRGBColorPacked packed_color = PietGlyphPacked_color(per_tile_command_list, item_offset);
                SRGBColor color = SRGBColorPacked_unpack(packed_color);

                fg.r = color.r/255.0;
                fg.g = color.g/255.0;
                fg.b = color.b/255.0;
                fg.a = glyph_alpha(pixel_pos, scene_bbox, atlas_bbox, color.a/255.0);
                bg = blend_pd_over(bg, fg);
            }
        }
    }

    canvas[DTid.xy] = bg;
}
