# TaxStud Refactoring Plan: Separation of Concerns

## Executive Summary

The current architecture suffers from a "God Object" anti-pattern with all 1,466 lines of application logic in a single `main.rs` file. This plan refactors the codebase into a clean, modular architecture with clear separation of concerns.

**Key Issues:**
- 26 UI callbacks mixed with business logic, I/O, and error handling
- Error mapping duplicated 6 times
- File loading logic duplicated 3 times
- AppState manages both domain data AND UI flow state
- Unused `taxstud-core` functions (filtering logic re-implemented locally)

**Target Architecture:** 5-6 focused modules with single responsibilities

---

## Proposed Module Structure

```
src/
├── main.rs                    # Entry point + app initialization (~150 lines)
├── state/
│   ├── mod.rs                # State module exports
│   ├── app_state.rs          # Domain state only (taxonomy, file, dirty flag)
│   └── ui_state.rs           # UI-specific state (dialogs, pending actions)
├── handlers/
│   ├── mod.rs                # Handler module exports
│   ├── file_handlers.rs      # File operations callbacks
│   ├── item_handlers.rs      # Item CRUD callbacks
│   ├── filter_handlers.rs    # Filtering/sorting callbacks
│   └── dialog_handlers.rs    # Confirmation dialog callbacks
├── operations/
│   ├── mod.rs                # Operations module exports
│   ├── file_ops.rs           # File I/O orchestration
│   ├── item_ops.rs           # Item operations
│   └── validation.rs         # Input validation
├── ui/
│   ├── mod.rs                # UI module exports
│   ├── dialogs.rs            # Dialog management
│   ├── updates.rs            # UI update helpers
│   └── formatting.rs         # Display formatting
└── errors/
    ├── mod.rs                # Error module exports
    └── error_mapper.rs       # Error to user message mapping
```

---

## Phase 1: Extract Error Handling (Low Risk)

**Goal:** Create reusable error mapping functions

**Why First:** No dependencies, immediately reduces 72+ lines of duplication

### 1.1 Create `src/errors/mod.rs`
```rust
pub mod error_mapper;
pub use error_mapper::{map_file_load_error, map_file_save_error};
```

### 1.2 Create `src/errors/error_mapper.rs`

**Extract from:** Lines with error mapping pattern (6 locations)

**Functions to create:**
```rust
pub fn map_file_load_error(
    error: &dyn std::error::Error,
    path: &Path
) -> (String, String, String) {
    // Returns (title, message, details)
}

pub fn map_file_save_error(
    error: &dyn std::error::Error,
    path: &Path
) -> (String, String, String) {
    // Returns (title, message, details)
}

pub fn map_revert_error(
    error: &dyn std::error::Error,
    path: &Path
) -> (String, String, String) {
    // Returns (title, message, details)
}
```

**Replace in main.rs:**
- Lines 313-324 (on_file_open error handling)
- Lines 351-362 (on_file_save error handling)
- Lines 396-405 (on_file_save_as error handling)
- Lines 1007-1018 (on_confirmation_save error handling)
- Lines 1089-1100 (on_confirmation_dont_save error handling)
- Lines 1246-1257 (on_simple_confirmation_ok error handling)

**Expected Reduction:** ~72 lines → ~18 lines (4x compression)

---

## Phase 2: Extract UI Update Helpers (Low Risk)

**Goal:** Standardize UI refresh patterns

**Why Second:** Creates foundation for other handlers, no state dependencies

### 2.1 Create `src/ui/mod.rs`
```rust
pub mod updates;
pub mod dialogs;
pub mod formatting;

pub use updates::{refresh_ui_after_state_change, update_window_title};
pub use dialogs::{show_error, show_confirmation, show_simple_confirmation};
pub use formatting::{format_facets, format_facet_dimensions, create_facet_inputs};
```

### 2.2 Create `src/ui/updates.rs`

**Move from main.rs:**
- `refresh_ui_after_state_change()` (lines 1290-1305)
- `update_ui_from_state()` (lines 1309-1364)

**Add new functions:**
```rust
pub fn update_window_title(
    window: &MainWindow,
    state: &AppState
) {
    let title = state.get_window_title();
    window.set_window_title(SharedString::from(title));
}

pub fn refresh_after_file_load(
    window: &MainWindow,
    state: &AppState,
    message: impl Into<SharedString>
) {
    update_window_title(window, state);
    update_ui_from_state(window, state);
    set_status(window, message, StatusLevel::Success);
}
```

**Expected Reduction:** ~32 lines of duplication eliminated

### 2.3 Create `src/ui/formatting.rs`

**Move from main.rs:**
- `format_facets()` (lines 1367-1387)
- `format_facet_dimensions()` (lines 1390-1401)
- `create_facet_inputs()` (lines 1404-1431)
- `flatten_hierarchy()` (lines 1435-1444)
- `flatten_node()` (lines 1447-1466)

**Expected Reduction:** ~95 lines moved to focused module

### 2.4 Create `src/ui/dialogs.rs`

**Move from main.rs:**
- `set_status()` (lines 11-16)
- `show_confirmation()` (lines 18-24)
- `hide_confirmation()` (lines 26-29)
- `show_error()` (lines 31-43)
- `hide_error()` (lines 45-48)
- `show_simple_confirmation()` (lines 50-61)
- `hide_simple_confirmation()` (lines 63-66)

**Add new orchestration:**
```rust
pub struct DialogManager<'a> {
    window: &'a MainWindow,
}

impl<'a> DialogManager<'a> {
    pub fn new(window: &'a MainWindow) -> Self {
        Self { window }
    }

    pub fn show_error_from_result<T, E: std::error::Error>(
        &self,
        result: Result<T, E>,
        operation: &str
    ) -> Option<T> {
        // Handle Result, show error dialog if Err
    }
}
```

**Expected Reduction:** ~56 lines moved + better abstraction

---

## Phase 3: Split AppState (Medium Risk)

**Goal:** Separate domain state from UI flow state

**Why Third:** Required before handler extraction, affects state borrowing

### 3.1 Create `src/state/mod.rs`
```rust
pub mod app_state;
pub mod ui_state;

pub use app_state::AppState;
pub use ui_state::{UiState, PendingAction, SimpleConfirmationAction};
```

### 3.2 Create `src/state/app_state.rs`

**Domain State Only:**
```rust
pub struct AppState {
    pub taxonomy: Option<HybridTaxonomy>,
    pub current_file: Option<PathBuf>,
    pub dirty: bool,
    pub selected_item: Option<usize>,
    pub filters: Filters,
}

impl AppState {
    pub fn new() -> Self { ... }
    pub fn mark_dirty(&mut self) { ... }
    pub fn get_window_title(&self) -> String { ... }
}
```

**Remove from AppState:**
- File I/O methods (load_from_file, save, save_as) → Move to operations module
- UI state (pending_action, simple_confirmation_action) → Move to UiState

### 3.3 Create `src/state/ui_state.rs`

**UI Flow State:**
```rust
pub enum PendingAction {
    Open,
    New,
}

pub enum SimpleConfirmationAction {
    Revert,
}

pub struct UiState {
    pub pending_action: Option<PendingAction>,
    pub simple_confirmation_action: Option<SimpleConfirmationAction>,
}

impl UiState {
    pub fn new() -> Self {
        Self {
            pending_action: None,
            simple_confirmation_action: None,
        }
    }

    pub fn set_pending(&mut self, action: PendingAction) {
        self.pending_action = Some(action);
    }

    pub fn take_pending(&mut self) -> Option<PendingAction> {
        self.pending_action.take()
    }
}
```

**Update main.rs:**
- Change state management to use two RefCells:
  ```rust
  let app_state = Rc::new(RefCell::new(AppState::new()));
  let ui_state = Rc::new(RefCell::new(UiState::new()));
  ```
- Update all callbacks to use appropriate state

**Expected Impact:** Better separation but requires updating all 26 callbacks

---

## Phase 4: Extract File Operations (Medium Risk)

**Goal:** Centralize file I/O orchestration

**Why Fourth:** Eliminates 3x duplication of file loading logic

### 4.1 Create `src/operations/mod.rs`
```rust
pub mod file_ops;
pub mod item_ops;
pub mod validation;

pub use file_ops::{FileOperations, LoadResult};
pub use item_ops::ItemOperations;
pub use validation::{validate_item_input, ValidationError};
```

### 4.2 Create `src/operations/file_ops.rs`

**Purpose:** Orchestrate file operations with proper error handling

```rust
use crate::errors::error_mapper;
use crate::state::AppState;
use crate::ui::{dialogs::DialogManager, updates};

pub struct FileOperations<'a> {
    state: &'a Rc<RefCell<AppState>>,
    window: &'a MainWindow,
}

impl<'a> FileOperations<'a> {
    pub fn new(state: &'a Rc<RefCell<AppState>>, window: &'a MainWindow) -> Self {
        Self { state, window }
    }

    pub async fn open_file_dialog_and_load(&self) {
        if let Some(file) = rfd::AsyncFileDialog::new()
            .add_filter("JSON", &["json"])
            .set_title("Open Taxonomy File")
            .pick_file()
            .await
        {
            self.load_file(file.path()).await;
        }
    }

    pub async fn load_file(&self, path: &Path) {
        match load_taxonomy(path) {
            Ok(taxonomy) => {
                // Validate
                if let Err(errors) = validate_taxonomy(&taxonomy) {
                    let details = errors.join("\n");
                    show_error(self.window, "Validation Error",
                              "The taxonomy file has validation errors.",
                              details);
                    return;
                }

                // Update state
                let mut state = self.state.borrow_mut();
                state.taxonomy = Some(taxonomy);
                state.current_file = Some(path.to_path_buf());
                state.dirty = false;
                state.selected_item = None;
                drop(state);

                // Update UI
                updates::refresh_after_file_load(
                    self.window,
                    &self.state.borrow(),
                    "File loaded successfully"
                );
            }
            Err(e) => {
                let (title, msg, details) = error_mapper::map_file_load_error(&*e, path);
                show_error(self.window, title, msg, details);
            }
        }
    }

    pub fn save(&self) -> Result<(), String> {
        let state = self.state.borrow();
        let taxonomy = state.taxonomy.as_ref()
            .ok_or("No taxonomy to save")?;
        let path = state.current_file.as_ref()
            .ok_or("No file path set")?;

        match save_taxonomy(taxonomy, path) {
            Ok(_) => {
                drop(state);
                self.state.borrow_mut().dirty = false;
                Ok(())
            }
            Err(e) => Err(e.to_string())
        }
    }

    pub fn revert(&self) {
        let path = self.state.borrow().current_file.clone();

        if let Some(file_path) = path {
            // Use load_file logic
            slint::spawn_local({
                let ops = FileOperations::new(self.state, self.window);
                async move {
                    ops.load_file(&file_path).await;
                }
            });
        }
    }
}
```

**Replace in main.rs:**
- `on_file_open` logic (lines 314-354) → `ops.open_file_dialog_and_load().await`
- `on_confirmation_save` open logic (lines 1049-1081) → `ops.open_file_dialog_and_load().await`
- `on_confirmation_dont_save` open logic (lines 1131-1163) → `ops.open_file_dialog_and_load().await`
- `on_simple_confirmation_ok` revert logic (lines 1229-1259) → `ops.revert()`

**Expected Reduction:** ~120 lines → ~30 lines (4x compression)

### 4.3 Create `src/operations/validation.rs`

**Purpose:** Validate user inputs before applying to state

```rust
pub struct ValidationError {
    pub field: String,
    pub message: String,
}

pub fn validate_item_input(
    name: &str,
    path_str: &str,
) -> Result<(String, Vec<String>), ValidationError> {
    // Validate name
    if name.trim().is_empty() {
        return Err(ValidationError {
            field: "name".to_string(),
            message: "Name cannot be empty".to_string(),
        });
    }

    // Parse and validate path
    let path = parse_classification_path(path_str)?;

    Ok((name.trim().to_string(), path))
}

pub fn parse_classification_path(path_str: &str) -> Result<Vec<String>, ValidationError> {
    let path: Vec<String> = path_str
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    if path.is_empty() {
        return Err(ValidationError {
            field: "path".to_string(),
            message: "Classification path cannot be empty".to_string(),
        });
    }

    Ok(path)
}

pub fn collect_facets(
    facet_inputs: &[FacetInput]
) -> HashMap<String, serde_json::Value> {
    facet_inputs.iter()
        .filter_map(|input| {
            let value = input.value.to_string();
            if !value.trim().is_empty() {
                Some((
                    input.name.to_string(),
                    serde_json::Value::String(value.trim().to_string())
                ))
            } else {
                None
            }
        })
        .collect()
}
```

**Replace in main.rs:**
- `on_save_edit` validation (lines 686-701) → `validate_item_input()`
- `on_save_new_item` validation (lines 822-837) → `validate_item_input()`
- `on_save_edit` facet collection (lines 704-714) → `collect_facets()`
- `on_save_new_item` facet collection (lines 841-849) → `collect_facets()`

**Expected Reduction:** ~40 lines of duplication eliminated

---

## Phase 5: Extract Handler Modules (High Risk)

**Goal:** Split 26 callbacks into focused handler modules

**Why Fifth:** Requires Phase 1-4 infrastructure, affects main.rs structure

### 5.1 Create `src/handlers/mod.rs`
```rust
pub mod file_handlers;
pub mod item_handlers;
pub mod filter_handlers;
pub mod dialog_handlers;

pub use file_handlers::register_file_handlers;
pub use item_handlers::register_item_handlers;
pub use filter_handlers::register_filter_handlers;
pub use dialog_handlers::register_dialog_handlers;
```

### 5.2 Create `src/handlers/file_handlers.rs`

**Purpose:** All file operation callbacks

```rust
use crate::operations::FileOperations;
use crate::state::{AppState, UiState, PendingAction};
use crate::ui::{dialogs, updates};

pub fn register_file_handlers(
    window: &MainWindow,
    app_state: &Rc<RefCell<AppState>>,
    ui_state: &Rc<RefCell<UiState>>,
) {
    register_file_open(window, app_state, ui_state);
    register_file_save(window, app_state);
    register_file_save_as(window, app_state);
    register_file_new(window, app_state, ui_state);
    register_file_revert(window, app_state, ui_state);
}

fn register_file_open(
    window: &MainWindow,
    app_state: &Rc<RefCell<AppState>>,
    ui_state: &Rc<RefCell<UiState>>,
) {
    let main_window_weak = window.as_weak();
    let app_state = app_state.clone();
    let ui_state = ui_state.clone();

    window.on_file_open(move || {
        let window = main_window_weak.unwrap();

        // Check for unsaved changes
        if app_state.borrow().dirty {
            ui_state.borrow_mut().set_pending(PendingAction::Open);
            dialogs::show_confirmation(
                &window,
                "You have unsaved changes. Do you want to save before opening another file?"
            );
        } else {
            let app_state = app_state.clone();
            slint::spawn_local(async move {
                let ops = FileOperations::new(&app_state, &window);
                ops.open_file_dialog_and_load().await;
            }).unwrap();
        }
    });
}

// Similar for register_file_save, register_file_save_as, etc.
```

**Move from main.rs:**
- `on_file_open` (lines 300-329)
- `on_file_save` (lines 364-398)
- `on_file_save_as` (lines 399-441)
- `on_file_new` (lines 444-469)
- `on_file_revert` (lines 476-504)

**Expected Reduction:** main.rs loses ~200 lines, gains focused module

### 5.3 Create `src/handlers/item_handlers.rs`

**Purpose:** All item CRUD callbacks

**Move from main.rs:**
- `on_item_selected` (lines 271-297)
- `on_start_edit` (lines 641-675)
- `on_save_edit` (lines 678-764)
- `on_cancel_edit` (lines 771-783)
- `on_start_create_item` (lines 786-811)
- `on_save_new_item` (lines 814-884)
- `on_cancel_create_item` (lines 891-902)
- `on_delete_item` (lines 970-1020)

**Use:**
- `operations::validation` for input validation
- `operations::item_ops` for state mutations

**Expected Reduction:** main.rs loses ~350 lines

### 5.4 Create `src/handlers/filter_handlers.rs`

**Purpose:** Filtering and sorting callbacks

**Move from main.rs:**
- `on_sort_by_name` (lines 511-538)
- `on_apply_filters` (lines 541-607)
- `on_clear_filters` (lines 615-638)

**Use `taxstud-core` functions instead of re-implementing:**
```rust
// Current code re-implements filtering locally
let filtered_items: Vec<_> = items.iter()
    .filter(|item| matches_filters(item, &filters))
    .cloned()
    .collect();

// Use taxstud-core instead
use taxstud_core::apply_filters;
let filtered_items = apply_filters(items, &filters);
```

**Expected Reduction:** main.rs loses ~100 lines + uses existing core functions

### 5.5 Create `src/handlers/dialog_handlers.rs`

**Purpose:** Confirmation dialog response callbacks

**Move from main.rs:**
- `on_confirmation_save` (lines 1028-1116)
- `on_confirmation_dont_save` (lines 1119-1180)
- `on_confirmation_cancel` (lines 1183-1197)
- `on_simple_confirmation_ok` (lines 1213-1264)
- `on_simple_confirmation_cancel` (lines 1267-1279)
- `on_error_dialog_close` (lines 1200-1206)

**Consolidate duplicated logic:**
```rust
fn execute_pending_action(
    action: PendingAction,
    app_state: &Rc<RefCell<AppState>>,
    window: &MainWindow,
) {
    match action {
        PendingAction::Open => {
            let app_state = app_state.clone();
            slint::spawn_local(async move {
                let ops = FileOperations::new(&app_state, &window);
                ops.open_file_dialog_and_load().await;
            }).unwrap();
        }
        PendingAction::New => {
            app_state.borrow_mut().create_new();
            updates::refresh_after_file_load(
                window,
                &app_state.borrow(),
                "New taxonomy created"
            );
        }
    }
}
```

**Expected Reduction:** main.rs loses ~150 lines + eliminates 70 lines of duplication

---

## Phase 6: Update main.rs (Final Integration)

**Goal:** Clean entry point that orchestrates modules

### 6.1 New main.rs structure (~150 lines)

```rust
mod state;
mod handlers;
mod operations;
mod ui;
mod errors;

use state::{AppState, UiState};
use handlers::*;

pub fn main() {
    let args = Args::parse();
    let main_window = MainWindow::new().unwrap();

    // Initialize state
    let app_state = Rc::new(RefCell::new(AppState::new()));
    let ui_state = Rc::new(RefCell::new(UiState::new()));

    // Set initial window title
    main_window.set_window_title(SharedString::from("Taxonomy Studio - No file loaded"));

    // Load file from command line if provided
    if let Some(file_path) = args.file {
        load_initial_file(&main_window, &app_state, file_path);
    }

    // Register all handlers
    register_file_handlers(&main_window, &app_state, &ui_state);
    register_item_handlers(&main_window, &app_state);
    register_filter_handlers(&main_window, &app_state);
    register_dialog_handlers(&main_window, &app_state, &ui_state);
    register_ui_handlers(&main_window); // theme, about, facet updates

    main_window.run().unwrap();
}

fn load_initial_file(
    window: &MainWindow,
    state: &Rc<RefCell<AppState>>,
    path: PathBuf,
) {
    slint::spawn_local({
        let ops = FileOperations::new(state, window);
        async move {
            ops.load_file(&path).await;
        }
    }).unwrap();
}
```

**Expected Result:** main.rs reduced from 1,466 lines to ~150 lines (10x reduction)

---

## Implementation Order and Risk Assessment

### Low Risk (Do First)
1. **Phase 1: Extract Error Handling** - No dependencies, immediate value
2. **Phase 2: Extract UI Helpers** - Creates foundation, minimal changes

### Medium Risk (Do Second)
3. **Phase 3: Split AppState** - Affects all callbacks but clear migration path
4. **Phase 4: Extract File Operations** - Uses Phase 1-3 infrastructure

### High Risk (Do Last)
5. **Phase 5: Extract Handlers** - Requires Phases 1-4, touches all callbacks
6. **Phase 6: Update main.rs** - Final integration

---

## Testing Strategy

### After Each Phase

**Regression Testing:**
1. Build succeeds: `cargo build`
2. No clippy warnings: `cargo clippy`
3. App launches: `cargo run -- assets/beverages.json`
4. Manual testing:
   - Open file
   - Edit item
   - Create item
   - Delete item
   - Save file
   - Revert to saved
   - All dialogs work

### Critical Test Cases

**File Operations:**
- Open with unsaved changes → Confirm dialog works
- Save with no path → Error dialog works
- Revert with no changes → Proper message

**Item Operations:**
- Edit validation → Errors show correctly
- Create validation → Errors show correctly
- Delete → Confirmation works

**Filters:**
- Apply filters → Results correct
- Clear filters → All items show

---

## Success Metrics

**Code Quality:**
- ✅ main.rs reduced from 1,466 lines to ~150 lines
- ✅ Error handling code reduced from 72 lines to 18 lines
- ✅ File loading duplication eliminated (3x → 1x)
- ✅ Validation duplication eliminated (2x → 1x)
- ✅ UI refresh patterns standardized (8 variants → 2 functions)

**Maintainability:**
- ✅ Clear module boundaries
- ✅ Single Responsibility Principle applied
- ✅ Easy to add new features (new handler = new file)
- ✅ Easy to test (modules can be tested independently)

**Architecture:**
- ✅ Domain state separated from UI state
- ✅ Business logic separated from UI callbacks
- ✅ File I/O orchestration centralized
- ✅ Error handling standardized
- ✅ Validation logic reusable

---

## Future Enhancements Enabled

After refactoring, these become easier:

1. **Unit Testing:** Operations and handlers can be tested independently
2. **Feature Flags:** Easy to disable handlers or operations
3. **Undo/Redo:** State operations are centralized
4. **Multiple Views:** State management is independent of UI
5. **Plugin System:** Handler registration is explicit
6. **Async Operations:** File operations already use async patterns

---

## Critical Files to Modify

**Phase 1:**
- Create: `src/errors/mod.rs`, `src/errors/error_mapper.rs`
- Modify: `src/main.rs` (6 locations)

**Phase 2:**
- Create: `src/ui/mod.rs`, `src/ui/updates.rs`, `src/ui/formatting.rs`, `src/ui/dialogs.rs`
- Modify: `src/main.rs` (move functions)

**Phase 3:**
- Create: `src/state/mod.rs`, `src/state/app_state.rs`, `src/state/ui_state.rs`
- Modify: `src/main.rs` (all 26 callbacks)

**Phase 4:**
- Create: `src/operations/mod.rs`, `src/operations/file_ops.rs`, `src/operations/validation.rs`
- Modify: `src/main.rs` (4 file operation callbacks)

**Phase 5:**
- Create: `src/handlers/mod.rs`, `src/handlers/file_handlers.rs`, `src/handlers/item_handlers.rs`, `src/handlers/filter_handlers.rs`, `src/handlers/dialog_handlers.rs`
- Modify: `src/main.rs` (all 26 callbacks extracted)

**Phase 6:**
- Restructure: `src/main.rs` (final cleanup)

---

## Rollback Plan

Each phase can be rolled back independently:

- **Phase 1:** Remove error module, revert error handling in main.rs
- **Phase 2:** Remove ui module, restore functions to main.rs
- **Phase 3:** Merge states back to single AppState
- **Phase 4:** Move operations back to main.rs
- **Phase 5:** Move handlers back to main.rs
- **Phase 6:** Restore old main.rs structure

Git commits should be made after each successful phase.
