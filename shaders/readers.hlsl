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
