#[macro_use]
extern crate piet_gpu_derive;

use piet_gpu_types::encoder::{Encode, Encoder};

fn main() {
    print!("{}", piet_gpu_types::scene::gen_gpu_scene("HLSL"));
}
