# Solana Voting: Anchor vs. Native

An educational repository demonstrating the architectural differences and compute unit (CU) costs between standard Anchor, Zero-Copy Anchor, and highly optimized Native Solana programs.

This project implements the exact same Voting Smart Contract in three different ways to objectively measure the overhead introduced by different frameworks.

## 🏗️ The Contenders

1. **`anchor_vote` (Standard Anchor):** Idiomatic Anchor implementation using standard `borsh` serialization and standard `#[account]` structures.
2. **`zero_copy_anchor_vote` (Anchor Zero-Copy):** Optimized Anchor implementation using `AccountLoader<'info, T>` and `bytemuck` to avoid serialization overhead.
3. **`native_voter_cheap` (Native + `dummy-sdk`):** A pure native implementation built on top of a custom, ultra-lightweight framework designed specifically for this project.

## 🛠️ Enter `dummy-sdk`

Writing raw `solana-program` code is tedious and unsafe. Anchor solves this but introduces unavoidable global dispatchers, heavy trait initializations, and CPI safety-check overhead. 

`dummy-sdk` is an experimental, minimalist framework. It bridges the gap by providing **Zero-Cost Abstractions**:
- **POD-First (Zero-Copy):** Uses `bytemuck` for instant mapping of account data directly from memory.
- **Opt-in Abstractions:** Provides Anchor-like wrappers (`SignerAccount`, `OwnedAccount`, `InitOwnedAccount`) that compile down to raw `AccountInfo` checks.
- **No Global Router:** Uses a simple `match` statement on a `u64` instruction tag, completely bypassing Anchor's heavy routing.

## ⚡ Performance Benchmarks

Below is the benchmark of the Voting Program implemented in all three ways. 
*(Note: PDA derivations are deterministic across all tests to ensure CU measurements reflect pure framework overhead).*

| Instruction | `dummy-sdk` (Native) | Anchor (Zero-Copy) | Anchor (Standard) | `dummy-sdk` Savings vs Anchor |
| :--- | :--- | :--- | :--- | :--- |
| **Create Pull** | **3,564 CU** | 5,010 CU | 7,903 CU | **-55%** |
| **Create Candidate** | **3,852 CU** | 4,997 CU | 8,654 CU | **-55%** |
| **Voting** | **7,406 CU** | 9,748 CU | 13,460 CU | **-45%** |
| **Close Candidate**| **1,388 CU** | 1,769 CU | 4,370 CU | **-68%** |
| **Close Pull** | **1,002 CU** | 1,289 CU | 2,720 CU | **-63%** |

### Max Batch Throughput 
Solana has a strict 200,000 CU limit per transaction. How many `Voting` instructions (votes) can fit into a single transaction?
- **Anchor Standard:** ~14 votes
- **Anchor Zero-Copy:** ~20 votes
- **`dummy-sdk` (Native): ~27 votes (+35% capacity over Anchor ZC)**

### Understanding the True Framework Overhead
In instructions like `Create Pull`, a CPI call to the System Program to allocate an account takes a fixed ~3,000 CU tax. If we subtract this fixed SVM tax, we see the *true* overhead of the frameworks for parsing, routing, and state initialization:
- `dummy-sdk`: **~564 CU**
- Anchor Zero-Copy: **~2,010 CU** *(3.5x slower)*
- Anchor Standard: **~4,903 CU** *(8.5x slower)*

## Conclusion

While Anchor is the industry standard for a reason (incredible Developer Experience and safety), it comes with a steep compute penalty. For high-frequency protocols, aggregators, or games where every Compute Unit counts, falling back to Native Solana with lightweight zero-copy abstractions (like dummy-sdk) can radically increase your protocol's throughput and lower transaction fees.

## License
MIT