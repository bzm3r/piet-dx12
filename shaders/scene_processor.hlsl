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
ByteAddressBuffer roots_per_invocation: register(t1);

ByteAddressBuffer item_bbox_buf: register(t2);
ByteAddressBuffer item_buf: register(t3);

typedef uint ItemRef;
typedef uint BBoxRef;

struct StackElement {
    PietGroup group;
    uint in_group_ix;
};

void scene_processor(uint3 DTid : SV_DispatchThreadID) {
    StackElement stack[MAX_STACK];

    uint stack_ix = 0;
    // For simplicity's sake, let us assume that we have a 1D dispatch.
    PietGroupRef grp_ref = roots_per_invocation[DTid.x];
    PietGroupPacked p_grp = PietGroup_read(item_buf, grp_ref);
    PietGroup grp = PietGroup_unpack(p_grp);
    BBox tilegroup_bounds = >>>> get tilegroup bounds for this invocation

    uint in_group_ix = 0;
    while (true) {
        if (in_group_ix < grp.n) {
            uint item_ref = in_group_ix + grp_desc.first;
            bool hit = false;
            bool is_group = false;

            BBoxPacked p_bbox = BBox_read(item_bbox_buf, item_ref);
            BBox bbox = BBox_unpack(p_bbox);

            hit = bbox_intersects(bbox, tilegroup_bounds);
            is_group = (PietItem_tag(item_data, item_ix) == PietItemGroup);

            if (hit && !is_group) {
                // write item with item_ref to output
            }

            // why is this not supposed to be hit && is_group?
            if (is_group) {
                uint next_ix = ix + 1;
                if (next_ix < grp.n) {
                    StackElement el;
                    el.group = grp;
                    el.in_group_ix = next_ix;
                    stack[stack_ix] = el;
                    stack_ix += 1;
                }
                PietGroupPacked p_new_grp = PietGroup_read(item_data, item_ref);
                PietGroup new_grp = PietGroup_unpack(p_new_grp);
                new_grp.in_group_offset += grp.in_group_offset;
                grp = new_grp;
                in_group_ix = 0;
            } else {
                in_group_ix += 1;
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
