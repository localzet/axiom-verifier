# axiom-verifier v0.2.0

The verifier-facing trust boundary. v0.2 distinguishes **proof production** (`axiom-symbolic`, later Z3/Lean/zkVM
backends) from **receipt acceptance**. The Rust gate checks artifact binding, verdict and declared soundness scope
before a runtime can trust a receipt.
