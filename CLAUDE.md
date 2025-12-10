# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

TaxStud is a Rust desktop application for managing hybrid taxonomies that combine classical hierarchical classification (genus-species-differentia) with faceted search. Built with Slint UI toolkit for the GUI and organized as a workspace with a shared core library.

## Essential Build Commands

### Development
```bash
# Build and run GUI application
cargo run --release

# Build only (faster iteration during development)
cargo build

# Run with a taxonomy file
cargo run --release -- assets/beverages.json

# Run CLI tool
cargo run --release --example taxstud_cli -- assets/beverages.json
```

### Testing
```bash
# Run all tests
cargo test

# Run tests for core library only
cargo test -p taxstud-core

# Run specific test
cargo test <test_name>
```

### Linting and Formatting
```bash
# Check for lint issues
cargo clippy

# Format code
cargo fmt
```

## Architecture Overview

### Workspace Structure
This is a Cargo workspace with two main components:

1. **`taxstud` (root package)** - GUI application
   - Entry point: `src/main.rs`
   - UI definition: `ui/app-window.slint`
   - Build script: `build.rs` (compiles Slint UI)

2. **`taxstud-core` library** - Shared business logic
   - Located in: `taxstud-core/src/`
   - Used by both GUI and CLI tools
   - Pure Rust (no UI dependencies)

### Core Library Modules (`taxstud-core/src/`)

The core library is organized into focused modules:

- **`models.rs`** - Data structures (`HybridTaxonomy`, `Item`, `ClassicalHierarchy`, `HierarchyNode`, `Filters`)
- **`validation.rs`** - Taxonomy schema validation
- **`filtering.rs`** - Filter logic for genera and facets
- **`sorting.rs`** - Library science sorting (strips articles like "The", "A")
- **`grouping.rs`** - Grouping items by facet dimensions
- **`io.rs`** - File I/O (load/save JSON taxonomies)
- **`lib.rs`** - Public API and re-exports

### GUI Architecture (Slint + Rust)

**Separation of concerns:**
- **Slint files** (`ui/*.slint`) - UI layout, styling, component structure
- **Rust code** (`src/main.rs`) - Business logic, state management, event handlers

**State Management:**
- `AppState` struct manages taxonomy data, current file, dirty flag, filters
- Wrapped in `Rc<RefCell<>>` for shared mutable state across callbacks
- Window state synchronized via `update_ui_from_state()`

**Key patterns:**
- Callbacks registered with `main_window.on_*` methods
- Async file operations use `slint::spawn_local()` with `rfd::AsyncFileDialog`
- UI models use `Rc<VecModel<T>>` for list data

### Data Model

**Core types:**
- `HybridTaxonomy` - Top-level taxonomy with classical hierarchy + faceted dimensions
- `ClassicalHierarchy` - Tree structure (root + optional children nodes)
- `HierarchyNode` - Has genus, species, differentia, and optional children
- `Item` - Taxonomy entry with name, classical_path (Vec<String>), and facets (HashMap)
- `Filters` - Contains genera (Vec<String>) and facets (HashMap<String, Vec<String>>)

**File format:** JSON with structure defined by serde serialization of these types

### Examples Directory

- **`examples/taxstud_cli.rs`** - CLI tool demonstrating core library usage
  - Supports filtering, sorting, grouping operations
  - Good reference for understanding core library API

## UI Framework (Slint)

**Build process:**
- `build.rs` compiles `.slint` files at build time using `slint-build`
- Generated Rust code is imported with `slint::slint!` macro

**Theme system:**
- Supports light/dark themes via `Theme` enum
- Theme state stored in `MainWindow.theme` property
- Colors defined as properties that change based on theme state

**Status system:**
- `StatusMessage` struct with text and `StatusLevel` enum (none, success, info, warning, danger)
- Helper function `set_status()` for consistent status updates

## Sample Data

Located in `assets/` directory:
- `beverages.json` - Beverage taxonomy (coffee, tea, etc.)
- `books.json` - Book taxonomy
- `stocks.json` - Stock/financial taxonomy

These demonstrate the hybrid taxonomy structure and are useful for testing.

## Important Conventions

### Error Handling
- Use `Result<T, Box<dyn std::error::Error>>` for operations that can fail
- Display errors to user via status bar with `StatusLevel::Danger`

### State Mutations
- Always call `state.mark_dirty()` after modifying taxonomy
- Update window title after state changes using `get_window_title()`
- Call `update_ui_from_state()` to refresh UI after modifications

### Filtering Logic
- Genera filters use OR logic (match any genus in list)
- Facet filters use AND logic between dimensions, OR within dimension
- Empty filters match all items
- Use `matches_filters()` to test items against filter criteria

### Sorting
- `sort_items()` strips leading articles ("The", "A", "An") for library science sorting
- Unicode normalization applied for consistent sorting across diacritics

## File Locations

**Critical paths:**
- Main GUI: `src/main.rs`
- UI definition: `ui/app-window.slint`
- Core library: `taxstud-core/src/`
- CLI example: `examples/taxstud_cli.rs`
- Sample taxonomies: `assets/*.json`
