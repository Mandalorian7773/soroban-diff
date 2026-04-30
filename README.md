# soroban-diff

> Differential analysis tool comparing Hyperledger Solang vs
> Soroban Rust SDK — built for LFX Mentorship 2026 application.

## What This Does

soroban-diff compiles equivalent smart contracts through two pipelines —
Solidity via Solang targeting Soroban, and Rust via soroban-sdk — then
compares WASM binary size, instruction count, section structure, and
behavioral output.

This directly prototypes the differential testing tool described in the
LFX mentorship: **"Improving Hyperledger Solang Through Comparative
Analysis with the Soroban Rust SDK"**  
<https://github.com/hyperledger-solang/solang>

---

## Prerequisites

| Tool | Purpose | Install |
|------|---------|---------|
| Rust + Cargo | Build tool + Rust contracts | <https://rustup.rs> |
| `wasm32-unknown-unknown` target | Cross-compile Rust → WASM | `rustup target add wasm32-unknown-unknown` |
| `solang` v0.3.4+ | Compile Solidity → Soroban WASM | <https://github.com/hyperledger-solang/solang/releases> |
| `wabt` (`wasm-objdump`) | Static WASM analysis | `brew install wabt` |
| `stellar-cli` | Behavioral verification (optional) | `brew install stellar-cli` |

> **Rust version:** soroban-sdk 22 requires **Rust ≥ 1.74**.
> Run `rustup update stable` if needed.

---

## Run

```bash
cargo run               # full differential analysis
cargo run -- --json     # JSON output stub (coming soon)
```

---

## Real Results (v0.1.0 baseline)

| Contract      | Metric            | Solang | Rust SDK | Delta   |
|---------------|-------------------|--------|----------|---------|
| basic_storage | WASM size (bytes) | 866    | 966      | -10.4%  |
| basic_storage | Instructions      | 71     | 137      | -48.2%  |
| basic_storage | Sections          | 13     | 10       | +30.0%  |
| counter       | WASM size (bytes) | 1,019  | 946      | +7.7%   |
| counter       | Instructions      | 110    | 136      | -19.1%  |
| counter       | Sections          | 13     | 10       | +30.0%  |

---

## Key Findings

**1. Instruction count: Solang < Rust SDK**  
Solang produces significantly fewer WASM instructions (-48% for storage,
-19% for counter). This is counterintuitive. It may reflect genuine
optimisation, or missing code paths in Solang's Soroban codegen.
Requires behavioral verification to distinguish.

**2. Section overhead: Solang consistently +30%**  
Both Solang contracts produce 13 WASM sections vs 10 for Rust SDK. The 3
extra sections contain contract spec and metadata emitted by Solang's
Soroban codegen. This overhead affects module instantiation cost even
when instruction count appears lower.

**3. Implicit storage type behavior**  
Solang warns: *"storage type not specified, defaulting to persistent"*.
The Rust SDK requires explicit storage type declaration (`persistent`,
`temporary`, or `instance`). This silent default could affect TTL and
rent costs in production — a behavioral divergence worth documenting.

**4. Behavioral equivalence: not yet verified**  
Functional correctness under identical inputs requires `stellar-cli` and
a running Soroban network. This is the next milestone.

---

## Architecture

```
soroban-diff/
├── contracts/
│   ├── storage/      # get/set uint64 — basic storage pattern
│   ├── counter/      # increment/get — stateful counter pattern
│   └── auth/         # requireAuth() — authentication pattern
└── src/main.rs       # compilation harness + metrics engine (~310 lines)
```

---

## Next Steps (full mentorship scope)

- [ ] Soroban budget metering (CPU instructions + memory bytes)
- [ ] Host function call count via soroban-env-host recording mode
- [ ] Behavioral equivalence verification via `stellar-cli`
- [ ] Cross-contract call overhead analysis
- [ ] Recommendations for Solang storage model improvements
