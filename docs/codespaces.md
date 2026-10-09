# Working on PdfCraft in GitHub Codespaces

This fork's `.devcontainer/` configuration supplies Rust/Cargo and the Linux
native build dependencies automatically **inside the remote Codespace**.
It does **not** install anything on the computer running the browser.

## Start a new Codespace

1. Open this repository on GitHub.
2. Select **Code → Codespaces → Create codespace on main**.
3. Wait for the configured container to start. Its setup reports the versions of
   `rustc` and `cargo` automatically.

## Update an existing Codespace

After merging the environment setup into `main`:

1. Open your existing PdfCraft Codespace.
2. Save or commit any changes you want to keep, then run `git pull` while on
   `main` so that `.devcontainer/` is present locally.
3. Open the Command Palette (**Ctrl+Shift+P** on Windows).
4. Select **Codespaces: Rebuild Container**, then **Rebuild**.
5. Once the rebuild completes, use the terminal to check:

   ```bash
   rustc --version
   cargo --version
   ```

Rebuilding replaces the container environment but preserves files in
`/workspaces`. Files and tools installed outside that workspace can be lost.
The Git repository remains stored on GitHub.

## First smoke check

Run these in a Codespaces terminal at the repository root (where `Cargo.toml`
is located):

```bash
cargo check -p pdfcraft-cli --no-default-features
cargo test -p pdfcraft-geom
```

These compile/check a command-line component and run focused tests.
They do **not** launch PdfCraft's desktop GUI or install anything on your local computer.

To check the complete desktop app without launching a Linux graphical session:

```bash
cargo check -p pdfcraft
```

Full repository tests can be much more resource-intensive:

```bash
cargo test --workspace
```

The existing `.github/workflows/ci.yml` automatically checks formatting,
Clippy, and tests on pull requests and pushes to `main`, including Windows.
Those jobs use **GitHub Actions**, not the local computer, and may consume
GitHub Actions minutes. Do not confuse a successful `cargo check` with a
finished Windows installer; packaged builds are a separate step.

## Keep resource usage low

GitHub limits simultaneous running Codespaces. Stop (do not delete) an unused
Codespace at https://github.com/codespaces before starting another. Stopping
retains the workspace. Build caches and large Rust dependencies can consume
Codespace disk space, so avoid unnecessary release builds.

## Distribution and branding

The Rust code uses MIT / Apache-2.0 licenses, but ArtCraft marks in
`docs/brand/` are **not** open source. Modified builds/forks intended for
distribution must follow `docs/brand/LICENSE-brand.txt`, including removing
those marks and identifying the fork under its own branding.
