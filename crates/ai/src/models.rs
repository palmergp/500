//! Trained networks embedded in the binary (so they work offline and in WebAssembly).
//! Produced by `train/train_bid.py`.

use crate::nn::Mlp;
use std::sync::OnceLock;

static BID: OnceLock<Option<Mlp>> = OnceLock::new();
static PLAY: OnceLock<Option<Mlp>> = OnceLock::new();

/// The neural card-play network, if a usable model is embedded (`train/train_play.py`).
pub fn play_model() -> Option<&'static Mlp> {
    PLAY.get_or_init(|| {
        let model = Mlp::from_bytes(include_bytes!("../models/play.fhnn")).ok()?;
        (model.inputs() == crate::features::PLAY_FEATURES && model.outputs() == crate::features::PLAY_SLOTS).then_some(model)
    })
    .as_ref()
}

/// The neural bidding evaluator, if a usable model is embedded.
pub fn bid_model() -> Option<&'static Mlp> {
    BID.get_or_init(|| {
        let model = Mlp::from_bytes(include_bytes!("../models/bid.fhnn")).ok()?;
        (model.inputs() == crate::features::BID_FEATURES && model.outputs() == crate::features::BID_OUTPUTS).then_some(model)
    })
    .as_ref()
}
