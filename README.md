# 🛡️ rust-recon

**A blazing-fast, strictly deterministic static AST analyzer for Solana Anchor smart contracts.**

`rust-recon` is a purely local Rust Command Line Interface (CLI) that parses your Solana Anchor source code and extracts hard, indisputable facts about the protocol's architecture. It serves as an infallible ground-truth engine for solo-auditors, bug hunters, and security researchers.

---

##  Why `rust-recon`? (The Anti-AI Approach)

If you've used AI (LLMs) to audit or summarize smart contracts, you know the problem: **AI hallucinates.** It invents PDAs, assumes access controls exist when they don't, and misreads complex CPI logic.

`rust-recon` is different:
*   **100% Deterministic:** It uses the `syn` crate to parse the actual Rust Abstract Syntax Tree (AST). If it's in the code, it's in the output. If it's not, it's not.
*   **Zero API Keys Needed:** Everything runs entirely locally on your machine. No data is sent to the cloud.
*   **Perfect Synergy with AI:** By feeding `rust-recon`'s deterministic JSON outputs (`facts.json`, `summary.json`) into LLMs (via our [rust-recon-skill](https://github.com/NVN404/rust-recon-skill)), you force the AI to write reports based strictly on mathematically verified facts, eliminating hallucination.

##  What Our Tool Does

When you run `rust-recon` in an Anchor directory, it surgically extracts:
1.  **Instruction Surface:** Every parameter, account constraint, signer requirement, and mutable state.
2.  **Account & PDA Catalogue:** Exact seed structures, bump allocations, and space requirements.
3.  **Cross-Program Invocations (CPIs):** Detects `token::transfer`, `system_program` calls, etc.
4.  **Security Flags:** Automatically flags high-risk patterns like `UncheckedAccount`, `init_if_needed`, `mut`, and complex arithmetic arrays natively.
5.  **Error Code Registry:** Pulls all custom errors mapped across the project.

It dumps this directly into an organized `.rust-recon/` directory containing `scope.json`, `facts.json`, and `summary.json`.

---

##  Generated Reports (Via Claude Skill)

When paired with the local **Claude Skill orchestrator**, `rust-recon` powers the generation of beautifully formatted, hallucination-free Markdown reports. 

You can configure the skill to output:
- **Detailed Reports:** Comprehensive, 1000+ line breakdowns of every single parameter, account struct, and trust assumption model.
- **Condensed Reports:** High-level summary of exact instruction numbers, error counts, high-risk security flags, and the attack surface.

---

## 💻 Installation Guide

**Prerequisites:**
Ensure you have the Rust toolchain installed.
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

**Install from Source (Global):**
```bash
git clone https://github.com/NVN404/rust-recon.git ~/.rust-recon_tool
cd ~/.rust-recon_tool
cargo install --path cli
```

Verify installation:
```bash
rust-recon --version
```

---

## 🚀 Usage (The Zero-Friction Way)

The beauty of `rust-recon` is that **you never have to run terminal commands manually.** The entire process is orchestrated through our Claude Skill.

1. Install the [rust-recon-skill](https://github.com/NVN404/rust-recon-skill) into your Claude Desktop/environment.
2. Open Claude in your Solana Anchor workspace.
3. Simply type: `/recon`

**That's it.** Claude will automatically:
- Download and install the `rust-recon` engine globally in the background.
- Run the AST extraction (`scope` and `facts`).
- Read the generated JSONs.
- Spit out a completely mathematically verified security report.

*(Note: If you are a developer integrating the tool into a CI/CD pipeline, you can manually run `rust-recon scope` and `rust-recon facts` to get the raw JSON outputs.)*

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

## 📄 License

This project is licensed under the **MIT License**. Check the `LICENSE` file for more details. 

You are free to use, modify, and distribute this software in personal and commercial projects.
