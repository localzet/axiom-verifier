use anyhow::{bail, Context, Result};
use sha2::{Digest, Sha256};
use std::{collections::BTreeMap, env, fs};

fn main() -> Result<()> {
    let mut a = env::args().skip(1);
    if a.next().as_deref() != Some("verify") {
        bail!("usage: axiom-verifier verify spec.aix program.axp --proof out.axproof");
    }
    let sp = a.next().context("missing spec")?;
    let pp = a.next().context("missing program")?;
    if a.next().as_deref() != Some("--proof") {
        bail!("expected --proof");
    }
    let op = a.next().context("missing proof path")?;
    let spec_raw = fs::read_to_string(sp)?;
    let prog_raw = fs::read_to_string(pp)?;
    let spec = parse_ir(&spec_raw)?;
    let prog = parse_program(&prog_raw)?;
    if spec["module"] != prog.module {
        bail!("module mismatch");
    }
    let min: i64 = spec["domain.min"].parse()?;
    let max: i64 = spec["domain.max"].parse()?;
    let clauses = ensures(&spec);
    let mut leaves = Vec::new();
    let mut counterexample = None;
    for x in min..=max {
        let r = run(&prog, x)?;
        let ok = clauses.iter().all(|c| eval_bool(c, x, r).unwrap_or(false));
        leaves.push(hash(format!("x={x};result={r};ok={ok}").as_bytes()));
        if !ok {
            counterexample = Some((x, r));
            break;
        }
    }
    if let Some((x, r)) = counterexample {
        println!("INVALID counterexample: x={x}, result={r}");
        std::process::exit(3);
    }
    let receipt=format!("AXIOM-PROOF/1\nkind=bounded-exhaustive\nmodule={}\nspec.sha256={}\nprogram.sha256={}\ndomain.min={}\ndomain.max={}\ncases={}\ntrace.merkle={}\nverdict=VALID\n",spec["module"],hex(&hash(spec_raw.as_bytes())),hex(&hash(prog_raw.as_bytes())),min,max,(max-min+1),hex(&merkle(&leaves)));
    fs::write(op, &receipt)?;
    print!("{receipt}");
    Ok(())
}

#[derive(Debug)]
struct Program {
    module: String,
    code: Vec<String>,
}
fn parse_program(raw: &str) -> Result<Program> {
    let mut it = raw.lines();
    if it.next() != Some("AXIOM-PROGRAM/1") {
        bail!("bad program header");
    }
    let mut module = None;
    let mut code = Vec::new();
    let mut in_code = false;
    for l in it {
        let t = l.trim();
        if t == "code:" {
            in_code = true;
            continue;
        }
        if t == "end" {
            break;
        }
        if in_code {
            if !t.is_empty() {
                code.push(t.to_owned())
            }
        } else if let Some(v) = t.strip_prefix("module=") {
            module = Some(v.to_owned())
        }
    }
    Ok(Program {
        module: module.context("missing module")?,
        code,
    })
}
fn run(p: &Program, x: i64) -> Result<i64> {
    let mut r = [0i64; 128];
    for l in &p.code {
        let w: Vec<_> = l.split_whitespace().collect();
        match w.as_slice() {
            ["LOAD_INPUT", dst, "x"] => r[reg(dst)?] = x,
            ["CONST", dst, v] => r[reg(dst)?] = v.parse()?,
            ["NEG", dst, a] => r[reg(dst)?] = r[reg(a)?].wrapping_neg(),
            ["ADD", dst, a, b] => r[reg(dst)?] = r[reg(a)?].wrapping_add(r[reg(b)?]),
            ["SUB", dst, a, b] => r[reg(dst)?] = r[reg(a)?].wrapping_sub(r[reg(b)?]),
            ["SELECT_NEG", dst, a, b] => r[reg(dst)?] = if x < 0 { r[reg(a)?] } else { r[reg(b)?] },
            ["RETURN", a] => return Ok(r[reg(a)?]),
            _ => bail!("unsupported instruction: {l}"),
        }
    }
    bail!("program has no RETURN")
}
fn reg(s: &str) -> Result<usize> {
    let n = s
        .strip_prefix('r')
        .context("expected register")?
        .parse::<usize>()?;
    if n >= 128 {
        bail!("register out of range")
    };
    Ok(n)
}
fn parse_ir(raw: &str) -> Result<BTreeMap<String, String>> {
    let mut it = raw.lines();
    if it.next() != Some("AXIOM-IR/1") {
        bail!("bad IR header");
    }
    let mut m = BTreeMap::new();
    for l in it.filter(|l| !l.trim().is_empty()) {
        let (k, v) = l.split_once('=').context("bad IR line")?;
        m.insert(k.to_owned(), v.to_owned());
    }
    Ok(m)
}
fn ensures(d: &BTreeMap<String, String>) -> Vec<String> {
    d.iter()
        .filter(|(k, _)| k.starts_with("ensures."))
        .map(|(_, v)| v.clone())
        .collect()
}
fn eval_bool(expr: &str, x: i64, result: i64) -> Result<bool> {
    let e = expr.trim();
    if let Some((a, b)) = split_top(e, "||") {
        return Ok(eval_bool(a, x, result)? || eval_bool(b, x, result)?);
    }
    if let Some((a, b)) = split_top(e, "&&") {
        return Ok(eval_bool(a, x, result)? && eval_bool(b, x, result)?);
    }
    for op in ["==", "!=", ">=", "<=", ">", "<"] {
        if let Some((a, b)) = split_top(e, op) {
            let a = eval_int(a, x, result)?;
            let b = eval_int(b, x, result)?;
            return Ok(match op {
                "==" => a == b,
                "!=" => a != b,
                ">=" => a >= b,
                "<=" => a <= b,
                ">" => a > b,
                "<" => a < b,
                _ => unreachable!(),
            });
        }
    }
    if e == "true" {
        return Ok(true);
    }
    if e == "false" {
        return Ok(false);
    }
    bail!("unsupported clause {e}")
}
fn eval_int(expr: &str, x: i64, result: i64) -> Result<i64> {
    let e = expr.trim().trim_matches(|c| c == '(' || c == ')').trim();
    match e {
        "x" => Ok(x),
        "result" => Ok(result),
        "-x" => Ok(x.wrapping_neg()),
        _ => Ok(e.parse()?),
    }
}
fn split_top<'a>(expr: &'a str, op: &str) -> Option<(&'a str, &'a str)> {
    let mut d = 0i32;
    let b = expr.as_bytes();
    let o = op.as_bytes();
    let mut i = 0;
    while i + o.len() <= b.len() {
        match b[i] as char {
            '(' => d += 1,
            ')' => d -= 1,
            _ => {}
        }
        if d == 0 && &b[i..i + o.len()] == o {
            return Some((&expr[..i], &expr[i + o.len()..]));
        }
        i += 1;
    }
    None
}
fn hash(b: &[u8]) -> [u8; 32] {
    Sha256::digest(b).into()
}
fn hex(h: &[u8; 32]) -> String {
    h.iter().map(|b| format!("{b:02x}")).collect()
}
fn merkle(leaves: &[[u8; 32]]) -> [u8; 32] {
    if leaves.is_empty() {
        return hash(b"");
    }
    let mut level = leaves.to_vec();
    while level.len() > 1 {
        let mut next = Vec::new();
        for pair in level.chunks(2) {
            let a = pair[0];
            let b = *pair.get(1).unwrap_or(&a);
            let mut bytes = Vec::with_capacity(64);
            bytes.extend(a);
            bytes.extend(b);
            next.push(hash(&bytes));
        }
        level = next;
    }
    level[0]
}
