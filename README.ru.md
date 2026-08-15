# axiom-verifier v0.2.0

Verifier-facing граница доверия. v0.2 разделяет **производство доказательства** (`axiom-symbolic`, позднее Z3/Lean/zkVM
backends) и **принятие receipt**. Rust-gate проверяет привязку артефактов, verdict и заявленную область корректности до
того, как runtime сможет доверять receipt.
