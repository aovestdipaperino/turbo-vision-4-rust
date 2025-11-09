# Button and Palette System - Visual Guide

## 1. Button Appearance by State

```
NORMAL STATE (Index 1 -> Black on Green)
┌─────────────┐
│   [ OK ]    │  ← Black text on Green background
└─────────────┘

DEFAULT STATE (Index 2 -> LightGreen on Green) 
┌═════════════┐
║  [ OK ]     ║  ← Brighter green text on Green, double border shows default
└═════════════┘

FOCUSED STATE (Index 3 -> White on Green)
┏━━━━━━━━━━━━━┓
┃   [ OK ]    ┃  ← White text on Green, bold border shows focus
┗━━━━━━━━━━━━━┛

DISABLED STATE (Index 4 -> DarkGray on Green)
┌─────────────┐
│  ░OK░       │  ← DarkGray text on Green, dimmed appearance
└─────────────┘
    ▐▌        ← Shadow (Index 8 -> LightGray on DarkGray)
```

## 2. Palette Remapping Chain (Visualization)

```
┌────────────────────────────────────────────────────────────────┐
│                        COLOR RESOLUTION                         │
└────────────────────────────────────────────────────────────────┘

USER CALLS:                button.map_color(1)
                                   │
                                   ▼
STEP 1: Apply Button Palette
    ┌─────────────┐
    │ CP_BUTTON   │
    ├─────────────┤
    │ [1] → 13    │  ◄─── Input: 1, Output: 13
    │ [2] → 13    │
    │ [3] → 14    │
    │ [4] → 14    │
    │ [7] → 15    │
    │ [8] → 9     │
    └─────────────┘
                                   │
                                   ▼ (color = 13)
STEP 2: Check Dialog Range (32-63)
    ┌─────────────────────┐
    │ Is 13 >= 32?        │  ◄─── NO! Skip dialog palette
    │ Is 13 < 64?         │
    └─────────────────────┘
                                   │
                                   ▼ (color = 13)
STEP 3: Apply App Palette
    ┌──────────────────┐
    │ CP_APP_COLOR     │
    ├──────────────────┤
    │ [1] → 0x71       │
    │ [2] → 0x70       │
    │ ...              │
    │ [13]→ 0x1E ◄─────── Lookup at index 13
    │ ...              │
    └──────────────────┘
                                   │
                                   ▼ (raw byte = 0x1E)
STEP 4: Decode Attribute
    ┌────────────────────────┐
    │ 0x1E = 0b00011110      │
    │ Foreground: 0xE = 14   │
    │ Background: 0x1 = 1    │
    │ TvColor(14) = Yellow   │
    │ TvColor(1) = Blue      │
    └────────────────────────┘
                                   │
                                   ▼
RESULT:                  Attr { Yellow on Blue }
```

## 3. Button State Machine

```
                        ┌──────────────────┐
                        │  BUTTON CREATED  │
                        │                  │
                        │ Check command    │
                        │ in global set    │
                        └──────────────────┘
                                 │
                    ┌────────────┴────────────┐
                    ▼                         ▼
            ┌──────────────┐        ┌──────────────┐
            │  DISABLED    │        │   ENABLED    │
            │              │        │              │
            │ Ignores:     │        │ Handles:     │
            │ • Mouse      │        │ • Mouse      │
            │ • Keyboard   │        │ • Keyboard   │
            │ Processes:   │        │ Processes:   │
            │ • Broadcasts │        │ • Broadcasts │
            └──────────────┘        └──────────────┘
                    │                         │
                    │ CM_COMMAND_             │ CM_COMMAND_
                    │ SET_CHANGED             │ SET_CHANGED
                    │                         │
                    └────────────┬────────────┘
                                 ▼
                        ┌──────────────────┐
                        │ Update disabled  │
                        │ state from       │
                        │ global set       │
                        └──────────────────┘
```

## 4. Button Color Index Flow

```
STATE DECISION                    PALETTE INDEX           FINAL COLOR
────────────────────────────────────────────────────────────────────

┌─ Disabled?                      Index 4                 DarkGray
│  Yes ─────────────────────────────────────────────────────────────►
│                                                         on Green
│
│  No ─────┬─ Focused?            Index 3                 White
│          │  Yes ─────────────────────────────────────────────────►
│          │                                              on Green
│          │
│          │  No ─────┬─ Default?  Index 2                LightGreen
│          │          │  Yes ──────────────────────────────────────►
│          │          │                                  on Green
│          │          │
│          │          │  No ──────  Index 1              Black
└──────────┴──────────┴────────────────────────────────────────────►
                                                         on Green

SHORTCUTS: Always use Index 7 (unless disabled, then Index 4)
SHADOW:    Always use Index 8 (LightGray on DarkGray)
```

## 5. Menu Palette Hierarchy

```
                    ┌──────────────────┐
                    │  CP_MENU_BAR     │
                    │  (4 entries)     │
                    ├──────────────────┤
                    │ [1] → 2          │
                    │ [2] → 39         │
                    │ [3] → 3          │
                    │ [4] → 4          │
                    └──────────────────┘
                            │
           ┌────────────────┼────────────────┐
           ▼                ▼                ▼
        Index 1         Index 2           Index 3
        Normal          Selected          Disabled
        ↓               ↓                 ↓
    [2] Menu        [39] Menu         [3] Menu
    Black/Light     White/Green       DarkGray/Light
    Gray            (Highlighted)     Gray


    ┌────────────────────────────────────────┐
    │ Applied by MenuBar.draw_dropdown()     │
    │ and MenuBox.draw()                     │
    │                                        │
    │ Constants added for clarity:           │
    │ const MENU_NORMAL: u8 = 1;             │
    │ const MENU_SELECTED: u8 = 2;           │
    │ const MENU_DISABLED: u8 = 3;           │
    │ const MENU_SHORTCUT: u8 = 4;           │
    └────────────────────────────────────────┘
```

## 6. Color Palette Structure

```
┌──────────────────────────────────────────────────────────────┐
│                  THREE-LAYER PALETTE SYSTEM                  │
└──────────────────────────────────────────────────────────────┘

LAYER 1: COMPONENT PALETTES (View Level)
┌─────────────────────────────────────────┐
│ CP_BUTTON (8 entries)                   │ ◄─ Button.get_palette()
│ Maps: 1-8 → 9-16 (dialog indices)       │
└─────────────────────────────────────────┘
┌─────────────────────────────────────────┐
│ CP_MENU_BAR (4 entries)                 │ ◄─ MenuBar.get_palette()
│ Maps: 1-4 → 2,39,3,4 (app indices)      │
└─────────────────────────────────────────┘
┌─────────────────────────────────────────┐
│ CP_LISTBOX (4 entries)                  │ ◄─ ListBox.get_palette()
│ Maps: 1-4 → 26-28 (dialog indices)      │
└─────────────────────────────────────────┘

                    ▼

LAYER 2: DIALOG PALETTE (Parent Level)
┌──────────────────────────────────────┐
│ CP_GRAY_DIALOG (32 entries)          │ ◄─ Parent of buttons in dialogs
│ Maps: 32-63 → app palette indices    │
└──────────────────────────────────────┘
                    ▼

LAYER 3: APPLICATION PALETTE (Root Level)
┌──────────────────────────────────────┐
│ CP_APP_COLOR (85 entries)            │ ◄─ Final color definitions
│ Maps: 1-85 → actual 0xBF attributes  │
│       (foreground << 4 | background) │
└──────────────────────────────────────┘
```

## 7. Button in Dialog Context

```
┌─────────────────────────────────────────────────────────────────┐
│ DIALOG (index range 16-31 or parent dialog palette)             │
│                                                                 │
│  Label:                                      Button:            │
│  ┌─────────────────┐                        ┌──────────┐       │
│  │ Name: _______   │                        │  [  OK  ]│       │
│  └─────────────────┘                        └──────────┘       │
│      ↑                                            ↑             │
│      │                                            │             │
│  CP_LABEL (3 entries)                    CP_BUTTON (8 entries) │
│  Maps: 1-3 → 7,8,9                       Maps: 1-8 → 13...    │
│              (directly to app)                (to dialog)       │
│              ↓                                ↓                 │
│         CP_APP_COLOR                    CP_GRAY_DIALOG (32-63) │
│         [7], [8], [9]                           ↓              │
│              ↓                             CP_APP_COLOR         │
│         [13], [14], [36]                       ↓               │
│              ↓                             Final Attr          │
│         Final Attr                                             │
│                                                                 │
│  Note: Dialog buttons use index remap via:                     │
│  View (1→13) → Dialog (13→36) → App (36→0x20)                │
└─────────────────────────────────────────────────────────────────┘
```

## 8. Recent Changes Map

```
palette-owner BRANCH

┌─────────────────────────────────────────────────────────────┐
│ Commit: a0df6b7 "Add comprehensive palette documentation"  │
│  ├─ Updated palette.rs documentation                       │
│  └─ Explained hierarchy and usage                          │
└─────────────────────────────────────────────────────────────┘
                          ▲
                          │
┌─────────────────────────────────────────────────────────────┐
│ Commit: 9b74935 "Remove unsafe pointer casting..."         │
│  ├─ Removed owner chain traversal (unsafe)                 │
│  ├─ Implemented fixed palette chain                        │
│  └─ Added dialog range detection (32-63)                   │
└─────────────────────────────────────────────────────────────┘
                          ▲
                          │
┌─────────────────────────────────────────────────────────────┐
│ Commit: 68899be "Palette indirect implementation"          │
│  ├─ Initial palette system                                 │
│  ├─ Set up CP_BUTTON, CP_MENU_BAR                          │
│  └─ Implemented map_color() in View trait                  │
└─────────────────────────────────────────────────────────────┘

CURRENT STATUS (palette-owner branch):
├─ button.rs:       Modified (added logging)
├─ menu_bar.rs:     Modified (added constants, formatting)
├─ menu_box.rs:     Modified (added constants, bug fix)
├─ view.rs:         Modified (documentation, formatting)
└─ palette.rs:      Not modified (stable)
```

## 9. Debug Log Trace Example

```
calc.log output when drawing button in different states:

NORMAL STATE:
─────────────
Button 'OK' draw START, owner=None
  Calling map_color(1)...
    Remapped 1 -> 13 via own palette
    Using CP_APP_COLOR[13]
  map_color(1) OK
  Calling map_color(8) for shadow...
    Remapped 8 -> 9 via own palette
    Using CP_APP_COLOR[9]
  map_color(8) OK

DISABLED STATE:
───────────────
Button 'OK' draw START, owner=None
  Calling map_color(4)...
    Remapped 4 -> 14 via own palette
    Using CP_APP_COLOR[14]
  map_color(4) OK
  Calling map_color(8) for shadow...
    Remapped 8 -> 9 via own palette
    Using CP_APP_COLOR[9]
  map_color(8) OK

FOCUSED STATE:
──────────────
Button 'OK' draw START, owner=None
  Calling map_color(3)...
    Remapped 3 -> 14 via own palette
    Using CP_APP_COLOR[14]
  map_color(3) OK
  Calling map_color(8) for shadow...
    [similar trace]
```

## 10. Event Handling Flow

```
Button receives event:

┌─────────────────────────────┐
│ Event received              │
└─────────────────────────────┘
          │
          ▼
┌─────────────────────────────┐
│ Is it a Broadcast?          │
├─────────────────────────────┤
│ YES                    NO   │
└─┬───────────────────────────┘
  │                           │
  ▼                           ▼
┌──────────────────┐   ┌─────────────────┐
│ Handle broadcast │   │ Is disabled?    │
│ (even if         │   ├─────────────────┤
│  disabled!)      │   │ YES       NO    │
└──────────────────┘   └─┬───────────────┘
  │                      │               │
  ▼                      ▼               ▼
┌──────────────────┐  ┌────────┐   ┌──────────────┐
│ Update from      │  │ Return │   │ Handle:      │
│ global command   │  │ (ignore)│   │ • Keyboard  │
│ set              │  │        │   │ • Mouse     │
└──────────────────┘  └────────┘   └──────────────┘
  │
  ▼
┌──────────────────┐
│ DON'T clear      │
│ broadcast!       │
│ (propagate to    │
│  other views)    │
└──────────────────┘
```

## 11. Owner Chain (Potential Future)

```
CURRENT IMPLEMENTATION:
───────────────────────

Button
  └─ owner: Option<*const View>
       └─ Parent view pointer
            └─ Used for: logging, potential palette chain

VIEW HIERARCHY:
───────────────

Desktop (root)
  │
  ├─ MenuBar
  │
  ├─ Window
  │   │
  │   └─ Dialog
  │       │
  │       ├─ Button (owner → Dialog)
  │       ├─ InputLine (owner → Dialog)
  │       └─ StaticText (owner → Dialog)
  │
  └─ StatusLine

PALETTE CHAIN (current):
────────────────────────
Any View → Dialog (if 32-63) → App

PALETTE CHAIN (potential with owner):
──────────────────────────────────────
Button → Dialog → Window → Desktop → App

This would allow:
• Different window palettes
• Cascading dialog palettes
• Custom desktop color schemes
```

## 12. Color Byte Breakdown

```
ATTRIBUTE BYTE: 0xBF (example: Blue fg on Brown bg)

┌─────────────────────────────┐
│ BYTE: 0xBF = 10111111 (binary)
├─────────────────────────────┤
│                             │
│ ┌──────────┐    ┌────────┐ │
│ │1011      │    │1111    │ │
│ │Background│    │Foreground
│ │(upper 4) │    │(lower 4)  │
│ └──────────┘    └────────┘ │
│    = 0xB               = 0xF   │
│    = 11                = 15    │
│    = Brown             = White │
│                             │
└─────────────────────────────┘

COMMON BUTTON ENCODINGS:
──────────────────────────

0x20 = Normal        │ 0b00100000
     Black on Green  │ Green(2) << 4 | Black(0)

0x2F = Focused       │ 0b00101111  
     White on Green  │ Green(2) << 4 | White(15)

0x2A = Default       │ 0b00101010
     LightGreen      │ Green(2) << 4 | LightGreen(10)
     on Green

0x28 = Disabled      │ 0b00101000
     DarkGray on     │ Green(2) << 4 | DarkGray(8)
     Green

0x87 = Shadow        │ 0b10000111
     LightGray on    │ DarkGray(8) << 4 | LightGray(7)
     DarkGray
```

## 13. Code Navigation Cheat Sheet

```
FIND THIS:              IN THIS FILE:          AT THIS LOCATION:
─────────────────────────────────────────────────────────────────

Button structure        button.rs              lines 1-30
Button::new()          button.rs              lines 25-45
Button::draw()         button.rs              lines 80-140
Handle events          button.rs              lines 145-265
Broadcast handling     button.rs              lines 185-210 ***
Get palette            button.rs              last 5 lines
Map color logic        view.rs                lines 300-365
Button palette def     palette.rs             CP_BUTTON const
Menu palette def       palette.rs             CP_MENU_BAR const
App palette def        palette.rs             CP_APP_COLOR const
Dialog palette def     palette.rs             CP_GRAY_DIALOG const
Color constants        palette.rs             colors module
TvColor enum           palette.rs             lines 5-40
Attr struct            palette.rs             lines 50-80
Menu constants (new)   menu_bar.rs            lines 16-19
Menu constants (new)   menu_box.rs            lines 14-17
Menu color usage       menu_bar.rs            lines 159-162
Menu color usage       menu_box.rs            lines 154-157
Owner tracking         view.rs                lines 280-295
Test suite             button.rs              lines 350-650
Regression test        button.rs              lines 430-480 ***

*** = Critical for understanding recent changes
```

## 14. Testing Checklist

```
TO VERIFY PALETTE SYSTEM:
─────────────────────────

[ ] Button draws with correct color in each state:
    ✓ Normal (Black on Green)
    ✓ Focused (White on Green)
    ✓ Default (LightGreen on Green)
    ✓ Disabled (DarkGray on Green)

[ ] Button colors change on focus/disable:
    ✓ Gaining focus changes to White on Green
    ✓ Losing focus changes back to Black on Green
    ✓ Disabling changes to DarkGray on Green

[ ] Broadcast handling works:
    ✓ Disabled button receives CM_COMMAND_SET_CHANGED
    ✓ Button becomes enabled after broadcast
    ✓ Button becomes disabled when command disabled

[ ] Menu colors display correctly:
    ✓ Normal items (Black on LightGray)
    ✓ Selected items (White on Green)
    ✓ Disabled items (DarkGray on LightGray)
    ✓ Shortcut keys (Red on LightGray)

[ ] Palette constants used:
    ✓ menu_bar.rs uses MENU_* constants
    ✓ menu_box.rs uses MENU_* constants
    ✓ No magic numbers in palette lookups

[ ] Debug logging traces palette chain:
    ✓ calc.log shows palette remapping
    ✓ Index transforms visible in log
    ✓ Final app palette lookup shown
```

This visual guide provides quick reference for understanding the button and palette system architecture!
