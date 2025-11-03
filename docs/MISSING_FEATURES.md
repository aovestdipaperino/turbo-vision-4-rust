# Missing Features Inventory

*Generated from Borland Turbo Vision source analysis*
*Last updated: 2025-11-03 (post-List Components implementation)*

This document catalogs missing features compared to the original Borland Turbo Vision framework, providing a roadmap for future development.

## Summary Statistics

- **Total Missing Components**: 46 (was 85, implemented 9, skipped 30 obsolete pre-Rust features)
- **Estimated Total Effort**: 826 hours (~21 weeks at 40 hrs/week)
- **HIGH Priority**: 11 items (212 hours) - Core functionality
- **MEDIUM Priority**: 31 items (352 hours) - Extended features
- **LOW Priority**: 17 items (262 hours) - Nice to have

## Quick Reference by Category

| Category | Count | Priority | Effort |
|----------|-------|----------|--------|
| Core Views/Controls | 10 | HIGH-MEDIUM | 112h |
| Specialized Dialogs | 13 | LOW-MEDIUM | 126h |
| Editor Components | 3 | HIGH-MEDIUM | 52h |
| System Utilities | 10 | MEDIUM | 34h |
| Helper Classes | 0 | - | 0h |
| Advanced Features | 10 | HIGH-LOW | 162h |

## High Priority Components (Core Functionality)

### Collections & Data Structures (~0 hours - NOT NEEDED)
- ~~**TCollection**~~ - Use Rust `Vec<T>` instead (type-safe, generic)
- ~~**TSortedCollection**~~ - Use `Vec<T>` + sort/binary_search
- ~~**TNSCollection**~~ - Not needed in Rust
- ~~**TNSSortedCollection**~~ - Not needed in Rust

**Note:** Borland's collections were pre-generics workarounds. Rust's `Vec<T>`, `HashMap<K,V>`, and standard library provide superior type-safe alternatives. We use `Vec` throughout the codebase instead of recreating 1990s-era dynamic arrays.

### Menu & Status Infrastructure (~0 hours remaining)
- ✅ **MenuItem** - Menu item data structure (IMPLEMENTED in v0.2.2 - `src/core/menu_data.rs`)
- ✅ **Menu** - Menu data structure (IMPLEMENTED in v0.2.2 - `src/core/menu_data.rs`)
- ✅ **MenuBuilder** - Fluent builder for menus (IMPLEMENTED in v0.2.2 - `src/core/menu_data.rs`)
- ✅ **StatusItem** - Status line item (IMPLEMENTED in v0.2.2 - `src/core/status_data.rs`)
- ✅ **StatusDef** - Status line definition (IMPLEMENTED in v0.2.2 - `src/core/status_data.rs`)
- ✅ **StatusLine** - Status line configuration (IMPLEMENTED in v0.2.2 - `src/core/status_data.rs`)
- ✅ **StatusLineBuilder** - Fluent builder for status lines (IMPLEMENTED in v0.2.2 - `src/core/status_data.rs`)

**Note:** Rust implementation uses `Vec` instead of linked lists for type safety. Provides both Borland-compatible API and idiomatic Rust builders.

### List Components (~0 hours remaining)
- ✅ **TListViewer** - Base for list views (IMPLEMENTED - `src/views/list_viewer.rs`)
- ✅ **TMenuView** - Base for menu views (IMPLEMENTED - `src/views/menu_viewer.rs`)
- ✅ **TMenuBox** - Popup menu container (IMPLEMENTED - `src/views/menu_box.rs`)

**Implementation Notes:**
- Hybrid trait + helper struct pattern (ListViewer/MenuViewer traits + State structs)
- ListBox refactored to use ListViewer trait (eliminated 70+ lines of duplicate navigation)
- MenuBar refactored to use MenuViewer trait (eliminated 200+ lines of duplicate logic)
- All navigation behavior now shared through default trait implementations
- Borland-compatible while being idiomatic Rust

### Input Controls (0 hours remaining)
- ✅ **TCluster** - Base for radio/checkbox (IMPLEMENTED - `src/views/cluster.rs`)
- ✅ **THistory** - History dropdown (IMPLEMENTED - `src/views/history.rs`)
- ✅ **THistoryViewer** - History list viewer (IMPLEMENTED - `src/views/history_viewer.rs`)
- ✅ **THistoryWindow** - History popup (IMPLEMENTED - `src/views/history_window.rs`)

### File System (26 hours)
- **TFileList** - File browser list (12h)
- **TDirListBox** - Directory tree (14h)

### Editor (32 hours)
- **TFileEditor** - File editor with load/save (24h)
- **TEditWindow** - Editor window wrapper (8h)

### Application Framework (58 hours)
- **TProgram** - Base application (20h)
- **TApplication** - Extended application (16h)
- **TScreen** - Screen manager (20h)
- **TDisplay** - Display abstraction (16h)
- **TMouse** - Mouse system (12h)
- **TEventQueue** - Event queue (10h)

**Total HIGH Priority: 212 hours** (was 282 hours, removed 38 hours of obsolete collections, completed 32 hours: TCluster 8h + History System 24h)

## Medium Priority Components (Extended Features)

### File Dialog Components (28 hours)
- **TFileInputLine** - File path input (6h)
- **TFileInfoPane** - File info display (6h)
- **TChDirDialog** - Change directory dialog (10h)
- **TFileCollection** - File entry collection (8h)
- **TDirCollection** - Directory collection (8h)

### Resource System (~0 hours - NOT NEEDED)
- ~~**TResourceFile**~~ - Use JSON/TOML/RON with serde instead
- ~~**TResourceCollection**~~ - Use HashMap<String, Resource>
- ~~**TResourceItem**~~ - Use custom structs with derive macros

**Note:** Borland's binary resource files were a 1990s necessity. Modern Rust has excellent serialization libraries (serde) and standard formats (JSON, TOML, RON) that are more maintainable and debuggable.

### Help System (56 hours)
- **THelpFile** - Help file manager (20h)
- **THelpBase** - Help infrastructure (12h)
- **THelpWindow** - Help display window (12h)
- **THelpViewer** - Help content viewer (12h)

### Streaming System (~0 hours - NOT NEEDED)
- ~~**pstream, ipstream, opstream**~~ - Use serde for serialization
- ~~**fpstream, ifpstream, ofpstream, iopstream**~~ - Use std::fs + serde_json/bincode
- ~~**TWriteObjects, TReadObjects**~~ - Use serde Serialize/Deserialize traits
- ~~**TStreamable**~~ - Use #[derive(Serialize, Deserialize)] macros

**Note:** Borland's streaming system predated modern serialization libraries. Rust's serde ecosystem provides superior type-safe serialization to JSON, TOML, MessagePack, bincode, etc. with derive macros and zero-copy deserialization.

### String Utilities (~0 hours - NOT NEEDED)
- ~~**TStringCollection**~~ - Use Vec<String>
- ~~**TStringList**~~ - Use Vec<String> or HashMap<usize, String>
- ~~**TStrListMaker**~~ - Use Vec::push() or collect()
- ~~**TStrIndexRec**~~ - Not needed with Rust's type system

**Note:** String collections were pre-generic workarounds. Use Vec<String>, HashSet<String>, or HashMap for string management.

### List Enhancements (~0 hours remaining)
- ✅ **TSortedListBox** - Sorted list with binary search (IMPLEMENTED - `src/views/sorted_listbox.rs`)

### Application Enhancements (20 hours)
- **TDeskTop** - Enhanced desktop features (10h)
- **TEditorApp** - Editor application framework (20h)
- **TDrawBuffer** - Drawing utilities (8h)
- **CodePage** - Character encoding (12h)
- **OSClipboard** - System clipboard (10h)

**Total MEDIUM Priority: 352 hours** (was 486 hours, removed 126 hours of obsolete streaming/resources/strings, completed 8 hours of TSortedListBox)

## Low Priority Components (Nice to Have)

### Color Customization Suite (66 hours)
Complete color editor system:
- TColorDialog, TColorSelector, TMonoSelector (40h)
- TColorDisplay, TColorGroup, TColorItem (14h)
- TColorGroupList, TColorItemList (12h)

### Calculator (24 hours)
- TCalculator dialog (16h)
- TCalcDisplay component (8h)

### Advanced Validators (20 hours)
- **TPXPictureValidator** - Mask validation (12h)
- **TLookupValidator** - List validation (8h)

### Text Output (40 hours)
- **TTextDevice** - Text output base (12h)
- **TTerminal** - Terminal emulator (20h)
- **otstream** - Output text stream (8h)

### Configuration (10 hours)
- **ConfigFile** - Configuration manager (10h)

**Total LOW Priority: 262 hours**

## Recommended Implementation Roadmap

### ✅ Phase 1: Menu & Status Infrastructure (20 hours) - COMPLETE
Foundation data structures:
- ✅ MenuItem, Menu, MenuBuilder (v0.2.2)
- ✅ StatusItem, StatusDef, StatusLine, StatusLineBuilder (v0.2.2)

### ✅ Phase 2: List Components (38 hours) - COMPLETE
Proper hierarchy for list and menu controls:
- ✅ ListViewer trait + ListViewerState (16h)
- ✅ MenuViewer trait + MenuViewerState (12h)
- ✅ MenuBox popup container (10h)
- ✅ ListBox refactored to use ListViewer
- ✅ MenuBar refactored to use MenuViewer

**Phase 1-2 Complete: 58 hours implemented, ~270 lines of code eliminated through trait-based architecture**

### ~~Phase 3: Core Collections (80 hours)~~ - SKIPPED (NOT NEEDED)
~~Foundation for all other components~~
- ~~TCollection, TSortedCollection, TNSCollection, TNSSortedCollection~~

**Rationale:** Borland collections were pre-generics workarounds. Rust's `Vec<T>`, `HashMap<K,V>`, etc. are superior. No need to recreate 1990s dynamic arrays.

### ✅ Phase 3: TCluster Refactoring (8 hours) - COMPLETE
Architectural improvement for button groups:
- ✅ Created Cluster trait for RadioButton/CheckBox base
- ✅ Refactored RadioButton to use Cluster trait
- ✅ Refactored CheckBox to use Cluster trait
- ✅ Eliminated duplicate selection/group logic
- ✅ Similar pattern to ListViewer/MenuViewer success

**Implementation Notes:**
- Hybrid trait + helper struct pattern (ClusterState + Cluster trait)
- RadioButton refactored: 202 → 182 lines (20 lines saved)
- CheckBox refactored: 173 → 159 lines (14 lines saved)
- All 7 tests passing (3 CheckBox + 4 RadioButton)
- Common drawing, event handling, and color logic now shared
- Borland-compatible while being idiomatic Rust

### ✅ Phase 4: Sorted Lists (8 hours) - COMPLETE
**Goal**: Extend list infrastructure with sorting and search
- ✅ TSortedListBox with binary search using Vec::sort
- ✅ find_exact() for exact match search
- ✅ find_prefix() for prefix search
- ✅ focus_prefix() for quick keyboard navigation
- ✅ Case-sensitive and case-insensitive modes

**Completed**: 2025-11-03. Uses Vec::sort and Vec::binary_search_by for efficient sorted list management. Eight tests added.

### ✅ Phase 5: History System (24 hours) - COMPLETE
**Goal**: Professional input field enhancement with history
- ✅ HistoryManager - Global history management by ID
- ✅ HistoryList - Stores up to 20 items with deduplication
- ✅ THistory - History dropdown button
- ✅ THistoryViewer - List display using ListViewer trait
- ✅ THistoryWindow - Modal popup for history selection

**Completed**: 2025-11-03. Thread-safe global history management with 14 tests passing.

### Phase 6: File Dialogs (52 hours)
Complete file system UI:
- TFileList, TDirListBox (using Vec for file lists)
- TFileInputLine, TFileInfoPane, TChDirDialog

### Phase 7: Editor Enhancements (32 hours)
Full-featured text editing:
- TFileEditor with search/replace (24h)
- TEditWindow wrapper (8h)

### Phase 8: Application Framework (58 hours)
Enhanced core infrastructure:
- TProgram, TApplication
- TScreen, TDisplay, TMouse, TEventQueue

### ~~Phase 9: Resources & Persistence (90 hours)~~ - NOT NEEDED
~~Professional app development:~~
- ~~Complete streaming system~~ - Use serde instead
- ~~Resource file support~~ - Use JSON/TOML/RON with serde

**Rationale:** Modern Rust has superior serialization (serde) and standard formats. No need to recreate 1990s binary resource files.

### Phase 9: Help System (56 hours)
Context-sensitive help:
- THelpFile, THelpBase
- THelpWindow, THelpViewer

### Phase 10: Polish (262+ hours)
Optional enhancements:
- Color customization
- Calculator, validators
- Configuration system

## Milestone Markers

- **After Phase 2** (58 hours): ✅ COMPLETE - List and menu infrastructure solid
- **After Phase 3** (66 hours): ✅ COMPLETE - Button group controls unified with Cluster trait
- **After Phase 4** (74 hours): ✅ COMPLETE - Sorted lists with binary search
- **After Phase 5** (98 hours): ✅ COMPLETE - History system for professional input fields
- **After Phase 7** (182 hours): Professional editing applications possible
- **After Phase 9** (288 hours): Feature parity with Borland's core framework (minus obsolete pre-Rust features)
- **After Phase 10** (550+ hours): Complete framework with all utilities

## Quick Win Opportunities

These items provide high architectural value for relatively low effort:

1. ~~**TCluster** (8 hours)~~ - ✅ COMPLETE - Refactored RadioButton/CheckBox with trait pattern
2. ~~**TSortedListBox** (8 hours)~~ - ✅ COMPLETE - Binary search sorted lists
3. ~~**TStatusDef/TStatusItem** (7 hours)~~ - ✅ COMPLETE
4. ~~**TMenu/TMenuItem/TSubMenu** (14 hours)~~ - ✅ COMPLETE

**All quick wins completed!** Total: 37 hours of foundational architectural improvements.

## Current Implementation Status (v0.2.3+)

### What We Have
- Basic controls: Button, InputLine, StaticText, Label, CheckBox, RadioButton
- Lists: ListBox with ListViewer trait, SortedListBox with binary search
- Menus: MenuBar with MenuViewer trait, MenuBox popup menus
- Dialogs: Dialog, FileDialog (basic), MsgBox
- Text: Memo, TextView, Editor (basic)
- System: Desktop, StatusLine, Frame, Window, Group
- Utilities: ScrollBar, Scroller, Indicator, ParamText, Background
- Validation: Validator trait, FilterValidator, RangeValidator
- Event system: Three-phase processing, event re-queuing, broadcasts
- **NEW**: List Components (ListViewer, MenuViewer, MenuBox)
- **NEW**: Menu/Status data structures (MenuItem, Menu, MenuBuilder, StatusDef, etc.)
- **NEW**: Cluster trait (base for CheckBox/RadioButton button groups)
- **NEW**: SortedListBox with binary search (find_exact, find_prefix)

### Recent Improvements (TSortedListBox Phase)
- **SortedListBox**: Sorted list control with binary search capabilities
- **Automatic sorting**: Items maintained in sorted order on insertion
- **Binary search**: find_exact() for exact matches, find_prefix() for prefix search
- **Case sensitivity**: Supports both case-sensitive and case-insensitive modes
- **Efficient navigation**: focus_prefix() for quick keyboard navigation
- **8 comprehensive tests**: All passing
- **Example included**: examples/sorted_listbox.rs demonstrates all features

### Modern Rust Advantages
- **No need for TCollection**: Using `Vec<T>` (type-safe, generic, efficient)
- **No need for linked lists**: Vec provides better cache locality
- **No need for streaming system**: serde provides superior serialization
- **No need for resource files**: JSON/TOML/RON are more maintainable
- **No need for string collections**: Vec<String>, HashSet<String>, HashMap work perfectly
- **Trait-based inheritance**: More flexible than C++ class hierarchy
- **Safe memory management**: No manual memory management needed

**Total Obsolete Features Skipped**: 30 components, ~164 hours saved by using modern Rust alternatives

### Architectural Gaps
- No history system for input fields
- No help system infrastructure

## Next Steps

**Recommended: Phase 5 - History System (32 hours)**
- Implement THistory dropdown for input fields
- Add THistoryViewer for displaying history lists
- Create THistoryWindow popup for history selection
- Professional input field enhancement

**Alternative Options:**
- Phase 6: File Dialogs (52 hours) - Enhanced file system UI
- Phase 7: Editor Enhancements (32 hours) - Search/replace functionality

---

*This inventory was generated by analyzing 105 .cc files and 130+ headers from the original Borland Turbo Vision source code.*
