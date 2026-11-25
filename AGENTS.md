# lib-comb

`comb` + `combinators` — the combinator library (pure handl).

## Layout

| path | role |
| --- | --- |
| `src/combinators.hnl` | the `combinators` module doc surface (`pkg.hnl` `entry`) — spec-driven, no decls |
| `src/*.hnl` | one file per combinator — `compose`, `pipe`, `flip`, `partial`, `when`/`unless`, `fanout`/`converge`, `group_by`, … (the `comb.*` module) |
| `src/lib.rs`, `src/ops.rs` | embed consts + the native `combinators.*` `Op` impls |
| `atoms.yaml`, `Cargo.toml`, `xtask/` | atom registry + Rust gate |
| `pkg.hnl`, `scripts/hnl-gate.sh` | manifest + `.hnl` gate |
| `workflows/gate.yaml` | CI — shared `handl-gates` campaign |

## Contract

- Two surfaces: `comb.*` = the per-arrow pure-handl files (loaded by
  `kit-prelude`'s `CombModule`); `combinators.*` = the 16 native
  `handl_ops::Op` impls in `src/ops.rs`.
- `atoms.yaml` (P-013/T-151) is the `combinators` spec — its `impl:`
  keys (`fn_pipe`, …) resolve against the host's `combinators_impls()`
  table in `plugin-lib-stdlib` `ops/kernel_impls.rs`, NOT against
  anything in this repo. `combinators.hnl` is doc-only by convention
  (the cap/op/handle decl triples were stripped fleet-wide).
- Deps resolve from git — plain clone test-runs as-is.

## Verification

```bash
HNL=../host-cli/target/debug/hnl scripts/hnl-gate.sh
cargo xtask gate
```

<!-- myco:agent-awareness -->

## Agent awareness

- Session start / any work discussion: run `myco status --prompt`
  before proposing next steps — know the board first.
- User describes new work → offer to capture it (`myco todo add`,
  `issue new`, `proposal new`). Don't wait to be asked.
- Picking up a task: `myco resume <ID>` is the briefing.
- Wrapping up: `myco handoff <ID>` anything resumable; keep statuses
  honest (claimed → in-progress → blocked/done).

<!-- /myco:agent-awareness -->
