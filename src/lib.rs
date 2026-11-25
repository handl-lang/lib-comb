//! `comb` / `combinators` — Handl stdlib port.
//!
//! The per-arrow `.hnl` files are the pure-handl `comb` module surface;
//! `combinators.hnl` + `ops.rs` are the `combinators` module doc
//! surface and its native `Op` impls (moved verbatim from the retired
//! `kernel/crates/combinators` floor-lib crate). The `combinators.*`
//! interface spec is `atoms.yaml` (P-013/T-151) — its `impl:` keys
//! resolve against the host's `combinators_impls()` table in
//! `plugin-lib-stdlib`'s `ops/kernel_impls.rs`.

/// Checkout root of this repo (build-time).
pub const ROOT: &str = env!("CARGO_MANIFEST_DIR");

/// The handl source declaring the `combinators` interface.
pub const COMBINATORS_HNL: &str = include_str!("combinators.hnl");

pub mod ops;

pub use ops::{COMBINATORS_OPS, combinators_op_registry};

pub const APPLY_HNL: &str = include_str!("apply.hnl");

pub const CHOICE_HNL: &str = include_str!("choice.hnl");

pub const COMPOSE_HNL: &str = include_str!("compose.hnl");

pub const CONSTANTLY_HNL: &str = include_str!("constantly.hnl");

pub const CONVERGE_HNL: &str = include_str!("converge.hnl");

pub const COUNT_BY_HNL: &str = include_str!("count_by.hnl");

pub const EVERY_HNL: &str = include_str!("every.hnl");

pub const FANOUT_FN_HNL: &str = include_str!("fanout_fn.hnl");

pub const FANOUT_HNL: &str = include_str!("fanout.hnl");

pub const FIND_HNL: &str = include_str!("find.hnl");

pub const FLAT_MAP_HNL: &str = include_str!("flat_map.hnl");

pub const FLIP_HNL: &str = include_str!("flip.hnl");

pub const FLOW_HNL: &str = include_str!("flow.hnl");

pub const GROUP_BY_HNL: &str = include_str!("group_by.hnl");

pub const PARTIAL_HNL: &str = include_str!("partial.hnl");

pub const PARTITION_HNL: &str = include_str!("partition.hnl");

pub const PIPE_HNL: &str = include_str!("pipe.hnl");

pub const SOME_HNL: &str = include_str!("some.hnl");

pub const TAP_HNL: &str = include_str!("tap.hnl");

pub const UNLESS_HNL: &str = include_str!("unless.hnl");

pub const WHEN_HNL: &str = include_str!("when.hnl");
