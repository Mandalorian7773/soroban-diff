# soroban-diff

A differential testing prototype that compares Solidity smart contracts compiled
by [Solang](https://github.com/hyperledger-solang/solang) (targeting the Stellar
Soroban VM) against semantically equivalent contracts written with the
[soroban-sdk](https://github.com/stellar/rs-soroban-sdk), measuring static WASM
metrics and (future) runtime behavioral differences.

---

## What this tool does

`soroban-diff` automates the full compile-and-compare pipeline for a set of
contract pairs:

1. **Compiles** each Solidity contract with `solang compile --target soroban`.
2. **Compiles** the equivalent Rust contract with
   `cargo build --target wasm32-unknown-unknown --release`.
3. **Collects static metrics** per WASM artefact:
   - Binary size in bytes
   - Approximate instruction count (`wasm-objdump -d`)
   - Section count (`wasm-objdump -h`)
4. **Prints a formatted differential report** with deltas between the Solang
   and Rust SDK outputs.

Behavioral equivalence testing (actual contract invocation / return-value
comparison) is marked **PENDING** and requires a running Soroban network via
`stellar-cli`.

---

## Why it exists

This tool is a mentorship prototype built for the
[LFX Mentorship application to Hyperledger Solang](https://github.com/hyperledger-solang/solang/issues)
(issue #74 in the LFDT mentorship repo).

Solang is a Solidity compiler that targets multiple blockchain VMs, including
Stellar's Soroban. One open research question is: *how much overhead does
Solang's general-purpose encoding layer introduce compared to native Rust SDK
contracts?* `soroban-diff` provides a systematic, reproducible way to quantify
that gap — WASM binary size, instruction density, section layout — and lays the
groundwork for full behavioral differential testing.

---

## Prerequisites

| Tool | Purpose | Install |
|------|---------|---------|
| Rust + Cargo | Build the tool and Rust contracts | https://rustup.rs |
| `wasm32-unknown-unknown` target | Cross-compile Rust to WASM | `rustup target add wasm32-unknown-unknown` |
| `solang` | Compile Solidity → Soroban WASM | https://github.com/hyperledger-solang/solang/releases |
| `stellar-cli` | (Future) Deploy & invoke contracts | https://github.com/stellar/stellar-cli |
| `wabt` (`wasm-objdump`) | WASM static analysis | `brew install wabt` / package manager |

> **Rust version:** `soroban-sdk 22` requires **Rust 1.74** or later.
> Run `rustup update stable` to ensure you're on a recent toolchain.

---

## How to run

```bash
# 1. Clone / enter the repo
cd soroban-diff

# 2. Build the CLI tool (verifies the project compiles cleanly)
cargo build

# 3. Run the full differential analysis
cargo run

# 4. (Optional) JSON output flag stub
cargo run -- --json
```

The tool will gracefully print a warning and continue if `solang` or
`wasm-objdump` are not installed, so you can still observe partial results.

---

## Project structure

```
soroban-diff/
├── Cargo.toml                      # Binary crate (not a workspace)
├── README.md
├── contracts/
│   ├── storage/
│   │   ├── solidity/storage.sol    # Solidity: set/get a uint64
│   │   └── rust/                   # Rust SDK equivalent
│   │       ├── Cargo.toml
│   │       └── src/lib.rs
│   └── counter/
│       ├── solidity/counter.sol    # Solidity: increment/get counter
│       └── rust/                   # Rust SDK equivalent
│           ├── Cargo.toml
│           └── src/lib.rs
└── src/
    └── main.rs                     # CLI driver (~280 lines, std-only)
```

---

## Results

> Fill in after running the tool with `solang` and `wabt` installed.

| Contract      | Metric             | Solang | Rust SDK | Delta |
|---------------|--------------------|--------|----------|-------|
| basic_storage | WASM size (bytes)  | —      | —        | —     |
| basic_storage | Instruction count  | —      | —        | —     |
| basic_storage | Section count      | —      | —        | —     |
| counter       | WASM size (bytes)  | —      | —        | —     |
| counter       | Instruction count  | —      | —        | —     |
| counter       | Section count      | —      | —        | —     |

---

## Roadmap

- [ ] Behavioral equivalence via `stellar-cli` invocations
- [ ] `--json` flag for machine-readable output
- [ ] Extend to more complex contract patterns (tokens, governance)
- [ ] CI integration with reproducible WASM size tracking over time

---

## References

- Mentorship issue: https://github.com/hyperledger-solang/solang/issues (issue #74 in LFDT mentorship repo)
- Solang documentation: https://solang.readthedocs.io
- Soroban SDK: https://developers.stellar.org/docs/tools/sdks/library
- WABT (WebAssembly Binary Toolkit): https://github.com/WebAssembly/wabt
