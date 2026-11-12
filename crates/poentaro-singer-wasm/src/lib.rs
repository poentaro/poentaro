//! The wasm component boundary around `poentaro_singer`'s pieces, so the web
//! app can sing a song's pieces on several workers at once.
//!
//! wit-bindgen's generated glue trips `mem_forget` and `disallowed_macros`
//! on code attributed to the macro's expansion (see `poentaro-wasm`'s
//! `component` module), so those are allowed for this crate.
#![allow(clippy::mem_forget, clippy::disallowed_macros)]

use poentaro_singer::SingingPlan;

wit_bindgen::generate!({
    world: "poentaro-singer",
});

struct Component;

impl Guest for Component {
    fn piece_count(plan: Vec<u8>) -> Result<u32, String> {
        let plan = SingingPlan::from_bytes(&plan)?;
        u32::try_from(poentaro_singer::piece_count(&plan)).map_err(|error| error.to_string())
    }

    fn sing_piece(plan: Vec<u8>, index: u32) -> Result<SungPiece, String> {
        let plan = SingingPlan::from_bytes(&plan)?;
        let piece = poentaro_singer::sing_piece(&plan, index as usize)
            .ok_or_else(|| format!("the plan has no piece {index}"))?;
        Ok(SungPiece {
            part: u32::try_from(piece.part).map_err(|error| error.to_string())?,
            start_sample: u32::try_from(piece.start_sample).map_err(|error| error.to_string())?,
            samples: piece.samples,
        })
    }

    fn mix(plan: Vec<u8>, pieces: Vec<SungPiece>) -> Result<Vec<f32>, String> {
        let plan = SingingPlan::from_bytes(&plan)?;
        let pieces = pieces
            .into_iter()
            .map(|piece| poentaro_singer::SungPiece {
                part: piece.part as usize,
                start_sample: piece.start_sample as usize,
                samples: piece.samples,
            })
            .collect::<Vec<_>>();
        Ok(poentaro_singer::mix(&plan, &pieces))
    }
}

export!(Component);
