# lib-comb

`comb` + `combinators` — the combinator library (pure handl).

## Layout

| path | role |
| --- | --- |
| `src/combinators.hnl` | the package entry (`pkg.hnl` `entry`) |
| `src/*.hnl` | one file per combinator — `compose`, `pipe`, `flip`, `partial`, `when`/`unless`, `fanout`/`converge`, `group_by`, … |
| `src/lib.rs`, `src/ops.rs` | const-only Rust crate (embeds the sources) |
| `atoms.yaml`, `Cargo.toml`, `xtask/` | atom registry + Rust gate |
| `pkg.hnl`, `scripts/hnl-gate.sh` | manifest + `.hnl` gate |
| `workflows/gate.yaml` | CI — shared `handl-gates` campaign |

## Contract

- Pure `.hnl` arrows; the Rust side is a const/embed shim, not logic.
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
