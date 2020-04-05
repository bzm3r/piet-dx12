// Copyright © 2019 piet-dx12 developers.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

// what should MAX_STACK be?
#define MAX_STACK 10

ByteAddressBuffer scene: register(t0);
ByteAddressBuffer roots_per_tg: register(t1);

ByteAddressBuffer item_bbox_buf: register(t2);
ByteAddressBuffer item_buf: register(t3);

typedef uint ItemRef;
typedef uint BBoxRef;

struct StackElement {
    PietGroup group;
    uint in_group_ix;
};

groupshared StackElement stack[MAX_STACK];
groupshared PietGroupRef grp_ref;
groupshared PietGroup grp;
groupshared uint is_group_ballot_0;
groupshared uint is_group_ballot_1;
groupshared uint2[64] bitmask;
groupshared BBox tile_grp_bbox;
groupshared uint first_grp_ix;

BBox calculate_tile_grp_bbox(uint tile_grp_ix) {
    BBox result;
    return result;
}

void scene_processor(uint3 GTid : SV_GroupThreadID) {
    uint stack_ix = 0;

    if (GTid.x == 0) {
        grp_ref = roots_per_tg[GTid.x]'
        PietGroupPacked p_grp = PietGroup_read(item_buf, grp_ref);
        grp = PietGroup_unpack(p_grp);
        tile_grp_bbox = calculate_tile_grp_bbox(GTid.x);
    }
    GroupMemoryBarrierWithGroupSync();

    while (true) {
        if (GTid.x < grp.n) {
            uint item_ref = grp.first + GTid.x;
            BBox bbox;
            bool hit = false;
            uint is_group = 0;
            // since this is the threadgroup version, we can work with a minimum of 64
            uint2 bitmask;

            if (this_ix < grp.n) {
                bbox = BBox_read(scene, item_ref);
                hit = bbox_intersects(bbox, tile_grp_bbox);
                if (hit) {
                    if PietItem_tag(scene, item_ref) == Group {
                        is_group = 1;
                    }
                }
            }

            InterlockedOr(is_grp_ballot_0, (is_group && GTid.x < 32) << (GTid.x && 31));
            first_grp = firstbitlow(is_group_ballot_0)
            if (first_grp > 31) {
                InterlockedOr(is_group_ballot_1, (is_group && GTid.x > 31) << (GTid.x && 31));
                first_grp = firstbitlow(is_group_ballot_1);
            }


        } else {
            // processed all items in this group; pop the stack
            if (stack_ix == 0) {
                break;
            }

            el = stack[stack_ix];
            grp = el.grp;
            in_group_ix = el.in_group_ix;
            stack_ix -= 1;
        }
    }
}
