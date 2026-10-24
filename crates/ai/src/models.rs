//! Trained networks embedded in the binary (so they work offline and in WebAssembly).
//! Produced by `train/train_bid.py`.

use crate::nn::Mlp;
use std::sync::OnceLock;

static BID: OnceLock<Option<Mlp>> = OnceLock::new();
static ALT_BID: OnceLock<Option<Mlp>> = OnceLock::new();
static PLAY: OnceLock<Option<Mlp>> = OnceLock::new();

/// The neural card-play network, if a usable model is embedded (`train/train_play.py`).
pub fn play_model() -> Option<&'static Mlp> {
    PLAY.get_or_init(|| {
        let model = Mlp::from_bytes(include_bytes!("../models/play.fhnn")).ok()?;
        (model.inputs() == crate::features::PLAY_FEATURES && model.outputs() == crate::features::PLAY_SLOTS).then_some(model)
    })
    .as_ref()
}

/// A second bidding network for lab comparisons, read from the file named by `FH_ALT_BID_MODEL`
/// (lab variant `oracle-altnet`); `None` if it's unset or unusable.
pub fn alt_bid_model() -> Option<&'static Mlp> {
    ALT_BID
        .get_or_init(|| {
            let bytes = std::fs::read(std::env::var("FH_ALT_BID_MODEL").ok()?).ok()?;
            let model = Mlp::from_bytes(&bytes).ok()?;
            (model.inputs() == crate::features::BID_FEATURES && model.outputs() == crate::features::BID_OUTPUTS).then_some(model)
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
