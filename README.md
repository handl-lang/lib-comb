# lib-comb

`comb` + `combinators` — combinator library (pure handl).

## Layout

```text
src/                     the .hnl sources plus the const crate
pkg.hnl                  package manifest
scripts/hnl-gate.sh      the gate
workflows/gate.yaml                    bupha entry (handl-gates)
```

## Gate

```bash
HNL=../host-cli/target/debug/hnl scripts/hnl-gate.sh
```

## License

MIT.
