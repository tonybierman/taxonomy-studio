# Taxman - Next Steps and Future Enhancements

## Project Status

**Completed Phases:**
- ✅ Phase 1: Shared Library Creation (taxman-core)
- ✅ Phase 2: GUI Foundation (file operations, state management)
- ✅ Phase 3: UI Layout Redesign (3-panel layout)
- ✅ Phase 4: Read-Only Display (hierarchy, items, facets)
- ✅ Phase 5: Filtering & Sorting (genus filters, facet filters, sort by name)
- ✅ Phase 6: Edit Operations (edit items with validation)
- ✅ Phase 7: Create & Delete Operations (create/delete items)

**Current State:**
- Fully functional CRUD GUI for taxonomy items
- All CLI features available in GUI (filtering, sorting, grouping)
- Shared library eliminates code duplication
- Clean architecture with proper separation of concerns

---

## Phase 8: File Management Enhancements

**Goal:** Improve file operations with better error handling and user protection.

### 8.1 Unsaved Changes Warnings

**Description:** Warn users before losing unsaved work.

**Implementation:**
1. Check `dirty` flag before:
   - Opening a new file
   - Creating a new taxonomy
   - Closing the application (if possible with Slint)
2. Show confirmation dialog: "You have unsaved changes. Do you want to save before continuing?"
3. Options: "Save", "Don't Save", "Cancel"

**Technical Notes:**
- Slint doesn't have built-in dialog support
- Options:
  - Use native dialogs via `rfd` crate (async dialogs)
  - Create custom dialog overlay in Slint UI
  - For now, simple approach: add a modal overlay in app-window.slint

**Files to Modify:**
- `ui/app-window.slint` - Add confirmation dialog overlay
- `src/main.rs` - Add dirty check logic to file operation handlers

**Estimated Effort:** Medium (2-3 hours)

---

### 8.2 Recent Files List

**Description:** Quick access to recently opened taxonomy files.

**Implementation:**
1. Store recent files list in application state or config file
2. Add "Recent Files" submenu or section (if implementing menu bar)
3. Limit to 5-10 most recent files
4. Clear invalid/missing files automatically

**Technical Notes:**
- Store in `~/.config/taxman/recent.json` or similar
- Use `directories` crate for cross-platform config paths
- Consider using `serde` for serialization

**Files to Modify:**
- `src/main.rs` - Add recent files tracking
- `ui/app-window.slint` - Add UI for recent files (if desired)

**Estimated Effort:** Medium (2-4 hours)

---

### 8.3 Better Error Handling

**Description:** Improve error messages and recovery from failures.

**Current Issues:**
- Validation errors shown inline but could be more detailed
- File I/O errors only show in status bar
- No recovery suggestions

**Implementation:**
1. Create error dialog component in Slint
2. Show detailed error messages for:
   - File load failures (file not found, parse errors, validation errors)
   - File save failures (permissions, disk full)
   - Invalid user input
3. Provide actionable suggestions:
   - "File not found. Would you like to browse for it?"
   - "Validation failed: [specific errors]. Fix these issues before saving."

**Files to Modify:**
- `ui/app-window.slint` - Add error dialog overlay
- `src/main.rs` - Enhanced error handling in all operations

**Estimated Effort:** Medium (3-4 hours)

---

### 8.4 Auto-Save / Backup

**Description:** Prevent data loss from crashes or mistakes.

**Implementation:**
1. Auto-save to temporary file every N minutes (if dirty)
2. Store in `~/.cache/taxman/autosave/`
3. On startup, check for autosave files
4. Prompt: "Found unsaved changes from [timestamp]. Restore?"

**Technical Notes:**
- Use `std::time::Instant` for timing
- Timer can be tricky in Slint - may need async background task
- Consider using `slint::Timer` for periodic tasks

**Files to Modify:**
- `src/main.rs` - Add auto-save logic with timer

**Estimated Effort:** Medium-High (4-6 hours)

---

### 8.5 File Format Versioning

**Description:** Handle schema evolution gracefully.

**Implementation:**
1. Add `schema_version` field to taxonomy JSON
2. Implement migration functions for old versions
3. Warn users if opening a file from newer version
4. Offer to upgrade old files to current schema

**Files to Modify:**
- `taxman-core/src/models.rs` - Add schema_version field
- `taxman-core/src/io.rs` - Add migration logic
- `taxman-core/src/validation.rs` - Validate schema version

**Estimated Effort:** High (6-8 hours)

---

## Phase 9: Polish & User Experience

**Goal:** Improve overall usability and professional feel.

### 9.1 Keyboard Shortcuts

**Description:** Add standard keyboard shortcuts for common operations.

**Shortcuts to Implement:**
- `Ctrl+N` - New taxonomy
- `Ctrl+O` - Open file
- `Ctrl+S` - Save
- `Ctrl+Shift+S` - Save As
- `Ctrl+E` - Edit selected item
- `Ctrl+D` or `Delete` - Delete selected item
- `Ctrl+F` - Focus filter input
- `Escape` - Cancel edit/create mode
- `Ctrl+Z` - Undo (if implemented)
- `Ctrl+Y` - Redo (if implemented)

**Technical Notes:**
- Slint supports keyboard shortcuts via `FocusScope` and key handlers
- Use `key-pressed` callback on Window or focused elements
- May need to implement key event routing

**Files to Modify:**
- `ui/app-window.slint` - Add key handlers
- `src/main.rs` - Wire up keyboard shortcut callbacks

**Estimated Effort:** Medium (3-5 hours)

---

### 9.2 Improved Visual Feedback

**Description:** Better loading states, transitions, and visual polish.

**Enhancements:**
1. **Loading Indicators:**
   - Show spinner/progress when loading large files
   - Disable UI during async operations

2. **Smooth Transitions:**
   - Fade in/out for mode changes (edit/create)
   - Highlight newly created/edited items

3. **Visual Refinements:**
   - Better color scheme consistency
   - Icons for buttons (if adding icon support)
   - Hover states for interactive elements
   - Focus indicators for accessibility

**Files to Modify:**
- `ui/app-window.slint` - Visual improvements

**Estimated Effort:** Medium (4-6 hours)

---

### 9.3 Tooltips and Help Text

**Description:** Contextual help for users learning the system.

**Implementation:**
1. Add tooltips to all buttons explaining their function
2. Add help text for complex fields (facet format, path format)
3. Consider adding a "Help" panel or overlay with:
   - Quick start guide
   - Keyboard shortcuts reference
   - Facet syntax examples

**Technical Notes:**
- Slint may not have built-in tooltip support
- Can implement custom tooltip component with hover states

**Files to Modify:**
- `ui/app-window.slint` - Add tooltip component and help text

**Estimated Effort:** Medium (3-4 hours)

---

### 9.4 Accessibility Improvements

**Description:** Ensure the application is usable by all users.

**Enhancements:**
1. **Keyboard Navigation:**
   - Full keyboard navigation without mouse
   - Visible focus indicators
   - Logical tab order

2. **Screen Reader Support:**
   - Proper ARIA labels (if Slint supports)
   - Descriptive button labels
   - Status announcements for operations

3. **Visual Accessibility:**
   - Sufficient color contrast (WCAG AA minimum)
   - Resizable text
   - High contrast mode support

**Files to Modify:**
- `ui/app-window.slint` - Accessibility attributes
- Test with accessibility tools

**Estimated Effort:** High (6-10 hours, requires testing)

---

### 9.5 Performance Optimizations

**Description:** Ensure smooth performance with large taxonomies.

**Optimizations:**
1. **Virtual Scrolling:**
   - For large item lists (1000+ items)
   - Only render visible items

2. **Lazy Loading:**
   - Load large taxonomies incrementally
   - Show progress during load

3. **Efficient Filtering:**
   - Debounce filter updates
   - Cache filter results
   - Use indices for fast lookup

**Technical Notes:**
- May need to implement custom list component
- Profile performance with large datasets first

**Files to Modify:**
- `ui/app-window.slint` - Virtual scrolling implementation
- `src/main.rs` - Optimize filter/sort logic

**Estimated Effort:** High (8-12 hours)

---

## Additional Features

### Feature 1: Confirmation Dialogs

**Description:** Add confirmation before destructive operations.

**Priority:** High
**Difficulty:** Low
**Estimated Effort:** 2-3 hours

**Implementation:**
1. Create confirmation dialog component in Slint
2. Show before deleting items: "Delete '[item name]'? This cannot be undone."
3. Options: "Delete", "Cancel"
4. Consider adding "Don't ask again" checkbox

**Files to Modify:**
- `ui/app-window.slint` - Add confirmation dialog overlay
- `src/main.rs` - Show dialog before delete operations

---

### Feature 2: Undo/Redo Functionality

**Description:** Allow users to undo/redo changes to the taxonomy.

**Priority:** High
**Difficulty:** High
**Estimated Effort:** 8-12 hours

**Implementation:**
1. Implement command pattern for all mutations
2. Maintain undo/redo stacks in AppState
3. Commands to implement:
   - CreateItem, EditItem, DeleteItem
   - (Later) CreateNode, EditNode, DeleteNode
4. Limit stack size (e.g., 50 operations)
5. Add UI indicators (Undo/Redo buttons, keyboard shortcuts)

**Technical Notes:**
- Need to clone taxonomy state efficiently
- Consider using `im` crate for persistent data structures
- Alternative: Store deltas instead of full states

**Files to Modify:**
- `src/main.rs` - Add command pattern and undo/redo logic
- `ui/app-window.slint` - Add Undo/Redo buttons

**Data Structure Example:**
```rust
enum TaxonomyCommand {
    CreateItem { item: Item },
    EditItem { index: usize, old_item: Item, new_item: Item },
    DeleteItem { index: usize, item: Item },
}

struct UndoRedoManager {
    undo_stack: Vec<TaxonomyCommand>,
    redo_stack: Vec<TaxonomyCommand>,
}
```

---

### Feature 3: Hierarchy Node & Facet Dimension Editing

**Description:** Complete CRUD for all taxonomy elements, not just items.

**Priority:** Medium
**Difficulty:** Medium-High
**Estimated Effort:** 10-15 hours

**Sub-features:**

#### 3.1 Edit Hierarchy Nodes
- Edit genus, species, differentia
- Add/remove child nodes
- Reorder nodes in hierarchy
- Validation: ensure no cycles

#### 3.2 Edit Facet Dimensions
- Add new facet dimensions (e.g., "color", "size")
- Add/remove allowed values for facets
- Rename facet dimensions
- Delete facet dimensions (with cascade warning)

#### 3.3 Edit Taxonomy Metadata
- Edit taxonomy description
- Edit root node name
- Add custom fields to taxonomy

**Files to Modify:**
- `ui/app-window.slint` - Add UI for hierarchy/facet editing
- `src/main.rs` - Add edit handlers for all elements
- `taxman-core/src/validation.rs` - Validate hierarchy changes

---

### Feature 4: Search/Find Functionality

**Description:** Quick search to find items by name or properties.

**Priority:** Medium
**Difficulty:** Medium
**Estimated Effort:** 4-6 hours

**Implementation:**
1. Add search input box (top toolbar or filter panel)
2. Search modes:
   - **Item Name:** Fuzzy search by item name
   - **Facet Values:** Search within facet values
   - **Classification Path:** Search within hierarchy paths
3. Highlight matching items in list
4. Jump to match with Enter key
5. Show match count: "3 of 15 items match"

**Technical Notes:**
- Use fuzzy matching library (e.g., `fuzzy-matcher`)
- Debounce search input for performance
- Case-insensitive search

**Files to Modify:**
- `ui/app-window.slint` - Add search UI
- `src/main.rs` - Implement search logic
- Consider adding to `taxman-core/src/filtering.rs`

---

### Feature 5: Export/Import Formats

**Description:** Support additional file formats beyond JSON.

**Priority:** Medium
**Difficulty:** Medium
**Estimated Effort:** 6-10 hours per format

**Formats to Support:**

#### 5.1 CSV Export
- Export items as flat CSV with columns:
  - Name, Classification Path, Facet1, Facet2, ...
- Useful for spreadsheet analysis

#### 5.2 Markdown Export
- Export taxonomy as formatted markdown (similar to CLI output)
- Useful for documentation

#### 5.3 YAML Support
- Alternative to JSON (more human-readable)
- Import/export in YAML format

#### 5.4 Excel/XLSX Export
- Full-featured export with multiple sheets:
  - Sheet 1: Items
  - Sheet 2: Hierarchy
  - Sheet 3: Facet Dimensions
- Use `rust_xlsxwriter` crate

**Files to Modify:**
- `taxman-core/src/io.rs` - Add format-specific functions
- `src/main.rs` - Add export menu/options
- `ui/app-window.slint` - Add format selection UI

---

### Feature 6: Batch Operations

**Description:** Perform operations on multiple items at once.

**Priority:** Low-Medium
**Difficulty:** Medium
**Estimated Effort:** 6-8 hours

**Operations:**
1. **Multi-select items:**
   - Ctrl+Click for individual selection
   - Shift+Click for range selection
   - Ctrl+A for select all

2. **Batch edit:**
   - Edit facets for all selected items
   - Change classification path for selected items

3. **Batch delete:**
   - Delete all selected items with confirmation

4. **Bulk import:**
   - Import multiple items from CSV

**Files to Modify:**
- `ui/app-window.slint` - Add multi-select UI
- `src/main.rs` - Track selected items, batch operations

---

### Feature 7: Taxonomy Validation Report

**Description:** Generate comprehensive validation report for the taxonomy.

**Priority:** Low
**Difficulty:** Low-Medium
**Estimated Effort:** 3-5 hours

**Implementation:**
1. Add "Validate" button/menu item
2. Run full validation check (already exists in taxman-core)
3. Display results in a report view:
   - ✅ Passed checks
   - ⚠️ Warnings (non-critical issues)
   - ❌ Errors (must be fixed)
4. Click on error to navigate to problematic item

**Files to Modify:**
- `ui/app-window.slint` - Add validation report overlay
- `src/main.rs` - Generate and display validation report
- `taxman-core/src/validation.rs` - Enhanced validation with warnings

---

### Feature 8: Taxonomy Diff/Compare

**Description:** Compare two taxonomy versions to see changes.

**Priority:** Low
**Difficulty:** Medium-High
**Estimated Effort:** 8-12 hours

**Implementation:**
1. Load two taxonomy files for comparison
2. Show diff report:
   - Items added/removed/modified
   - Hierarchy changes
   - Facet dimension changes
3. Visual diff view (side-by-side or inline)
4. Export diff report

**Files to Modify:**
- `taxman-core/src/` - Add new `diff.rs` module
- `ui/app-window.slint` - Add diff UI
- `src/main.rs` - Diff logic and display

---

### Feature 9: Hierarchical Tree View

**Description:** Interactive tree view for classical hierarchy navigation.

**Priority:** Medium
**Difficulty:** High
**Estimated Effort:** 10-15 hours

**Implementation:**
1. Build tree component in Slint (currently shows placeholder)
2. Expandable/collapsible nodes
3. Display: Genus > Species (differentia)
4. Click to filter items by hierarchy node
5. Drag-and-drop to reorganize hierarchy (advanced)

**Technical Notes:**
- Slint doesn't have built-in tree component
- Need to implement custom recursive component
- State management for expand/collapse

**Files to Modify:**
- `ui/components/hierarchy-tree.slint` - New custom tree component
- `ui/app-window.slint` - Integrate tree component
- `src/main.rs` - Tree data model and interaction

---

### Feature 10: Grouping View

**Description:** Display items grouped by facet values (like CLI --group-by).

**Priority:** Low-Medium
**Difficulty:** Medium
**Estimated Effort:** 4-6 hours

**Implementation:**
1. Add "Group By" dropdown in items panel
2. Options: None, or any facet dimension
3. Display items in expandable groups:
   ```
   ▼ Temperature: Hot (5 items)
     - Coffee
     - Tea
     - ...
   ▼ Temperature: Iced (3 items)
     - Iced Coffee
     - ...
   ```
4. Use existing `taxman-core::group_items_by_facet()` logic

**Files to Modify:**
- `ui/app-window.slint` - Add group view UI
- `src/main.rs` - Implement grouping display logic

---

## Priority Recommendations

### High Priority (Do First)
1. **Phase 8.1** - Unsaved Changes Warnings (essential for data protection)
2. **Phase 9.1** - Keyboard Shortcuts (huge UX improvement)
3. **Feature 1** - Confirmation Dialogs (prevent accidental deletions)
4. **Phase 8.3** - Better Error Handling (professional feel)

### Medium Priority (Do Second)
1. **Feature 2** - Undo/Redo (very valuable, high effort)
2. **Feature 3** - Hierarchy/Facet Editing (complete CRUD)
3. **Feature 4** - Search/Find (essential for large taxonomies)
4. **Phase 9.2** - Visual Feedback (polish)
5. **Feature 9** - Hierarchical Tree View (core feature)

### Low Priority (Nice to Have)
1. **Phase 8.2** - Recent Files List
2. **Feature 5** - Export/Import Formats
3. **Feature 6** - Batch Operations
4. **Feature 7** - Validation Report
5. **Feature 10** - Grouping View
6. **Phase 8.4** - Auto-Save (high effort, medium value)
7. **Phase 9.5** - Performance Optimizations (only if needed)

### Long-term (Future Considerations)
1. **Phase 8.5** - File Format Versioning (important for production)
2. **Phase 9.4** - Accessibility (important but time-consuming)
3. **Feature 8** - Taxonomy Diff/Compare (advanced feature)

---

## Technical Debt & Known Issues

### Current Limitations
1. **No confirmation dialogs** - Deletes are immediate
2. **No tree view** - Hierarchy panel shows placeholder
3. **Simple error messages** - Only shown in status bar
4. **No keyboard shortcuts** - Mouse required for all operations
5. **No multi-select** - Can only operate on one item at a time

### Code Quality
- Well-structured, clean separation between core logic and UI
- Good use of Rust best practices
- Comprehensive error handling in core library
- Could benefit from more inline documentation

### Performance
- Current implementation is fine for small-medium taxonomies (< 1000 items)
- May need optimization for very large datasets

---

## Development Guidelines

### Before Starting a Feature
1. Review existing code and understand architecture
2. Check if taxman-core needs changes or just GUI
3. Plan UI changes first (sketch or mockup)
4. Consider backward compatibility

### Testing Strategy
1. Manual testing with all sample files:
   - `assets/beverages.json`
   - `assets/books.json`
   - `assets/stocks.json`
2. Test with empty taxonomies
3. Test with very large taxonomies (create test data)
4. Test error cases (invalid files, permissions, etc.)

### Code Style
- Follow existing patterns in codebase
- Use meaningful variable names
- Add comments for complex logic
- Keep functions focused and small

---

## Resources & References

### Documentation
- **Slint Documentation:** https://slint.dev/releases/1.14.1/docs/slint/
- **Slint Examples:** https://github.com/slint-ui/slint/tree/master/examples
- **Rust Book:** https://doc.rust-lang.org/book/

### Useful Crates
- `rfd` - File dialogs
- `directories` - Cross-platform paths
- `fuzzy-matcher` - Fuzzy search
- `im` - Persistent data structures (for undo/redo)
- `rust_xlsxwriter` - Excel export
- `serde_yaml` - YAML support

### Related Projects
- Review CLI implementation (`examples/taxman_cli.rs`) for feature inspiration
- Check taxman-core tests for validation examples

---

## Session Planning

### Suggested Session 1: Essential UX
- Phase 8.1: Unsaved Changes Warnings
- Feature 1: Confirmation Dialogs
- Phase 9.1: Keyboard Shortcuts (basic set)

**Estimated Time:** 4-6 hours

### Suggested Session 2: Advanced Editing
- Feature 3.1: Edit Hierarchy Nodes
- Feature 3.2: Edit Facet Dimensions
- Feature 3.3: Edit Taxonomy Metadata

**Estimated Time:** 6-8 hours

### Suggested Session 3: Search & Navigation
- Feature 4: Search/Find Functionality
- Feature 9: Hierarchical Tree View
- Feature 10: Grouping View

**Estimated Time:** 6-8 hours

### Suggested Session 4: Undo & Polish
- Feature 2: Undo/Redo Functionality
- Phase 9.2: Visual Feedback
- Phase 9.3: Tooltips and Help

**Estimated Time:** 6-8 hours

---

## Conclusion

The Taxman application has a solid foundation with complete CRUD functionality for items, filtering, sorting, and file management. The next phases will focus on:

1. **User Protection:** Preventing data loss through warnings and confirmations
2. **Usability:** Keyboard shortcuts, better feedback, help system
3. **Completeness:** Full hierarchy and facet editing
4. **Advanced Features:** Undo/redo, search, batch operations

Each suggested session builds on the previous one, gradually enhancing the application from a functional tool to a polished, professional taxonomy management system.

Good luck with future development sessions! 🚀
