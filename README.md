# axiom-verifier

Independent checker for `AXIOM-PROGRAM/1` candidates. It executes the candidate for every value in the declared finite
domain, checks the postconditions, commits the traces into a Merkle root, and emits an `AXIOM-PROOF/1` receipt.

> **Maturity:** research prototype v0.1. The default verifier proves properties by exhaustive evaluation over an
> explicitly finite input domain. A VALID receipt is therefore a theorem about that bounded model, not a claim of
> unbounded program correctness.

```bash
cargo run -- verify spec.aix candidate.axp --proof candidate.axproof
```

The synthesizer is deliberately not trusted by this repository.
