# Taxman - Hybrid Taxonomy Management System

A desktop GUI application for managing hybrid taxonomies that combine classical hierarchical classification with modern faceted search. Built with Rust and [Slint](https://slint.rs/).

## Overview

Taxman provides an intuitive interface for creating, editing, and exploring taxonomies that use both:
- **Classical Hierarchy:** Traditional genus-species-differentia tree structure
- **Faceted Classification:** Multi-dimensional tagging for flexible categorization

Perfect for organizing complex collections like beverages, books, products, or any domain that benefits from both hierarchical and faceted organization.

## Features

### Core Functionality
- ✅ **Full CRUD Operations:** Create, read, update, and delete taxonomy items
- ✅ **File Management:** Open, save, and create taxonomy files (JSON format)
- ✅ **Filtering:** Filter items by genus (classical hierarchy) and facets
- ✅ **Sorting:** Sort items alphabetically with library science rules (strips articles)
- ✅ **Validation:** Real-time validation of taxonomy structure and item data
- ✅ **Dirty State Tracking:** Visual indicator for unsaved changes

### User Interface
- **3-Panel Layout:**
  - Left: Classical hierarchy tree (placeholder for future tree view)
  - Center: Sortable, filterable items list
  - Right: Details panel with edit/create forms
- **Bottom Panel:** Advanced filtering controls
- **Inline Editing:** Edit items directly with validation feedback
- **Status Bar:** Real-time operation feedback

### Architecture
- **Shared Library (`taxman-core`):** Business logic shared between GUI and CLI
- **CLI Tool:** Command-line interface for power users (see `examples/taxman_cli.rs`)
- **Clean Separation:** UI logic in Slint, business logic in Rust

## Installation

### Prerequisites
- Rust 1.70+ ([Install Rust](https://www.rust-lang.org/learn/get-started))
- Cargo (comes with Rust)

### Build from Source
```bash
git clone <repository-url>
cd taxman
cargo build --release
```

### Run
```bash
# GUI application
cargo run --release

# CLI tool
cargo run --release --example taxman_cli -- <taxonomy-file.json>
```

## Quick Start

### Using the GUI

1. **Create a New Taxonomy:**
   - Click "New" in the toolbar
   - Start adding items with "New Item"

2. **Open an Existing Taxonomy:**
   - Click "Open..." and select a JSON file
   - Try the sample files in `assets/` (beverages, books, stocks)

3. **Browse Items:**
   - Select items from the center list
   - View details in the right panel

4. **Filter and Sort:**
   - Use "Sort by Name" for alphabetical ordering
   - Enter genus filters (e.g., "Coffee, Tea")
   - Enter facet filters (e.g., "temperature=hot")
   - Click "Apply Filters"

5. **Edit Items:**
   - Select an item
   - Click "Edit"
   - Modify name, classification path, or facets
   - Click "Save"

6. **Create Items:**
   - Click "New Item"
   - Fill in the form:
     - **Name:** Item display name
     - **Classification Path:** Comma-separated (e.g., "Beverages, Hot Beverages, Coffee")
     - **Facets:** Key=value pairs (e.g., "temperature=hot, caffeine_content=high")
   - Click "Create"

7. **Delete Items:**
   - Select an item
   - Click "Delete"

8. **Save Your Work:**
   - Click "Save" (or "Save As..." for new file)
   - Window title shows `*` when there are unsaved changes

### Using the CLI

```bash
# View entire taxonomy
cargo run --example taxman_cli -- assets/beverages.json

# Filter by genus
cargo run --example taxman_cli -- assets/beverages.json --genus Coffee

# Filter by facets
cargo run --example taxman_cli -- assets/beverages.json --facet temperature=hot

# Combine filters (AND logic between types)
cargo run --example taxman_cli -- assets/beverages.json --genus Coffee --facet caffeine_content=high

# Sort by name
cargo run --example taxman_cli -- assets/beverages.json --sort name

# Group by facet
cargo run --example taxman_cli -- assets/beverages.json --group-by temperature

# Combine all features
cargo run --example taxman_cli -- assets/beverages.json \
  --genus Coffee \
  --facet temperature=hot \
  --sort name \
  --group-by primary_theme
```

## Taxonomy File Format

Taxonomies are stored as JSON files with this structure:

```json
{
  "taxonomy_description": "Beverages Taxonomy",
  "classical_hierarchy": {
    "root": "Beverages",
    "children": [
      {
        "genus": "Hot Beverages",
        "species": "Coffee",
        "differentia": "Coffee-based hot drinks",
        "children": []
      }
    ]
  },
  "faceted_dimensions": {
    "temperature": ["hot", "iced", "room_temp"],
    "caffeine_content": ["high", "medium", "low", "none"],
    "primary_theme": ["coffee", "tea", "water", "juice"]
  },
  "example_items": [
    {
      "name": "Espresso",
      "classical_path": ["Beverages", "Hot Beverages", "Coffee"],
      "facets": {
        "temperature": "hot",
        "caffeine_content": "high",
        "primary_theme": "coffee"
      }
    }
  ]
}
```

## Project Structure

```
taxman/
├── Cargo.toml              # Workspace configuration
├── src/
│   └── main.rs            # GUI application
├── ui/
│   └── app-window.slint   # Main UI layout
├── taxman-core/           # Shared library
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs         # Public API
│       ├── models.rs      # Data structures
│       ├── validation.rs  # Schema validation
│       ├── filtering.rs   # Filter logic
│       ├── sorting.rs     # Library science sorting
│       ├── grouping.rs    # Grouping operations
│       └── io.rs          # File I/O
├── examples/
│   └── taxman_cli.rs      # CLI tool
├── assets/                # Sample taxonomy files
│   ├── beverages.json
│   ├── books.json
│   └── stocks.json
└── NEXT_STEPS.md          # Future development roadmap
```

## Development

### Running Tests
```bash
# Run all tests
cargo test

# Run tests for core library only
cargo test -p taxman-core
```

### Code Style
This project follows standard Rust conventions:
- `cargo fmt` for formatting
- `cargo clippy` for linting

### IDE Setup
Recommended: [Visual Studio Code](https://code.visualstudio.com) with extensions:
- [Slint Extension](https://marketplace.visualstudio.com/items?itemName=Slint.slint)
- [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)

## Completed Development Phases

- ✅ **Phase 1:** Shared Library Creation
- ✅ **Phase 2:** GUI Foundation
- ✅ **Phase 3:** UI Layout Redesign
- ✅ **Phase 4:** Read-Only Display
- ✅ **Phase 5:** Filtering & Sorting
- ✅ **Phase 6:** Edit Operations
- ✅ **Phase 7:** Create & Delete Operations

## Future Enhancements

See [NEXT_STEPS.md](NEXT_STEPS.md) for detailed roadmap including:

### Planned Features
- **Phase 8:** File Management (unsaved changes warnings, recent files, auto-save)
- **Phase 9:** Polish (keyboard shortcuts, tooltips, accessibility)
- **Additional Features:**
  - Undo/Redo functionality
  - Hierarchy tree view (interactive tree navigation)
  - Search/find functionality
  - Confirmation dialogs
  - Batch operations
  - Export to CSV, Markdown, Excel
  - Taxonomy validation reports
  - And more...

## Contributing

This is a personal project, but suggestions and feedback are welcome! Please file issues or pull requests on the repository.

## License

[Add your license here]

## Acknowledgments

- Built with [Slint UI Toolkit](https://slint.rs/)
- File dialogs via [rfd](https://crates.io/crates/rfd)
- Inspired by classical taxonomy and modern faceted search systems

## Contact

[Add your contact information here]

---

**Current Status:** Fully functional CRUD application with filtering, sorting, and validation. Ready for production use with small to medium taxonomies. See NEXT_STEPS.md for planned enhancements.
