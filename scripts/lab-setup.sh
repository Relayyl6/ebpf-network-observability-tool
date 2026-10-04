#!/bin/bash
set -e

echo "[*] Setting up eBPF Network Observability Lab..."
# Targetting Ubuntu/Debian environments typical for eBPF dev
sudo apt-get update
sudo apt-get install -y \
    clang \
    llvm \
    libelf-dev \
    libpcap-dev \
    build-essential \
    libc6-dev-i386 \
    linux-tools-common \
    linux-tools-generic \
    iperf3

echo "[*] Installing Rust toolchain..."
if ! command -v cargo &> /dev/null; then
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source $HOME/.cargo/env
fi

echo "[*] Installing bpf-linker for Aya..."
cargo install bpf-linker

echo "[*] Setup complete! You can now build the project."
