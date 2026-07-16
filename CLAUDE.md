# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a Bevy game engine project (`app_1830`) using Bevy 0.19.0 and Rust edition 2024. Bevy is a data-driven game engine built in Rust featuring an Entity Component System (ECS) architecture.

## Build and Development Commands

```bash
# Build the project
cargo build

# Build with optimizations (release mode)
# Note: Bevy compiles much faster in release mode and is recommended for development
cargo build --release

# Run the application
cargo run

# Run with release optimizations (recommended for Bevy)
cargo run --release

# Run with dynamic linking for faster compile times during development
cargo run --features bevy/dynamic_linking

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

- `src/main.rs` - Entry point for the Bevy application
- `Cargo.toml` - Project manifest with Bevy dependency

## Bevy Architecture Notes

- Bevy uses an ECS (Entity Component System) architecture
- Systems are functions that operate on entities with specific components
- The App is the main entry point, where plugins and systems are registered
- Bevy's schedule runs systems in parallel when possible
- Resources are singleton-like data accessible across systems
