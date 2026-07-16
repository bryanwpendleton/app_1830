# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a Rust project (`app_1830`) using Rust edition 2024. The project was recently initialized and is in early development.

## Build and Development Commands

```bash
# Build the project
cargo build

# Build with optimizations (release mode)
cargo build --release

# Run the application
cargo run

# Run with release optimizations
cargo run --release

# Check code without building
cargo check

# Run tests
cargo test

# Run a specific test
cargo test test_name

# Run tests with output
cargo test -- --nocapture

# Format code
cargo fmt

# Check formatting without applying
cargo fmt -- --check

# Run clippy linter
cargo clippy

# Run clippy with all warnings
cargo clippy -- -W clippy::all
```

## Project Structure

- `src/main.rs` - Entry point for the application
- `Cargo.toml` - Project manifest with dependencies and metadata
