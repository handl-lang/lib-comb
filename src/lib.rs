//! `comb` — Handl stdlib port.

/// Checkout root of this repo (build-time).
pub const ROOT: &str = env!("CARGO_MANIFEST_DIR");

pub const APPLY_HNL: &str = include_str!("apply.hnl");

pub const CHOICE_HNL: &str = include_str!("choice.hnl");

pub const COMPOSE_HNL: &str = include_str!("compose.hnl");

pub const CONSTANTLY_HNL: &str = include_str!("constantly.hnl");

pub const CONVERGE_HNL: &str = include_str!("converge.hnl");

pub const FANOUT_FN_HNL: &str = include_str!("fanout_fn.hnl");

pub const FANOUT_HNL: &str = include_str!("fanout.hnl");

pub const FLIP_HNL: &str = include_str!("flip.hnl");

pub const FLOW_HNL: &str = include_str!("flow.hnl");

pub const PARTIAL_HNL: &str = include_str!("partial.hnl");

pub const PIPE_HNL: &str = include_str!("pipe.hnl");

pub const TAP_HNL: &str = include_str!("tap.hnl");

pub const UNLESS_HNL: &str = include_str!("unless.hnl");

pub const WHEN_HNL: &str = include_str!("when.hnl");
