# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a Bevy game engine project (`app_1830`) implementing a digital version of the 1830 railroad investment and building board game. Built with Bevy 0.19.0 (Rust edition 2024) and uses the `hexx` library for hexagonal tile management.

The game features a hex-based map where players act as stockholders of railroad corporations, expanding railroads and generating revenue by building track and operating trains.

## Build and Development Commands

```bash
# Run the application (recommended with release for Bevy performance)
cargo run --release

# Run with dynamic linking for faster compile times during development
cargo run --features bevy/dynamic_linking

# Build without running
cargo build --release

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

# Run clippy linter
cargo clippy
```

## Architecture Overview

### Module Structure

- **`src/main.rs`** - Application entry point, window configuration, camera setup, and system registration
- **`src/routemap.rs`** - Hexagonal map management (tiles, layout, coordinate conversion, mouse interaction)
- **`src/gamemodel.rs`** - Game logic for 1830 (players, corporations, trains, phases, resources)

### Key Dependencies

- **Bevy 0.19.0** - Game engine with ECS architecture
- **hexx 0.24.0** - Hexagonal grid library with Bevy integration for tile-based map system

### Bevy Systems Flow

The application registers systems in two schedules:

**Startup systems** (run once, chained in order):
1. `setup_camera` - Spawns 2D camera with `MainCamera` marker
2. `setup_routemap` - Initializes `HexSettings` resource with pointy-top layout (35.0 hex size, inverted Y)
3. `spawn_routemap` - Loads map tiles from `assets/Map/` directory and spawns them as entities

**Update systems** (run every frame):
- `handle_tile_clicks` - Detects mouse clicks, converts screen coordinates to hex coordinates, identifies clicked tiles

### Hex Coordinate System

- Uses axial coordinates (q, r) from the hexx library
- Pointy-top orientation with inverted Y axis
- Map center: F12 at hex coordinate (0, 0)
- Map dimensions: 24 columns (A-K rows) by 11 rows (1-24 columns in game notation)
- Tile naming convention: Row letter + column number (e.g., "A9", "F12", "K15")

### Resource Management

- **`HexSettings`** - Stores hex layout configuration and provides coordinate conversion methods
- **`GameState`** - Tracks current game phase and bank amount (defined but not yet integrated)

### Component Types

- **`MainCamera`** - Marker for the 2D camera entity
- **`HexTile`** - Attached to each map tile entity, stores hex coordinate and tile name
- **`Player`**, **`RailroadCorporation`** - Game entities (defined in gamemodel, not yet used)

### Asset Organization

- `assets/Map/` - Map tile images referenced by tile name (e.g., "A9.png", "blank.png", "blank_mountain.png")
- `assets/green/`, `assets/orange/`, `assets/yellow/` - Additional tile assets (not yet integrated)

### Window Configuration

- Fixed 1780x870 window (100px padding on 1680x770 game area)
- Non-resizable to maintain hex grid alignment
