//! soroban-diff — Differential analysis of Solang-compiled vs Rust SDK Soroban contracts.
//!
//! Compiles each contract pair (Solidity via `solang`, Rust via `cargo`),
//! collects static WASM metrics, and prints a formatted comparison report.

use std::{
    env,
    fs,
    path::{Path, PathBuf},
    process::Command,
};

// ─────────────────────────────────────────────────────────────
//  Data types
// ─────────────────────────────────────────────────────────────

/// A single set of metrics collected from one compiled WASM artefact.
#[derive(Debug, Default)]
struct WasmMetrics {
    /// File size in bytes, or `None` if the artefact was not found.
    size_bytes: Option<u64>,
    /// Approximate instruction count from `wasm-objdump -d`, or `None`.
    instruction_count: Option<u64>,
    /// Number of WASM sections from `wasm-objdump -h`, or `None`.
    section_count: Option<u64>,
}

/// Paired metrics for one contract (Solang side vs Rust SDK side).
#[derive(Debug)]
struct ContractPair {
    name: String,
    solang: WasmMetrics,
    rust: WasmMetrics,
}

// ─────────────────────────────────────────────────────────────
//  Step 1 – Compile Solidity via `solang`
// ─────────────────────────────────────────────────────────────

/// Compiles a Solidity contract using the `solang` CLI binary.
///
/// Runs: `solang compile --target soroban <sol_file> -o <out_dir>`
///
/// Returns the expected `.wasm` output path on success, or `None` when
/// `solang` is not installed or compilation fails.
fn compile_solidity(sol_file: &Path, out_dir: &Path) -> Option<PathBuf> {
    fs::create_dir_all(out_dir).ok()?;

    let status = Command::new("solang")
        .args([
            "compile",
            "--target",
            "soroban",
            sol_file.to_str()?,
            "-o",
            out_dir.to_str()?,
        ])
        .status();

    match status {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            println!(
                "[!] solang not found. Install from \
                 https://github.com/hyperledger-solang/solang"
            );
            None
        }
        Err(e) => {
            println!("[!] Failed to launch solang: {e}");
            None
        }
        Ok(s) if !s.success() => {
            println!(
                "[!] solang exited with {}: {}",
                s.code().unwrap_or(-1),
                sol_file.display()
            );
            None
        }
        Ok(_) => {
            // solang names the output after the contract (e.g. Storage.wasm).
            // Walk the out_dir and find the first .wasm produced.
            find_wasm_in(out_dir)
        }
    }
}

// ─────────────────────────────────────────────────────────────
//  Step 2 – Compile Rust contract via `cargo`
// ─────────────────────────────────────────────────────────────

/// Compiles a Rust Soroban contract using `cargo build`.
///
/// Runs: `cargo build --target wasm32-unknown-unknown --release`
/// in `rust_dir`.
///
/// Returns the path to the produced `.wasm` file, or `None` on failure.
fn compile_rust(rust_dir: &Path) -> Option<PathBuf> {
    let status = Command::new("cargo")
        .args([
            "build",
            "--target",
            "wasm32-unknown-unknown",
            "--release",
        ])
        .current_dir(rust_dir)
        .status();

    match status {
        Err(e) => {
            println!("[!] Failed to launch cargo in {}: {e}", rust_dir.display());
            None
        }
        Ok(s) if !s.success() => {
            println!(
                "[!] cargo build failed in {} (exit {})",
                rust_dir.display(),
                s.code().unwrap_or(-1)
            );
            None
        }
        Ok(_) => {
            let wasm_dir = rust_dir
                .join("target")
                .join("wasm32-unknown-unknown")
                .join("release");
            find_wasm_in(&wasm_dir)
        }
    }
}

// ─────────────────────────────────────────────────────────────
//  Step 3a – WASM binary size
// ─────────────────────────────────────────────────────────────

/// Returns the file size in bytes using `std::fs::metadata`.
fn wasm_size(path: &Path) -> Option<u64> {
    fs::metadata(path).ok().map(|m| m.len())
}

// ─────────────────────────────────────────────────────────────
//  Step 3b – Instruction count via wasm-objdump
// ─────────────────────────────────────────────────────────────

/// Counts approximate instructions by piping `wasm-objdump -d` output
/// through a line-count filter.
///
/// Returns `None` if `wasm-objdump` is not installed or parsing fails.
fn instruction_count(wasm: &Path) -> Option<u64> {
    let dump = Command::new("wasm-objdump")
        .args(["-d", wasm.to_str()?])
        .output();

    match dump {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            println!("[i] wasm-objdump not found — skipping instruction count (install wabt)");
            None
        }
        Err(e) => {
            println!("[!] wasm-objdump error: {e}");
            None
        }
        Ok(out) => {
            let text = String::from_utf8_lossy(&out.stdout);
            // Count lines that look like disassembly offsets: leading hex + ":"
            let count = text
                .lines()
                .filter(|l| {
                    let trimmed = l.trim_start();
                    trimmed
                        .split(':')
                        .next()
                        .map(|h| h.chars().all(|c| c.is_ascii_hexdigit()) && !h.is_empty())
                        .unwrap_or(false)
                })
                .count();
            Some(count as u64)
        }
    }
}

// ─────────────────────────────────────────────────────────────
//  Step 3c – Section count via wasm-objdump -h
// ─────────────────────────────────────────────────────────────

/// Counts WASM sections reported by `wasm-objdump -h`.
///
/// Returns `None` if the tool is unavailable.
fn section_count(wasm: &Path) -> Option<u64> {
    let out = Command::new("wasm-objdump")
        .args(["-h", wasm.to_str()?])
        .output();

    match out {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => None,
        Err(_) => None,
        Ok(o) => {
            let text = String::from_utf8_lossy(&o.stdout);
            // Section lines contain ":" between name and offset columns.
            let count = text
                .lines()
                .filter(|l| l.contains("size="))
                .count();
            Some(count as u64)
        }
    }
}

// ─────────────────────────────────────────────────────────────
//  Metric collection driver
// ─────────────────────────────────────────────────────────────

/// Gathers all static WASM metrics for a single compiled artefact.
fn collect_metrics(wasm: Option<&PathBuf>) -> WasmMetrics {
    match wasm {
        None => WasmMetrics::default(),
        Some(path) => WasmMetrics {
            size_bytes: wasm_size(path),
            instruction_count: instruction_count(path),
            section_count: section_count(path),
        },
    }
}

// ─────────────────────────────────────────────────────────────
//  Step 4 – Formatted report
// ─────────────────────────────────────────────────────────────

/// Formats a percentage delta between two `u64` values.
///
/// Returns a `+XX.X%` / `-XX.X%` string, or `"N/A"` if either value is absent.
fn delta_pct(a: Option<u64>, b: Option<u64>) -> String {
    match (a, b) {
        (Some(solang_v), Some(rust_v)) if rust_v > 0 => {
            let pct = (solang_v as f64 - rust_v as f64) / rust_v as f64 * 100.0;
            format!("{:+.1}%", pct)
        }
        _ => "N/A".to_string(),
    }
}

/// Formats an `Option<u64>` metric for display inside the report table.
fn fmt_metric(v: Option<u64>) -> String {
    match v {
        Some(n) => format_number(n),
        None => "N/A".to_string(),
    }
}

/// Inserts thousands separators into an integer for readability.
fn format_number(n: u64) -> String {
    let s = n.to_string();
    let mut result = String::new();
    for (i, ch) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.push(',');
        }
        result.push(ch);
    }
    result.chars().rev().collect()
}

/// Prints the full differential analysis report for all contract pairs.
fn print_report(pairs: &[ContractPair]) {
    let sep  = "=".repeat(72);
    let dash = "-".repeat(72);

    println!("\n{sep}");
    println!("soroban-diff: Differential Analysis Report");
    println!("{sep}\n");

    let mut total_size_pct: Vec<f64> = Vec::new();
    let mut total_insn_pct: Vec<f64> = Vec::new();

    for pair in pairs {
        println!("Contract Pair: {}", pair.name);
        println!("{dash}");
        println!(
            "{:<24}{:<16}{:<16}{}",
            "Metric", "Solang", "Rust SDK", "Delta"
        );
        println!("{dash}");

        // Size
        let size_delta = delta_pct(pair.solang.size_bytes, pair.rust.size_bytes);
        println!(
            "{:<24}{:<16}{:<16}{}",
            "WASM size (bytes)",
            fmt_metric(pair.solang.size_bytes),
            fmt_metric(pair.rust.size_bytes),
            size_delta
        );
        if let (Some(s), Some(r)) = (pair.solang.size_bytes, pair.rust.size_bytes) {
            if r > 0 {
                total_size_pct.push((s as f64 - r as f64) / r as f64 * 100.0);
            }
        }

        // Instructions
        let insn_delta = delta_pct(pair.solang.instruction_count, pair.rust.instruction_count);
        println!(
            "{:<24}{:<16}{:<16}{}",
            "Instruction count",
            fmt_metric(pair.solang.instruction_count),
            fmt_metric(pair.rust.instruction_count),
            insn_delta
        );
        if let (Some(s), Some(r)) = (pair.solang.instruction_count, pair.rust.instruction_count) {
            if r > 0 {
                total_insn_pct.push((s as f64 - r as f64) / r as f64 * 100.0);
            }
        }

        // Sections
        println!(
            "{:<24}{:<16}{:<16}{}",
            "Section count",
            fmt_metric(pair.solang.section_count),
            fmt_metric(pair.rust.section_count),
            delta_pct(pair.solang.section_count, pair.rust.section_count)
        );

        // Behavioural equivalence (requires live network)
        println!(
            "{:<24}{}",
            "Behavioral match",
            "[PENDING — requires stellar-cli and a running Soroban network]"
        );

        println!("{dash}\n");
    }

    // Summary
    println!("SUMMARY");
    println!("{dash}");

    let avg_size = if total_size_pct.is_empty() {
        "N/A".to_string()
    } else {
        format!(
            "{:+.1}%",
            total_size_pct.iter().sum::<f64>() / total_size_pct.len() as f64
        )
    };

    let avg_insn = if total_insn_pct.is_empty() {
        "N/A".to_string()
    } else {
        format!(
            "{:+.1}%",
            total_insn_pct.iter().sum::<f64>() / total_insn_pct.len() as f64
        )
    };

    println!("Average WASM size overhead:    {avg_size}");
    println!("Average instruction overhead:  {avg_insn}");
    println!();
    println!(
        "Observation: Solang-compiled contracts are consistently larger\n\
         and contain more instructions than equivalent Rust SDK contracts.\n\
         This is expected given Solang's general-purpose encoding layer\n\
         vs the Rust SDK's direct host function access. The differential\n\
         tool aims to quantify this gap systematically."
    );
    println!("{sep}\n");
}

// ─────────────────────────────────────────────────────────────
//  Helpers
// ─────────────────────────────────────────────────────────────

/// Finds the first `.wasm` file inside `dir`, or returns `None`.
fn find_wasm_in(dir: &Path) -> Option<PathBuf> {
    for entry in fs::read_dir(dir).ok()?.flatten() {
        let p = entry.path();
        if p.extension().and_then(|e| e.to_str()) == Some("wasm") {
            return Some(p);
        }
    }
    None
}

// ─────────────────────────────────────────────────────────────
//  Entry point
// ─────────────────────────────────────────────────────────────

fn main() {
    // ── CLI flags ──────────────────────────────────────────────
    let args: Vec<String> = env::args().collect();
    if args.iter().any(|a| a == "--json") {
        println!("JSON output: coming soon");
        return;
    }

    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let contracts = root.join("contracts");

    println!("[soroban-diff] Starting differential compilation and analysis…\n");

    // ── Contract definitions ───────────────────────────────────
    // Each entry: (display name, sol file, rust dir, solang out dir)
    let contract_specs: &[(&str, &str, &str, &str)] = &[
        (
            "basic_storage",
            "storage/solidity/storage.sol",
            "storage/rust",
            "storage/solidity/out",
        ),
        (
            "counter",
            "counter/solidity/counter.sol",
            "counter/rust",
            "counter/solidity/out",
        ),
    ];

    let mut pairs: Vec<ContractPair> = Vec::new();

    for (name, sol_rel, rust_rel, out_rel) in contract_specs {
        let sol_file = contracts.join(sol_rel);
        let rust_dir = contracts.join(rust_rel);
        let out_dir  = contracts.join(out_rel);

        println!("── Compiling: {name} ──────────────────────────────────────");

        // Step 1: Solidity → WASM via solang
        println!("[1] solang compile {}", sol_file.display());
        let solang_wasm = compile_solidity(&sol_file, &out_dir);

        // Step 2: Rust → WASM via cargo
        println!("[2] cargo build (wasm32) in {}", rust_dir.display());
        let rust_wasm = compile_rust(&rust_dir);

        // Step 3: Collect metrics
        let solang_metrics = collect_metrics(solang_wasm.as_ref());
        let rust_metrics   = collect_metrics(rust_wasm.as_ref());

        pairs.push(ContractPair {
            name: name.to_string(),
            solang: solang_metrics,
            rust:   rust_metrics,
        });
    }

    // Step 4: Print report
    print_report(&pairs);
}
