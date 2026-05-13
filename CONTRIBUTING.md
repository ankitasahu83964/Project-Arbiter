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
*   **Note on AI & LLMs**: Arbiter is a project about human-aware logic. While LLMs can be helpful for boilerplate, I moreso value PRs where the logic and design are clearly human-driven. Please avoid submitting large blocks of unverified AI code; I'd much rather see a smaller, manual PR that you fully understand. Even if you use a tool to help you, you are in charge of every line that goes in and every result that comes out. Use them if they help, but remember that you are the author of the PR, not the model.

---

## Code of Conduct
Please review the [Code of Conduct](CODE_OF_CONDUCT.md) before participating.

---

## Getting Help
If you're stuck, open an issue or reach out to the maintainers (mostly me for now). I'm happy to help you get your environment set up or talk through a technical design!
