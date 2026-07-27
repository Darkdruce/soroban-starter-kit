# Dev Container & GitHub Codespaces Guide

This repository ships with a fully configured dev container so you can start writing and testing Soroban smart contracts without installing anything locally.

## Pre-installed Tools

| Tool | Details |
|------|---------|
| **Rust** (stable) | Provided by the `mcr.microsoft.com/devcontainers/rust:1` base image |
| **Soroban CLI** (`stellar`) | Installed via `cargo install stellar-cli --features opt` during first-run setup |
| **Docker-in-Docker** | Lets you run `docker compose` commands inside the container |
| **rust-analyzer** | VS Code extension for Rust IntelliSense, inline errors, and auto-formatting |
| **Docker extension** | VS Code extension for managing containers from the sidebar |

Ports **8000** (Stellar RPC) and **8001** (Horizon API) are automatically forwarded so a local Stellar node is reachable from your browser or scripts.

---

## Option 1 — VS Code Dev Containers

### Prerequisites

- [Docker Desktop](https://www.docker.com/products/docker-desktop/) running locally
- [VS Code](https://code.visualstudio.com/) with the [Dev Containers extension](https://marketplace.visualstudio.com/items?itemName=ms-vscode-remote.remote-containers) installed

### Steps

1. Clone the repository:
   ```bash
   git clone https://github.com/Fidelis900/soroban-starter-kit.git
   cd soroban-starter-kit
   ```

2. Open the folder in VS Code:
   ```bash
   code .
   ```

3. When prompted **"Reopen in Container"**, click it. Alternatively, open the Command Palette (`Ctrl+Shift+P` / `Cmd+Shift+P`) and run:
   ```
   Dev Containers: Reopen in Container
   ```

4. VS Code builds the image and runs `scripts/setup.sh` automatically. This takes a few minutes on the first run.

5. Once the terminal prompt appears inside the container, you are ready.

---

## Option 2 — GitHub Codespaces

No local installation required.

1. Navigate to the repository on GitHub.
2. Click **Code → Codespaces → Create codespace on main** (or your branch).
3. GitHub builds the container and runs `scripts/setup.sh` in the background.
4. The browser-based VS Code editor opens when the container is ready.

> **Tip:** The Codespace automatically forwards ports 8000 and 8001. Find them under the **Ports** tab in the terminal panel.

---

## First-Run Verification

Run these commands in the integrated terminal to confirm everything is working:

```bash
# Rust compiler
rustc --version

# Soroban CLI
stellar --version

# Docker daemon (available via Docker-in-Docker)
docker info

# Build the token contract
cd contracts/token && cargo build

# Run all tests
cargo test
```

Expected output for the first two commands looks like:
```
rustc 1.XX.X (...)
stellar 21.X.X (...)
```

---

## Starting a Local Stellar Node

```bash
docker compose up stellar-node
```

The Stellar RPC endpoint will be available at `http://localhost:8000` and the Horizon API at `http://localhost:8001`.

---

## Deploying to Testnet

```bash
./scripts/deploy.sh testnet
```

---

## Troubleshooting

**`stellar` command not found after container starts**
The post-create script may still be running. Wait for it to finish (check the terminal labelled *"postCreateCommand"* in VS Code), then open a new terminal.

**Port forwarding not working in Codespaces**
Open the **Ports** tab, find port 8000 or 8001, right-click, and set visibility to *Public* if you need external access.

**Docker-in-Docker permission error**
Restart the container. The Docker socket is initialised during container startup and occasionally needs a fresh start.
