use anyhow::{bail, Context, Result};
use sha2::{Digest, Sha256};
use std::{collections::BTreeMap, env, fs};

fn main() -> Result<()> {
    let mut args = env::args().skip(1);
    match args.next().as_deref() {
        Some("check-receipt") => {
            let spec = fs::read_to_string(args.next().context("missing spec")?)?;
            let program = fs::read_to_string(args.next().context("missing program")?)?;
            let proof = fs::read_to_string(args.next().context("missing proof")?)?;
            validate_receipt(&spec, &program, &proof)?;
            println!("VALID RECEIPT BINDING");
        }
        _ => bail!("usage: axiom-verifier check-receipt spec.aix program.axp program.axproof"),
    }
    Ok(())
}

fn validate_receipt(spec: &str, program: &str, proof: &str) -> Result<()> {
    let doc = parse_receipt(proof)?;
    if doc.get("verdict").map(String::as_str) != Some("VALID") {
        bail!("receipt verdict is not VALID");
    }
    if doc.get("soundness.scope").map(String::as_str) != Some("exact-for-supported-fragment") {
        bail!("unsupported or missing soundness scope");
    }
    if doc.get("spec.sha256") != Some(&sha256_hex(spec.as_bytes())) {
        bail!("spec hash does not match receipt");
    }
    if doc.get("program.sha256") != Some(&sha256_hex(program.as_bytes())) {
        bail!("program hash does not match receipt");
    }
    if doc.get("counterexample").map(String::as_str) != Some("none") {
        bail!("VALID receipt unexpectedly carries a counterexample commitment");
    }
    Ok(())
}

fn parse_receipt(raw: &str) -> Result<BTreeMap<String, String>> {
    let mut lines = raw.lines();
    if lines.next() != Some("AXIOM-PROOF/2") {
        bail!("bad proof header");
    }
    let mut map = BTreeMap::new();
    for line in lines.filter(|line| !line.trim().is_empty()) {
        let (key, value) = line.split_once('=').context("bad proof line")?;
        if map.insert(key.to_owned(), value.to_owned()).is_some() {
            bail!("duplicate proof key: {key}");
        }
    }
    Ok(map)
}

fn sha256_hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::validate_receipt;

    #[test]
    fn rejects_invalid_verdict() {
        let proof = "AXIOM-PROOF/2\nverdict=INVALID\n";
        assert!(validate_receipt("s", "p", proof).is_err());
    }
}
