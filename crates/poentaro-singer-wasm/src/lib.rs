//! The wasm component boundary around `poentaro_singer::sing`.
//!
//! wit-bindgen's generated glue trips `mem_forget` and `disallowed_macros`
//! on code attributed to the macro's expansion (see `poentaro-wasm`'s
//! `component` module), so those are allowed for this crate.
#![allow(clippy::mem_forget, clippy::disallowed_macros)]

wit_bindgen::generate!({
    world: "poentaro-singer",
});

struct Component;

impl Guest for Component {
    fn sing(plan: Vec<u8>) -> Result<Vec<f32>, String> {
        let plan = poentaro_singer::SingingPlan::from_bytes(&plan)?;
        Ok(poentaro_singer::sing(&plan))
    }
}

export!(Component);
