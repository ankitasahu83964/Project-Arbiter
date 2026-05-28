# Contributing to Project Arbiter

First off, thanks for taking an interest in Arbiter! Whether you're here for NSoC or just because you like automation, I'm more than glad to have you here.

Arbiter is built to be lean, secure, and human-aware. What I would want is clean, readable, and performant code. Don't be afraid to question my design decisions, either. It helps me see the flaws in my logic.

---

## Development Environment Setup

Arbiter is written in Rust. You'll need the stable toolchain installed.

1.  **Install Rust**: If you haven't already, get it at [rustup.rs](https://rustup.rs/).
2.  **Dependencies**:
    *   **Slint**: Arbiter Forge is built with Slint. On Windows, you might need the [C++ Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/).
    *   **Win32 APIs**: Arbiter uses low-level Windows hooks, so it must be built and tested on Windows 10/11.
3.  **Build the Project**:
    ```powershell
    cargo build
    ```
4.  **Running Components**:
    *   **App (The Engine)**: `cargo run --package arbiter-app`
    *   **Forge (Management UI)**: `cargo run --package arbiter-forge` (Requires App to be running)
    *   Run the test suite locally before pushing: `cargo test --workspace` `cargo clippy --workspace -- -D warnings`

---

## Contribution Workflow

I'm not big on overly rigid processes, but a little structure helps everyone reading stay sane.

### Workflow
*   Fork the repository. Create a feature branch on your fork (e.g., feature/clipboard-trigger). Submit your Pull Request against the main branch of the upstream Arbiter repository.

### Commit Messages
While my own commit history isn't always spotless, clean commits make it significantly easier for me to review your PRs. If your branch is a mess of 'oops' and 'fix typo' commits, please squash them before submitting so the review process is faster.
*   **Good**: `Fix: resolve path escaping issue in Shell Bridge`
*   **Bad**: `just fixes`, `forgot to add file`, `oops`

### Code Style & Standards
*   **Rationale-Only Comments**: I would like you to follow a strict Rationale-Only policy. Don't comment on *what* the code does (if it's not clear, rewrite the code). Only comment on the *why*, aka the non-obvious design decisions or hardware-specific quirks. And trust me, for these libraries and windows services, you'll never run out of things to say.
*   **Format & Lint**: All code must pass `cargo fmt` and `cargo clippy -- -D warnings`. I rely heavily on the CI pipeline to enforce code style so I don't have to nitpick your formatting during reviews. The build will automatically fail and reject your PR if it catches warnings, so run these checks locally before you push.
*   **Testing**: Tests are mandatory. If you write a new feature, especially involving arbiter-bridge or serialization, you must include isolated unit tests.
*   **Safety**: Never bypass the Signet security layer (path jailing/binary whitelisting) without a very good reason. Arbiter relies on it for much of its functionality, and is by-design made to fail without its strict sandboxing.

### AI & LLM Policy: Total Ownership
Arbiter is a project about human-aware logic. It would be quite weird to build a system centered on human awareness using automated AI slop. Even though LLMs can be helpful for generating basic boilerplate or exploring ideas, I value pull requests where the logic, design, and edge case handling are clearly human-driven.

If you use AI tools to assist your workflow, you must adhere to the following strict rules:
* **You Wrote It Rule**: You are entirely in charge of every single line of code that goes in and every side effect that comes out. Your account made the PR and is thus your responsibility. If your code introduces a bug, memory leak, or security flaw, "AI wrote it" is not an acceptable defense.
* **No Raw LLM Artifacts**: PRs containing conversational LLM commentary (e.g., "Here is a robust implementation of..."), markdown formatting artifacts, or unedited AI comments will be closed immediately.
* **No Automated Self-Reviews**: Do not summon AI bots (like Copilot, etc.) to review your PR or approve your code within this repository.
* **Prefer Small & Manual**: I would much rather review a 50-line, manual PR that you deeply understand than a 500-line AI dump that you barely skimmed and produced in 10 minutes.

**Enforcement**: Maintainer time is a finite resource. If a PR displays obvious signs of unverified AI generation or lacks manual verification, it will be closed and locked immediately without a line-by-line review.

---

## Code of Conduct
Please review the [Code of Conduct](CODE_OF_CONDUCT.md) before participating.

---

## Getting Help
If you're stuck, open an issue or reach out to the maintainers (mostly me for now). I'm happy to help you get your environment set up or talk through a technical design!
