# 🛡️ rust-recon

**A blazing-fast, strictly deterministic static AST analyzer for Solana Anchor smart contracts.**

`rust-recon` is a purely local Rust Command Line Interface (CLI) tool that parses your Solana Anchor source code and extracts hard, indisputable facts about the protocol's architecture. It serves as an infallible ground-truth engine for solo-auditors, bug hunters, and security researchers.

**For formatted recon reports**, pair this tool with the **[rust-recon](https://github.com/NVN404/rust-recon)** Custom Skill (works with Claude, Copilot, Cursor, Codex, and other AI agents) to generate comprehensive 9-section markdown reports via `/recon` command.

---

##  Why `rust-recon`? (The Anti-Hallucination Approach)

If you've used AI (LLMs) to analyze smart contracts, you know the problem: **AI hallucinates.** It invents PDAs, misses account constraints, and confuses the trust model.

`rust-recon` is different:
*   **100% Deterministic:** It uses the `syn` crate to parse the actual Rust Abstract Syntax Tree (AST). If it's in the code, it's in the output. If it's not, it's not.
*   **Zero API Keys Needed:** Everything runs entirely locally on your machine. No data is sent to the cloud.
*   **Perfect Synergy with AI:** By feeding `rust-recon`'s deterministic JSON outputs (`global_facts.json`, `facts/index.json`, per-instruction facts files, `summary.json`) into any AI agent (via our [rust-recon](https://github.com/NVN404/rust-recon) Skill), you force the AI to write recon reports based strictly on mathematically verified facts, eliminating hallucination.

##  What Our Tool Does

When you run `rust-recon` in an Anchor directory, it surgically extracts:
1.  **Instruction Surface:** Every parameter, account constraint, signer requirement, and mutable state.
2.  **Account & PDA Catalogue:** Exact seed structures, bump allocations, and space requirements.
3.  **Cross-Program Invocations (CPIs):** Detects `token::transfer`, `system_program` calls, etc.
4.  **Recon Signals (Parser Metadata):** Aggregates extracted patterns (for example `UncheckedAccount`, `init_if_needed`, unchecked arithmetic, and `remaining_accounts`) into structured flags for downstream report generation.
5.  **Error Code Registry:** Pulls all custom errors mapped across the project.

It dumps this directly into an organized `.rust-recon/` directory containing:
- `scope.json`
- `global_facts.json`
- `facts/index.json`
- `facts/NN_instruction-name.json` (one file per instruction)
- `summary.json`
- `facts.json` (legacy compatibility file)

---

##  Generated Recon Reports (Via rust-recon Skill)

`rust-recon` by itself generates **raw JSON files** (`scope.json`, split facts files, `summary.json`, plus legacy `facts.json`).

To convert these into beautifully formatted **recon reports**, use the [rust-recon](https://github.com/NVN404/rust-recon) Custom Skill with your AI agent (Claude, Copilot, Cursor, Codex, etc.), which generates strict 9-section markdown reports:

- **Detailed Reports (default):** Full per-instruction 2a-2f schema with strict table headers and section ordering
- **Condensed Reports:** Summary-first layout for quick snapshots on larger codebases

The skill is recon-first: Section 2 instruction tables do not use `Risk` or `Severity` columns.

Formatting behavior (for example, Section 2 account presentation rules such as table + fact-card style) is defined in the skill repository and may evolve independently from this CLI tool.

See the **[rust-recon Skill README](https://github.com/NVN404/rust-recon)** for installation and usage.

---

##  Repository Architecture

`rust-recon` is split into **two separate repositories**:

| Repository | Purpose | Contains |
|---|---|---|
| **rust-recon-tool** (this repo) | Pure Rust CLI tool | Source code, AST parser, JSON extraction logic |
| **[rust-recon](https://github.com/NVN404/rust-recon)** | AI Custom Skill + report generation | Orchestrator, markdown rules, examples, references (works with Claude, Copilot, Cursor, Codex, etc.) |

**Why separate?**
- Claude Skills can't bundle compiled Rust binaries
- Keeps the tool pure and portable for CI/CD use
- Skill repo stays lightweight and easy to customize

---

##  Installation Guide

### Prerequisites
Ensure you have the Rust toolchain installed:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### Install the Tool
Clone and install globally:
```bash
git clone https://github.com/NVN404/rust-recon-tool ~/.rust-recon_tool
cd ~/.rust-recon_tool
cargo install --path cli
```

Verify:
```bash
rust-recon --version
```

### (Recommended) Install the Custom Skill
To use the tool with your AI agent and generate formatted reports, also install the **[rust-recon](https://github.com/NVN404/rust-recon)** Skill:

```bash
git clone https://github.com/NVN404/rust-recon ~/.rust-recon-skill
```

Then, link it to your AI agent (see [skill README](https://github.com/NVN404/rust-recon) for agent-specific instructions) and type: `/recon`

(Works with Claude, Copilot, Cursor, Codex, and other agents supporting custom skills.)

---

##  Usage (Two Paths)

###  Universal Command Pattern

**All AI agents use the same command syntax:**

```
[agent-prefix] recon [format]
```

Where **[format]** is optional: `condensed` or `detailed`. If omitted, the skill defaults to `detailed`.

| Agent | Default (Detailed) | Condensed | Detailed (Explicit) |
|-------|---------------------|-----------|----------------------|
| **Claude** | `/recon` | `/recon condensed` | `/recon detailed` |
| **Copilot** | `@rust-recon` | `@rust-recon condensed` | `@rust-recon detailed` |
| **Cursor** | `@rust-recon` | `@rust-recon condensed` | `@rust-recon detailed` |
| **Others** | See [skill README](https://github.com/NVN404/rust-recon) | — | — |

---

### The 2-Step Auditing Workflow (For every project)

When you start auditing a new Anchor project (e.g., `MissionX`), you only need to run two commands. 

**Step 1: Initialize the project (Terminal)**
```bash
cd MissionX
rust-recon facts
```
*(This extracts the AST JSON data and automatically drops hidden AI pointers into the project so your sandboxed AI agent knows the skill exists).*

**Step 2: Generate the report (AI Agent)**
Open your AI agent (Copilot, Cursor, or Claude) in the `MissionX` folder and type:
```
@rust-recon detailed
```
*(The AI agent automatically reads the pointers, fetches your global skill instructions, processes split facts outputs, and generates the 9-section markdown report.)*

---

### Fallback: Direct Tool Usage (Manual)

If you prefer to use the tool directly (or integrate it into CI/CD), install it globally:

```bash
git clone https://github.com/NVN404/rust-recon-tool ~/.rust-recon_tool
cd ~/.rust-recon_tool
cargo install --path cli
```

Then, in any Anchor workspace:

```bash
# 1. Discover the project scope and programs
rust-recon scope

# 2. Extract the deep AST facts
rust-recon facts

# 3. Check the generated .rust-recon/ folder for JSON outputs
cat .rust-recon/global_facts.json
cat .rust-recon/facts/index.json
```

**Note:** Using the tool directly gives you raw JSON outputs. To generate formatted recon reports, use the [rust-recon](https://github.com/NVN404/rust-recon) Custom Skill with your AI agent or pipe the JSON into your own report generation pipeline.

---

##  Known Installation Setbacks & Troubleshooting

If you run into issues during installation or execution:

1.  **`cargo install` fails with unresolved dependencies:** 
    Ensure you are using the latest stable Rust toolchain. Run `rustup update stable` and try again.
2.  **`rust-recon: command not found`:**
    Make sure `~/.cargo/bin` is in your system `$PATH`. 
    Add `export PATH="$HOME/.cargo/bin:$PATH"` to your `.bashrc` or `.zshrc`.
3.  **Fails to run on a project:**
    `rust-recon` strictly requires a standard Anchor workspace. Verify that you are running the command from the directory containing `Anchor.toml`. It does not currently support native (non-Anchor) Solana programs.

---

## 🤝 Contributing

We welcome contributions! If you'd like to help improve the AST extraction, add macro support, or refine the JSON schemas:
1. Fork the repository.
2. Create a new feature branch (`git checkout -b feature/new-extraction`).
3. Commit your changes (`git commit -m 'Add new extraction rule'`).
4. Push to the branch (`git push origin feature/new-extraction`).
5. Open a Pull Request.

---

##  License

This project is licensed under the **MIT License**. Check the `LICENSE` file for more details. 

You are free to use, modify, and distribute this software in personal and commercial projects.
