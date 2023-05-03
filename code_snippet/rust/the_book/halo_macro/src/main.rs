use halo_macro::AbsHaloMakro; // will be referred to by the macro below
use halo_macro_derive::{HiMakroPlusSth, second_custom_macro}; 

#[second_custom_macro(waving_gesture, "/path/to/file")]
#[derive(HiMakroPlusSth)]
struct PanCake;

#[second_custom_macro(discrete_math, "/well/built/game")]
struct FlatPita;

fn main() {
    PanCake::shout_out();
    PanCake::waving_gesture();
    FlatPita::discrete_math();
}
