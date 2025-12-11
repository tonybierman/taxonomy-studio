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
   - Entry point: `src/main.rs` (~70 lines)
   - UI definition: `ui/app-window.slint`
   - Build script: `build.rs` (compiles Slint UI)

2. **`taxstud-core` library** - Shared business logic
   - Located in: `taxstud-core/src/`
   - Used by both GUI and CLI tools
   - Pure Rust (no UI dependencies)

### GUI Application Module Structure

The GUI follows a clean, modular architecture with clear separation of concerns:

```
src/
├── main.rs                  # Entry point (~70 lines) - initialization only
├── errors/
│   ├── mod.rs
│   └── error_mapper.rs      # User-friendly error messages
├── state/
│   ├── mod.rs
│   ├── app_state.rs         # Domain state (taxonomy, file, dirty flag, filters)
│   └── ui_state.rs          # UI flow state (pending actions, dialogs)
├── operations/
│   ├── mod.rs
│   ├── file_ops.rs          # File I/O orchestration (FileOperations)
│   └── validation.rs        # Input validation functions
├── handlers/
│   ├── mod.rs
│   ├── file_handlers.rs     # File operations (Open, Save, New, Revert)
│   ├── item_handlers.rs     # Item CRUD operations
│   ├── filter_handlers.rs   # Filtering and sorting
│   ├── dialog_handlers.rs   # Dialog response handlers
│   └── ui_handlers.rs       # Theme, about, facet updates
└── ui/
    ├── mod.rs
    ├── dialogs.rs           # Dialog show/hide functions
    ├── formatting.rs        # Display formatting helpers
    ├── types.rs             # Type re-exports from Slint
    └── updates.rs           # UI refresh functions
```

**Key Architecture Principles:**
- **Separation of Concerns**: Domain state (`AppState`) is separate from UI flow state (`UiState`)
- **Handler Pattern**: All UI callbacks are registered via handler modules (file, item, filter, dialog, ui)
- **Operations Layer**: Business logic orchestration (file I/O, validation) is centralized
- **Error Mapping**: User-friendly error messages generated from Rust errors

### Core Library Modules (`taxstud-core/src/`)

The core library is organized into focused modules:

- **`models.rs`** - Data structures (`HybridTaxonomy`, `Item`, `ClassicalHierarchy`, `HierarchyNode`, `Filters`)
- **`validation.rs`** - Taxonomy schema validation
- **`filtering.rs`** - Filter logic for genera and facets
- **`sorting.rs`** - Library science sorting (strips articles like "The", "A")
- **`grouping.rs`** - Grouping items by facet dimensions
- **`io.rs`** - File I/O (load/save JSON taxonomies)
- **`lib.rs`** - Public API and re-exports

### State Management Pattern

**Two separate state structs:**
1. **`AppState`** (in `state/app_state.rs`) - Domain/business state
   - `taxonomy: Option<HybridTaxonomy>` - Currently loaded taxonomy
   - `current_file: Option<PathBuf>` - Path to current file
   - `dirty: bool` - Unsaved changes flag
   - `selected_item: Option<usize>` - Currently selected item index
   - `filters: Filters` - Active genus/facet filters

2. **`UiState`** (in `state/ui_state.rs`) - UI flow state
   - `pending_action: Option<PendingAction>` - Action waiting for confirmation (Open, New)
   - `simple_confirmation_action: Option<SimpleConfirmationAction>` - Simple confirmation actions (Revert)

Both wrapped in `Rc<RefCell<>>` for shared mutable state across async callbacks.

### Handler Registration Pattern

**Main entry point pattern** (`src/main.rs`):
```rust
pub fn main() {
    // 1. Initialize
    let main_window = MainWindow::new().unwrap();
    let app_state = Rc::new(RefCell::new(AppState::new()));
    let ui_state = Rc::new(RefCell::new(UiState::new()));

    // 2. Load command-line file if provided
    if let Some(file_path) = args.file { /* ... */ }

    // 3. Register all handlers (5 registration calls)
    register_item_handlers(&main_window, &app_state);
    register_file_handlers(&main_window, &app_state, &ui_state);
    register_filter_handlers(&main_window, &app_state);
    register_dialog_handlers(&main_window, &app_state, &ui_state);
    register_ui_handlers(&main_window);

    // 4. Run event loop
    main_window.run().unwrap();
}
```

Each handler module has a main `register_*_handlers()` function that internally calls individual registration functions for each callback.

### File Operations Pattern

**FileOperations struct** (`operations/file_ops.rs`) centralizes all file I/O:
- `open_file_dialog_and_load()` - Open file picker and load
- `load_file(path)` - Load from specific path
- `save()` - Save to current file
- `save_as()` - Save to new file with picker
- `revert()` - Reload from saved file

Pattern: Create `FileOperations::new(&app_state, &window)` and call methods. Handles errors, UI updates, and status messages automatically.

### UI Framework (Slint)

**Build process:**
- `build.rs` compiles `.slint` files at build time using `slint-build`
- Generated Rust code is imported with `slint::slint!` macro

**Theme system:**
- Supports light/dark themes via `Theme` enum
- Theme state stored in `MainWindow.theme` property
- Colors defined as properties that change based on theme state

**Key Slint patterns:**
- Callbacks registered with `main_window.on_*` methods
- Async file operations use `slint::spawn_local()` with `rfd::AsyncFileDialog`
- UI models use `Rc<VecModel<T>>` for list data
- Window cloning: Use `window.clone_strong()` for async closures, not `.clone()`

### Data Model

**Core types:**
- `HybridTaxonomy` - Top-level taxonomy with classical hierarchy + faceted dimensions
- `ClassicalHierarchy` - Tree structure (root + optional children nodes)
- `HierarchyNode` - Has genus, species, differentia, and optional children
- `Item` - Taxonomy entry with name, classical_path (Vec<String>), and facets (HashMap)
- `Filters` - Contains genera (Vec<String>) and facets (HashMap<String, Vec<String>>)

**File format:** JSON with structure defined by serde serialization of these types

## Important Conventions

### Adding New Features

**To add a new UI callback:**
1. Create/update handler function in appropriate `handlers/*.rs` module
2. Register it in the module's `register_*_handlers()` function
3. No changes needed to `main.rs` unless adding entirely new handler category

**To add new file operations:**
1. Add method to `FileOperations` struct in `operations/file_ops.rs`
2. Method should handle errors, UI updates, and status messages
3. Use error mapper functions for user-friendly error messages

**To add validation:**
1. Add validation function to `operations/validation.rs`
2. Return `Result<T, ValidationError>` for validation errors
3. ValidationError has `field` and `message` for UI display

### State Mutations

- Always call `state.mark_dirty()` after modifying taxonomy
- Update window title after state changes using `get_window_title()`
- Call `update_ui_from_state()` to refresh UI after modifications
- Use `refresh_ui_after_state_change()` for combined title update + UI refresh + status message

### Error Handling

- Use `Result<T, Box<dyn std::error::Error>>` for operations that can fail
- Use error mapper functions (`map_file_load_error`, `map_file_save_error`, `map_revert_error`) to convert errors to user-friendly dialogs
- Display errors via `show_error()` dialog or status bar with `StatusLevel::Danger`

### Filtering Logic

- Genera filters use OR logic (match any genus in list)
- Facet filters use AND logic between dimensions, OR within dimension
- Empty filters match all items
- Use `matches_filters()` (from taxstud-core) to test items against filter criteria
- Use `parse_facet_filters()` (from taxstud-core) to parse facet filter strings

### Sorting

- `sort_items()` (from taxstud-core) strips leading articles ("The", "A", "An") for library science sorting
- Unicode normalization applied for consistent sorting across diacritics

## Sample Data

Located in `assets/` directory:
- `beverages.json` - Beverage taxonomy (coffee, tea, etc.)
- `books.json` - Book taxonomy
- `stocks.json` - Stock/financial taxonomy

These demonstrate the hybrid taxonomy structure and are useful for testing.

## File Locations

**Critical paths:**
- Main GUI entry point: `src/main.rs` (70 lines)
- Handler modules: `src/handlers/*.rs`
- Operations modules: `src/operations/*.rs`
- State modules: `src/state/*.rs`
- UI helpers: `src/ui/*.rs`
- Error handling: `src/errors/*.rs`
- UI definition: `ui/app-window.slint`
- Core library: `taxstud-core/src/`
- CLI example: `examples/taxstud_cli.rs`
- Sample taxonomies: `assets/*.json`
