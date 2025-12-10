# Code Quality Improvement Plan

**Project:** TaxStud
**Date:** 2025-12-10
**Status:** Ready for Review

## Executive Summary

This document identifies DRY (Don't Repeat Yourself) violations and dead code in the TaxStud codebase, providing a prioritized remedy plan. The analysis covers both the Rust backend and Slint UI components.

**Key Findings:**
- 7 DRY violation categories identified
- 3 instances of dead code detected
- Estimated effort: 4-6 hours for all improvements
- No breaking changes required

---

## 1. DRY Violations

### 1.1 Facet Value Extraction Pattern [HIGH PRIORITY]

**Severity:** High
**Effort:** 1-2 hours
**Files Affected:** 4

**Description:**
The pattern for extracting string values from facets (which can be either `String` or `Array<String>`) is duplicated in 4 different locations:

1. `taxstud-core/src/sorting.rs:70-85` - `get_facet_string()`
2. `taxstud-core/src/filtering.rs:28-47` - inline in `matches_filters()`
3. `taxstud-core/src/grouping.rs:11-28` - inline in `group_items_by_facet()`
4. `examples/taxstud_cli.rs:260-268` - inline in `print_example_item()`

**Current Code Pattern:**
```rust
// Pattern repeated 4 times with slight variations
match facet_value {
    serde_json::Value::String(s) => { /* use s */ }
    serde_json::Value::Array(arr) => {
        // Extract strings from array
        arr.iter().filter_map(|v| v.as_str()).collect()
    }
    _ => { /* handle other cases */ }
}
```

**Recommended Solution:**
Extract into a shared utility function in `taxstud-core/src/models.rs`:

```rust
impl Item {
    /// Get a facet value as a string (handles both single values and arrays)
    pub fn get_facet_as_string(&self, facet_name: &str) -> Option<String> {
        self.facets.get(facet_name).and_then(|v| match v {
            serde_json::Value::String(s) => Some(s.clone()),
            serde_json::Value::Array(arr) => {
                let values: Vec<String> = arr
                    .iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect();
                if values.is_empty() {
                    None
                } else {
                    Some(values.join(", "))
                }
            }
            _ => None,
        })
    }

    /// Get facet values as a vector (always returns Vec, empty if not found)
    pub fn get_facet_as_vec(&self, facet_name: &str) -> Vec<String> {
        self.facets.get(facet_name).map(|v| match v {
            serde_json::Value::String(s) => vec![s.clone()],
            serde_json::Value::Array(arr) => arr
                .iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect(),
            _ => vec![],
        }).unwrap_or_default()
    }
}
```

**Migration Steps:**
1. Add methods to `Item` struct in `models.rs`
2. Update `sorting.rs` to use `item.get_facet_as_string()`
3. Update `filtering.rs` to use `item.get_facet_as_vec()`
4. Update `grouping.rs` to use `item.get_facet_as_vec()`
5. Update `taxstud_cli.rs` to use `item.get_facet_as_string()`
6. Remove old `get_facet_string()` function from `sorting.rs`

---

### 1.2 Facet Filter Parsing [MEDIUM PRIORITY]

**Severity:** Medium
**Effort:** 30 minutes
**Files Affected:** 2

**Description:**
The logic for parsing comma-separated "key=value" strings into `HashMap<String, Vec<String>>` is duplicated:

1. `src/main.rs:341-349` - in `on_apply_filters` callback
2. `examples/taxstud_cli.rs:96-105` - in `parse_filters()` function

**Recommended Solution:**
Move to `taxstud-core/src/filtering.rs`:

```rust
/// Parse facet filter strings in the format "key=value" into a filter map
/// Multiple values for the same key are collected into a vector
pub fn parse_facet_filters(facet_strings: &[String]) -> HashMap<String, Vec<String>> {
    let mut facet_map = HashMap::new();

    for facet_str in facet_strings {
        if let Some((key, value)) = facet_str.split_once('=') {
            facet_map
                .entry(key.trim().to_string())
                .or_insert_with(Vec::new)
                .push(value.trim().to_string());
        }
    }

    facet_map
}
```

**Migration Steps:**
1. Add function to `filtering.rs`
2. Export from `lib.rs`
3. Update `main.rs` to use the function
4. Update `taxstud_cli.rs` to use the function

---

### 1.3 String Empty Check Pattern [LOW PRIORITY]

**Severity:** Low
**Effort:** 20 minutes
**Files Affected:** 1

**Description:**
The pattern `.trim().is_empty()` is repeated 7 times in `validation.rs`.

**Recommended Solution:**
Add a helper trait or function:

```rust
trait StringValidation {
    fn is_empty_or_whitespace(&self) -> bool;
}

impl StringValidation for str {
    fn is_empty_or_whitespace(&self) -> bool {
        self.trim().is_empty()
    }
}
```

Or simpler function:
```rust
fn is_empty_or_whitespace(s: &str) -> bool {
    s.trim().is_empty()
}
```

**Note:** This is low priority as the current code is already clear. Only refactor if you prefer the abstraction.

---

### 1.4 UI State Update Pattern [MEDIUM PRIORITY]

**Severity:** Medium
**Effort:** 1 hour
**Files Affected:** 1

**Description:**
After edit/create/delete operations in `main.rs`, the same 4-step pattern is repeated:

1. Drop mutable state borrow
2. Update window title from state
3. Refresh UI from state
4. Set status message

**Locations:**
- Lines 520-549 (after save edit)
- Lines 662-674 (after create item)
- Lines 787-799 (after delete item)

**Recommended Solution:**
Extract into helper function:

```rust
fn refresh_ui_after_state_change(
    main_window: &MainWindow,
    state: &Rc<RefCell<AppState>>,
    status_message: &str
) {
    // Update window title
    let title = state.borrow().get_window_title();
    main_window.set_window_title(SharedString::from(title));

    // Refresh the UI
    update_ui_from_state(main_window, &state.borrow());

    // Set status
    main_window.set_status_message(SharedString::from(status_message));
}
```

**Migration Steps:**
1. Add helper function to `main.rs`
2. Replace duplicated code in save_edit handler
3. Replace duplicated code in save_new_item handler
4. Replace duplicated code in delete_item handler

---

### 1.5 Facet Display Formatting [LOW PRIORITY]

**Severity:** Low
**Effort:** 30 minutes
**Files Affected:** 2

**Description:**
Similar formatting logic for displaying facets appears in:
- `main.rs:870-890` - `format_facets()` for detail view
- `main.rs:893-903` - `format_facet_dimensions()` for dimension list
- `taxstud_cli.rs:254-270` - inline in `print_example_item()`

**Recommended Solution:**
Consider consolidating if these serve similar purposes, but this may be acceptable as they have different output formats (UI vs CLI).

**Decision Point:** Review if unification makes sense or if the different contexts justify separate implementations.

---

### 1.6 Item Access from State Pattern [MEDIUM PRIORITY]

**Severity:** Medium
**Effort:** 45 minutes
**Files Affected:** 1

**Description:**
Multiple callbacks in `main.rs` follow this pattern:
1. Borrow state
2. Check if taxonomy exists
3. Check if items exist
4. Validate index
5. Access item

**Recommended Solution:**
Add helper methods to `AppState`:

```rust
impl AppState {
    /// Get a reference to an item by index
    fn get_item(&self, index: i32) -> Option<&Item> {
        if index < 0 {
            return None;
        }

        self.taxonomy.as_ref()
            .and_then(|t| t.example_items.as_ref())
            .and_then(|items| items.get(index as usize))
    }

    /// Get a mutable reference to an item by index
    fn get_item_mut(&mut self, index: i32) -> Option<&mut Item> {
        if index < 0 {
            return None;
        }

        self.taxonomy.as_mut()
            .and_then(|t| t.example_items.as_mut())
            .and_then(|items| items.get_mut(index as usize))
    }
}
```

---

### 1.7 ScrollView Pattern in UI [LOW PRIORITY]

**Severity:** Low
**Effort:** 30 minutes
**Files Affected:** 1

**Description:**
The `ScrollView { VerticalBox { ... } }` pattern appears 3 times in `ui/app-window.slint` for different panels.

**Recommended Solution:**
This is acceptable duplication as each panel has different content. Slint doesn't have great component reuse for layout patterns. Consider this an acceptable trade-off.

**Decision Point:** Only refactor if you add more panels with the same pattern (4+ instances).

---

## 2. Dead Code

### 2.1 Unused State Field: `selected_hierarchy_node` [HIGH PRIORITY]

**Severity:** High
**Effort:** 10 minutes
**Files Affected:** 1

**Description:**
The `selected_hierarchy_node` field in `AppState` (line 19 of `main.rs`) is never read, only set to `None`.

**Locations:**
- Defined: `main.rs:19`
- Set to None: `main.rs:32, 53, 104`
- Never read anywhere

**Recommended Solution:**
Remove the field entirely:

```rust
// DELETE these lines
selected_hierarchy_node: Option<String>, // line 19
selected_hierarchy_node: None,            // lines 32, 53, 104
```

**Rationale:**
This appears to be leftover from earlier design where hierarchy selection was planned but never implemented. The UI currently doesn't allow selecting hierarchy nodes, only items from the list.

**Breaking Changes:** None (internal struct only)

---

### 2.2 Unused Function: `apply_filters` [MEDIUM PRIORITY]

**Severity:** Medium
**Effort:** 5 minutes
**Files Affected:** 2

**Description:**
The `apply_filters()` function in `taxstud-core/src/filtering.rs:4-10` is exported but never called anywhere in the codebase.

**Current Usage:**
- Defined in: `filtering.rs:4`
- Exported from: `lib.rs:12`
- **Never called** - all code uses `matches_filters()` directly with iterators

**Recommended Solution:**
**Option A (Conservative):** Keep it as it's part of the public API and may be useful for external users of the library.

**Option B (Aggressive):** Remove it if you're confident this is an internal-only library:

```rust
// DELETE from filtering.rs
pub fn apply_filters(items: &[Item], filters: &Filters) -> Vec<Item> { ... }

// DELETE from lib.rs
pub use filtering::{apply_filters, matches_filters, has_filters};
// CHANGE TO:
pub use filtering::{matches_filters, has_filters};
```

**Decision Point:**
- If `taxstud-core` is meant to be a reusable library → **Keep it** (convenience function)
- If it's internal-only → **Remove it** (unused code)

**Recommendation:** Keep it. It's a useful convenience wrapper even if not currently used internally.

---

### 2.3 Potentially Unused Imports [LOW PRIORITY]

**Severity:** Low
**Effort:** 5 minutes
**Files Affected:** Multiple

**Description:**
Some imports might be unused. Rust compiler will warn about these.

**Recommended Solution:**
Run `cargo clippy` to identify unused imports and remove them:

```bash
cargo clippy -- -W unused-imports
```

Then remove any flagged imports.

---

## 3. Implementation Priority

### Phase 1: High Priority (Do First)
1. **Remove dead state field** (2.1) - 10 minutes
2. **Extract facet value methods** (1.1) - 1-2 hours

**Total Phase 1:** ~2 hours

### Phase 2: Medium Priority (Do Second)
1. **Extract facet filter parsing** (1.2) - 30 minutes
2. **Extract UI refresh helper** (1.4) - 1 hour
3. **Add item access helpers** (1.6) - 45 minutes
4. **Review apply_filters usage** (2.2) - 5 minutes

**Total Phase 2:** ~2.5 hours

### Phase 3: Low Priority (Optional)
1. **String validation helper** (1.3) - 20 minutes
2. **Review facet formatting** (1.5) - 30 minutes
3. **Remove unused imports** (2.3) - 5 minutes
4. **Review ScrollView pattern** (1.7) - Decision only

**Total Phase 3:** ~1 hour

---

## 4. Testing Strategy

After each refactoring step:

1. **Unit Tests:** If tests exist, run `cargo test`
2. **Build Check:** Run `cargo build` to ensure compilation
3. **Clippy:** Run `cargo clippy` to catch issues
4. **Manual Testing:** Load the movies.json file and verify:
   - Items display correctly
   - Filtering works
   - Sorting works
   - Edit/Create/Delete operations work
   - Hierarchy tree displays

---

## 5. Risk Assessment

### Low Risk Refactorings
- 1.1 (Facet extraction) - Pure refactoring, no logic change
- 1.2 (Filter parsing) - Pure refactoring, no logic change
- 2.1 (Remove dead field) - No functional impact

### Medium Risk Refactorings
- 1.4 (UI refresh) - Timing of borrows matters, test carefully
- 1.6 (Item access) - Borrow checker sensitive, test thoroughly

### No Risk
- All Low Priority items are optional improvements

---

## 6. Estimated Total Effort

- **High Priority:** 2 hours
- **Medium Priority:** 2.5 hours
- **Low Priority:** 1 hour
- **Testing per phase:** 30 minutes × 3 = 1.5 hours

**Total:** 6-7 hours for complete refactoring

**Recommended Minimum:** Phase 1 only = 2 hours

---

## 7. Benefits

### Code Quality
- Reduced duplication (DRY principle)
- Cleaner, more maintainable code
- Easier to modify facet handling in future

### Performance
- No significant performance changes (refactoring only)

### Maintainability
- Single source of truth for common operations
- Easier to fix bugs (change in one place)
- Better for new contributors

---

## 8. Review Checklist

Before implementing, decide on:

- [ ] Prioritization: Which phases to implement?
- [ ] Timeline: When to start refactoring?
- [ ] API stability: Is `apply_filters` part of public API?
- [ ] Testing: Are there existing tests to update?
- [ ] Breaking changes: Any external dependents?

---

## 9. Notes

### Why These Were Flagged

**DRY Violations:** Code duplication makes maintenance harder and increases bug risk.

**Dead Code:** Unused code creates confusion and maintenance burden.

### Why Some Aren't Flagged

- Small helper functions that are only used once (acceptable)
- UI component repetition in Slint (framework limitation)
- Different output formats for different contexts (intentional)

---

## Appendix: Quick Reference

### Files to Modify
- `src/main.rs` - Most changes
- `taxstud-core/src/models.rs` - Add facet methods
- `taxstud-core/src/filtering.rs` - Add parsing function, remove dead code
- `taxstud-core/src/sorting.rs` - Use new facet methods
- `taxstud-core/src/grouping.rs` - Use new facet methods
- `taxstud-core/src/lib.rs` - Update exports
- `examples/taxstud_cli.rs` - Use new utilities

### Key Decisions Needed
1. Keep or remove `apply_filters()` function?
2. Implement all phases or just high priority?
3. When to schedule this work?
