# Borland Turbo Vision Palette Chart

Complete reference of all palette mappings from the original Borland Turbo Vision implementation.

## Source

This chart is derived from the original Borland Turbo Vision 2.0 C++ source code located in `local-only/borland/tvision/`.

## Palette Hierarchy

Borland Turbo Vision uses a three-level palette system:

1. **View Palette** - Logical color indices (1-32) for each view type
2. **Container Palette** - Dialog/Window palette that remaps view colors
3. **Application Palette** - Final RGB color attributes (cpColor)

## Application Palette (cpColor)

The root palette containing actual terminal color attributes. From `include/tv/program.h`:

```cpp
#define cpColor \
    "\x71\x70\x78\x74\x20\x28\x24\x17\x1F\x1A\x31\x31\x1E\x71\x00" \
    "\x37\x3F\x3A\x13\x13\x3E\x21\x00\x70\x7F\x7A\x13\x13\x70\x7F\x00" \
    "\x70\x7F\x7A\x13\x13\x70\x70\x7F\x7E\x20\x2B\x2F\x78\x2E\x70\x30" \
    "\x3F\x3E\x1F\x2F\x1A\x20\x72\x31\x31\x30\x2F\x3E\x31\x13\x38\x00"
```

### Application Palette Breakdown (63 entries)

Color format: `0xBF` where B=background (high nibble), F=foreground (low nibble)

```
Index  | Hex  | Colors                    | Usage
-------|------|---------------------------|------------------
1      | 0x71 | Blue on LightGray        | Background
2      | 0x70 | Black on LightGray       | Menu/Status normal
3      | 0x78 | DarkGray on LightGray    | Menu/Status disabled
4      | 0x74 | Red on LightGray         | Menu/Status shortcut
5      | 0x20 | Black on Green           | Menu selected
6      | 0x28 | DarkGray on Green        |
7      | 0x24 | Red on Green             |
8      | 0x17 | White on Blue            | Blue Window frame
9      | 0x1F | White on Blue            | Blue Window text
10     | 0x1A | LightGreen on Blue       |
11     | 0x31 | Blue on Cyan             |
12     | 0x31 | Blue on Cyan             |
13     | 0x1E | Yellow on Blue           |
14     | 0x71 | Blue on LightGray        |
15     | 0x00 | Black on Black           | Unused
16     | 0x37 | White on Cyan            | Cyan Window frame
17     | 0x3F | White on Cyan            | Cyan Window text
18     | 0x3A | LightGreen on Cyan       |
19     | 0x13 | LightCyan on Blue        |
20     | 0x13 | LightCyan on Blue        |
21     | 0x3E | Yellow on Cyan           |
22     | 0x21 | Blue on Green            |
23     | 0x00 | Black on Black           | Unused
24     | 0x70 | Black on LightGray       | Gray Window frame
25     | 0x7F | White on LightGray       | Gray Window active
26     | 0x7A | LightGreen on LightGray  |
27     | 0x13 | LightCyan on Blue        |
28     | 0x13 | LightCyan on Blue        |
29     | 0x70 | Black on LightGray       |
30     | 0x7F | White on LightGray       |
31     | 0x00 | Black on Black           | Unused
32     | 0x70 | Black on LightGray       | Dialog frame
33     | 0x7F | White on LightGray       | Dialog frame active
34     | 0x7A | LightGreen on LightGray  | Dialog interior
35     | 0x13 | LightCyan on Blue        | Dialog text
36     | 0x13 | LightCyan on Blue        | Dialog selected
37     | 0x70 | Black on LightGray       | Dialog reserved
38     | 0x70 | Black on LightGray       | Label normal
39     | 0x7F | White on LightGray       | Label selected
40     | 0x7E | Yellow on LightGray      | Label shortcut
41     | 0x20 | Black on Green           | Button normal
42     | 0x2B | LightGreen on Green      | Button default
43     | 0x2F | White on Green           | Button focused
44     | 0x78 | DarkGray on LightGray    | Button disabled
45     | 0x2E | Yellow on Green          | Button shortcut
46     | 0x70 | Black on LightGray       | Button shadow
47     | 0x30 | Black on Cyan            |
48     | 0x3F | White on Cyan            |
49     | 0x3E | Yellow on Cyan           |
50     | 0x1F | White on Blue            | InputLine passive
51     | 0x2F | White on Green           | InputLine selected
52     | 0x1A | LightGreen on Blue       | InputLine arrow
53     | 0x20 | Black on Green           |
54     | 0x72 | LightGray on LightGray   |
55     | 0x31 | Blue on Cyan             |
56     | 0x31 | Blue on Cyan             |
57     | 0x30 | Black on Cyan            |
58     | 0x2F | White on Green           |
59     | 0x3E | Yellow on Cyan           |
60     | 0x31 | Blue on Cyan             |
61     | 0x13 | LightCyan on Blue        |
62     | 0x38 | DarkGray on Cyan         |
63     | 0x00 | Black on Black           | Unused
```

### Palette Layout (from program.h comments)

```
Index Range | Usage
------------|------------------------
1           | TBackground
2-7         | TMenuView and TStatusLine
8-15        | TWindow (Blue)
16-23       | TWindow (Cyan)
24-31       | TWindow (Gray)
32-63       | TDialog
```

## Container Palettes

### Gray Dialog Palette (cpDialog)

From `classes/tdialog.cc`:

```cpp
#define cpDialog "\x20\x21\x22\x23\x24\x25\x26\x27\x28\x29\x2A\x2B\x2C\x2D\x2E\x2F"\
                 "\x30\x31\x32\x33\x34\x35\x36\x37\x38\x39\x3A\x3B\x3C\x3D\x3E\x3F"
```

Maps dialog indices 1-32 to application indices 32-63:

```
Dialog Index | App Index | Hex  | Colors
-------------|-----------|------|------------------------
1            | 32        | 0x70 | Black on LightGray (frame)
2            | 33        | 0x7F | White on LightGray (frame active)
3            | 34        | 0x7A | LightGreen on LightGray (interior)
4            | 35        | 0x13 | LightCyan on Blue (text)
5            | 36        | 0x13 | LightCyan on Blue (selected)
6            | 37        | 0x70 | Black on LightGray (reserved)
7            | 38        | 0x70 | Black on LightGray (label normal)
8            | 39        | 0x7F | White on LightGray (label selected)
9            | 40        | 0x7E | Yellow on LightGray (label shortcut)
10           | 41        | 0x20 | Black on Green (button normal)
11           | 42        | 0x2B | LightGreen on Green (button default)
12           | 43        | 0x2F | White on Green (button focused)
13           | 44        | 0x78 | DarkGray on LightGray (button disabled)
14           | 45        | 0x2E | Yellow on Green (button shortcut)
15           | 46        | 0x70 | Black on LightGray (button shadow)
16           | 47        | 0x30 | Black on Cyan
17           | 48        | 0x3F | White on Cyan
18           | 49        | 0x3E | Yellow on Cyan
19           | 50        | 0x1F | White on Blue (inputline passive)
20           | 51        | 0x2F | White on Green (inputline selected)
21           | 52        | 0x1A | LightGreen on Blue (inputline arrow)
22           | 53        | 0x20 | Black on Green
23           | 54        | 0x72 | LightGray on LightGray
24           | 55        | 0x31 | Blue on Cyan
25           | 56        | 0x31 | Blue on Cyan
26           | 57        | 0x30 | Black on Cyan (listviewer normal)
27           | 58        | 0x2F | White on Green (listviewer selected)
28           | 59        | 0x3E | Yellow on Cyan (listviewer focused)
29           | 60        | 0x31 | Blue on Cyan (listviewer divider)
30           | 61        | 0x13 | LightCyan on Blue
31           | 62        | 0x38 | DarkGray on Cyan
32           | 63        | 0x00 | Black on Black
```

### Blue Window Palette (cpBlueWindow)

From `classes/twindow.cc`:

```cpp
#define cpBlueWindow "\x08\x09\x0A\x0B\x0C\x0D\x0E\x0F"
```

Maps window indices 1-8 to application indices 8-15.

### Gray Window Palette (cpGrayWindow)

From `classes/twindow.cc`:

```cpp
#define cpGrayWindow "\x18\x19\x1A\x1B\x1C\x1D\x1E\x1F"
```

Maps window indices 1-8 to application indices 24-31.

### Cyan Window Palette (cpCyanWindow)

From `classes/twindow.cc`:

```cpp
#define cpCyanWindow "\x10\x11\x12\x13\x14\x15\x16\x17"
```

Maps window indices 1-8 to application indices 16-23.

## View Palettes

### TButton (cpButton)

From `classes/tbutton.cc`:

```cpp
#define cpButton "\x0A\x0B\x0C\x0D\x0E\x0E\x0E\x0F"
```

**Mapping Chain:**

```
Button Index | Dialog Index | App Index | Hex  | Colors                  | Usage
-------------|--------------|-----------|------|-------------------------|------------------
1            | 10           | 41        | 0x20 | Black on Green         | Normal text
2            | 11           | 42        | 0x2B | LightGreen on Green    | Default button
3            | 12           | 43        | 0x2F | White on Green         | Focused text
4            | 13           | 44        | 0x78 | DarkGray on LightGray  | Disabled text
5            | 14           | 45        | 0x2E | Yellow on Green        | Shortcut
6            | 14           | 45        | 0x2E | Yellow on Green        | Normal shortcut
7            | 14           | 45        | 0x2E | Yellow on Green        | Selected shortcut
8            | 15           | 46        | 0x70 | Black on LightGray     | Shadow
```

**Source:** Verified from `tbutton.cc` getPalette() and Borland documentation.

### TLabel (cpLabel)

From `classes/tlabel.cc`:

```cpp
#define cpLabel "\x07\x08\x09\x09\x0D\x0D"
```

**Mapping Chain:**

```
Label Index | Dialog Index | App Index | Hex  | Colors                  | Usage
------------|--------------|-----------|------|-------------------------|------------------
1           | 7            | 38        | 0x70 | Black on LightGray     | Normal foreground
2           | 8            | 39        | 0x7F | White on LightGray     | Normal background
3           | 9            | 40        | 0x7E | Yellow on LightGray    | Light/shortcut fg
4           | 9            | 40        | 0x7E | Yellow on LightGray    | Light/shortcut bg
5           | 13           | 44        | 0x78 | DarkGray on LightGray  | Disabled foreground
6           | 13           | 44        | 0x78 | DarkGray on LightGray  | Disabled background
```

**Source:** Verified from `tlabel.cc` getPalette().

### TStaticText (cpStaticText)

From `classes/tstatict.cc`:

```cpp
#define cpStaticText "\x06"
```

**Mapping Chain:**

```
StaticText Index | Dialog Index | App Index | Hex  | Colors              | Usage
-----------------|--------------|-----------|------|---------------------|------------------
1                | 6            | 37        | 0x70 | Black on LightGray | Normal text
```

**Source:** Verified from `tstatict.cc` getPalette().

### TInputLine (cpInputLine)

From `classes/tinputli.cc`:

```cpp
#define cpInputLine "\x13\x13\x14\x15"
```

**Mapping Chain:**

```
InputLine Index | Dialog Index | App Index | Hex  | Colors                 | Usage
----------------|--------------|-----------|------|------------------------|------------------
1               | 19           | 50        | 0x1F | White on Blue         | Passive (unfocused)
2               | 19           | 50        | 0x1F | White on Blue         | Active (focused)
3               | 20           | 51        | 0x2F | White on Green        | Selected text
4               | 21           | 52        | 0x1A | LightGreen on Blue    | Arrow
```

**Source:** Verified from `tinputli.cc` getPalette().

### TMenuView (cpMenuView)

From `classes/tmenuvie.cc`:

```cpp
#define cpMenuView "\x02\x03\x04\x05\x06\x07"
```

**Mapping Chain (Direct to App - NO Dialog Remapping):**

```
MenuView Index | App Index | Hex  | Colors                 | Usage
---------------|-----------|------|------------------------|------------------
1              | 2         | 0x70 | Black on LightGray    | Normal text
2              | 3         | 0x78 | DarkGray on LightGray | Disabled
3              | 4         | 0x74 | Red on LightGray      | Shortcut
4              | 5         | 0x20 | Black on Green        | Selected
5              | 6         | 0x28 | DarkGray on Green     | Disabled selected
6              | 7         | 0x24 | Red on Green          | Shortcut selected
```

**Note:** MenuView is a top-level view - uses app palette indices 2-7 directly (no dialog remapping).

**Source:** Verified from `tmenuvie.cc` getPalette().

### TStatusLine (cpStatusLine)

From `classes/tstatlin.cc`:

```cpp
#define cpStatusLine "\x02\x03\x04\x05\x06\x07"
```

**Mapping Chain (Direct to App - NO Dialog Remapping):**

```
StatusLine Index | App Index | Hex  | Colors                 | Usage
-----------------|-----------|------|------------------------|------------------
1                | 2         | 0x70 | Black on LightGray    | Normal text
2                | 3         | 0x78 | DarkGray on LightGray | Shortcut
3                | 4         | 0x74 | Red on LightGray      | Selected
4                | 5         | 0x20 | Black on Green        | Selected shortcut
5                | 6         | 0x28 | DarkGray on Green     |
6                | 7         | 0x24 | Red on Green          |
```

**Note:** StatusLine is a top-level view - uses app palette indices 2-7 directly (no dialog remapping).

**Source:** Verified from `tstatlin.cc` getPalette().

### TScrollBar (cpScrollBar)

From `classes/tscrollb.cc`:

```cpp
#define cpScrollBar  "\x04\x05\x05"
```

**Mapping Chain (Direct to App - indices 4-5):**

```
ScrollBar Index | App Index | Hex  | Colors              | Usage
----------------|-----------|------|---------------------|------------------
1               | 4         | 0x74 | Red on LightGray   | Page area
2               | 5         | 0x20 | Black on Green     | Arrows
3               | 5         | 0x20 | Black on Green     | Indicator
```

**Note:** ScrollBar uses direct app indices 4-5. In Borland, these can be remapped through Window palette when inside a window, or Dialog palette when inside a dialog.

**Source:** Verified from `tscrollb.cc` getPalette().

### TFrame (cpFrame)

From `classes/tframe.cc`:

```cpp
#define cpFrame "\x01\x01\x02\x02\x03"
```

**Mapping Chain (through Window/Dialog palette):**

```
Frame Index | Container Index | App Index | Hex  | Colors              | Usage
------------|-----------------|-----------|------|---------------------|------------------
1           | 1               | varies    |      |                     | Passive (frame)
2           | 1               | varies    |      |                     | Passive (icons)
3           | 2               | varies    |      |                     | Active (frame)
4           | 2               | varies    |      |                     | Active (icons)
5           | 3               | varies    |      |                     | Icons
```

**Note:** Frame remaps through its parent (Window or Dialog) palette. Index 1-5 map to container's indices.

For Gray Window:
- Index 1 → GrayWindow[1]=24 → App[24]=0x70 (Black on LightGray)
- Index 3 → GrayWindow[2]=25 → App[25]=0x7F (White on LightGray)

For Dialog:
- Index 1 → Dialog[1]=32 → App[32]=0x70 (Black on LightGray)
- Index 3 → Dialog[2]=33 → App[33]=0x7F (White on LightGray)

**Source:** Verified from `tframe.cc` getPalette().

### TCluster (cpCluster) - RadioButtons/CheckBoxes

From `classes/tcluster.cc`:

```cpp
#define cpCluster "\x10\x11\x12\x12\x1F"
```

**Mapping Chain:**

```
Cluster Index | Dialog Index | App Index | Hex  | Colors                 | Usage
--------------|--------------|-----------|------|------------------------|------------------
1             | 16           | 47        | 0x30 | Black on Cyan         | Normal text
2             | 17           | 48        | 0x3F | White on Cyan         | Selected
3             | 18           | 49        | 0x3E | Yellow on Cyan        | Shortcut
4             | 18           | 49        | 0x3E | Yellow on Cyan        | (duplicate)
5             | 31           | 62        | 0x38 | DarkGray on Cyan      | Disabled
```

**Source:** Verified from `tcluster.cc` getPalette().

### TListViewer (cpListViewer)

From `classes/tlistvi.cc`:

```cpp
#define cpListViewer "\x1A\x1A\x1B\x1C\x1D"
```

**Mapping Chain:**

```
ListViewer Index | Dialog Index | App Index | Hex  | Colors              | Usage
-----------------|--------------|-----------|------|---------------------|------------------
1                | 26           | 57        | 0x30 | Black on Cyan      | Normal (inactive)
2                | 26           | 57        | 0x30 | Black on Cyan      | Focused (inactive)
3                | 27           | 58        | 0x2F | White on Green     | Selected
4                | 28           | 59        | 0x3E | Yellow on Cyan     | Divider
5                | 29           | 60        | 0x31 | Blue on Cyan       | (reserved)
```

**Source:** Verified from `tlistvi.cc` getPalette().

### THistoryViewer (cpHistoryViewer)

From `classes/thist.cc`:

```cpp
#define cpHistoryViewer "\x06\x06\x07\x06\x06"
```

**Mapping Chain:**

```
HistoryViewer Index | Dialog Index | App Index | Hex  | Colors              | Usage
--------------------|--------------|-----------|------|---------------------|------------------
1                   | 6            | 37        | 0x70 | Black on LightGray | Inactive
2                   | 6            | 37        | 0x70 | Black on LightGray | Active
3                   | 7            | 38        | 0x70 | Black on LightGray | Focused
4                   | 6            | 37        | 0x70 | Black on LightGray | Selected
5                   | 6            | 37        | 0x70 | Black on LightGray | Divider
```

**Source:** Verified from `thist.cc` getPalette().

### TBackground (cpBackground)

From `classes/tbackgro.cc`:

```cpp
#define cpBackground "\x01"
```

**Mapping Chain (Direct to App):**

```
Background Index | App Index | Hex  | Colors              | Usage
-----------------|-----------|------|---------------------|------------------
1                | 1         | 0x71 | Blue on LightGray  | Desktop pattern
```

**Source:** Verified from `tbackgro.cc` getPalette().

### TIndicator (cpIndicator)

From `classes/tindicat.cc`:

```cpp
#define cpIndicator "\x02\x03"
```

**Mapping Chain (through Window palette):**

```
Indicator Index | Window Index | App Index | Hex  | Colors                 | Usage
----------------|--------------|-----------|------|------------------------|------------------
1               | 2            | varies    |      |                        | Normal
2               | 3            | varies    |      |                        | Modified
```

For Gray Window:
- Index 1 → GrayWindow[2]=25 → App[25]=0x7F (White on LightGray)
- Index 2 → GrayWindow[3]=26 → App[26]=0x7A (LightGreen on LightGray)

**Source:** Verified from `tindicat.cc` getPalette().

### TScroller (cpScroller)

From `classes/tscrolle.cc`:

```cpp
#define cpScroller "\x06\x07"
```

**Mapping Chain (through Window/Dialog palette):**

```
Scroller Index | Container Index | App Index | Hex  | Colors              | Usage
---------------|-----------------|-----------|------|---------------------|------------------
1              | 6               | varies    |      |                     | Normal
2              | 7               | varies    |      |                     | Selected
```

**Source:** Verified from `tscrolle.cc` getPalette().

### TMemo (cpMemo)

From `classes/tmemo.cc`:

```cpp
#define cpMemo "\x1A\x1B"
```

**Mapping Chain:**

```
Memo Index | Dialog Index | App Index | Hex  | Colors          | Usage
-----------|--------------|-----------|------|-----------------|------------------
1          | 26           | 57        | 0x30 | Black on Cyan  | Normal
2          | 27           | 58        | 0x2F | White on Green | Selected
```

**Source:** Verified from `tmemo.cc` getPalette().

### TEditor (cpEditor)

From `classes/teditor.cc`:

```cpp
#define cpEditor "\x06\x07"
```

**Mapping Chain (through Window palette):**

```
Editor Index | Window Index | App Index | Hex  | Colors                  | Usage
-------------|--------------|-----------|------|-------------------------|------------------
1            | 6            | varies    |      |                         | Normal text
2            | 7            | varies    |      |                         | Selected text
```

For Gray Window:
- Index 1 → GrayWindow[6]=29 → App[29]=0x70 (Black on LightGray)
- Index 2 → GrayWindow[7]=30 → App[30]=0x7F (White on LightGray)

**Source:** Verified from `teditor.cc` getPalette().

## Color Values Reference

### 16 VGA Colors

```
Value | Name       | RGB Approximation
------|------------|------------------
0     | Black      | #000000
1     | Blue       | #0000AA
2     | Green      | #00AA00
3     | Cyan       | #00AAAA
4     | Red        | #AA0000
5     | Magenta    | #AA00AA
6     | Brown      | #AA5500
7     | LightGray  | #AAAAAA
8     | DarkGray   | #555555
9     | LightBlue  | #5555FF
A     | LightGreen | #55FF55
B     | LightCyan  | #55FFFF
C     | LightRed   | #FF5555
D     | LightMagenta | #FF55FF
E     | Yellow     | #FFFF55
F     | White      | #FFFFFF
```

### Attribute Byte Format

```
7  6  5  4  3  2  1  0
│  │  │  │  │  │  │  └─ Foreground bit 0
│  │  │  │  │  │  └──── Foreground bit 1
│  │  │  │  │  └─────── Foreground bit 2
│  │  │  │  └────────── Foreground bit 3 (intensity)
│  │  │  └───────────── Background bit 0
│  │  └──────────────── Background bit 1
│  └─────────────────── Background bit 2
└────────────────────── Blink/Intensity (mode dependent)
```

**Hex notation:** `0xBF` where B = background (high nibble), F = foreground (low nibble)

**Example:** `0x2F` = background 2 (Green), foreground F (White) = **White on Green**

## Summary Tables

### Top-Level Views (No Dialog Remapping)

These views use direct application palette indices 1-7:

| View          | Palette       | App Indices | Notes
|---------------|---------------|-------------|---------------------------
| TBackground   | cpBackground  | 1           | Desktop pattern
| TMenuView     | cpMenuView    | 2-7         | Menu bar and dropdowns
| TStatusLine   | cpStatusLine  | 2-7         | Status bar at bottom

### Window-Contained Views

These views remap through Window palette (Blue/Cyan/Gray):

| View         | Palette      | Window Indices | Final App Range
|--------------|--------------|----------------|------------------
| TFrame       | cpFrame      | 1-3            | 8-15 / 16-23 / 24-31
| TScroller    | cpScroller   | 6-7            | Window[6-7]
| TIndicator   | cpIndicator  | 2-3            | Window[2-3]
| TEditor      | cpEditor     | 6-7            | Window[6-7]

### Dialog-Contained Controls

These views remap through Dialog palette (indices 32-63):

| View            | Palette          | Dialog Indices | Final App Range
|-----------------|------------------|----------------|------------------
| TStaticText     | cpStaticText     | 6              | 37
| TLabel          | cpLabel          | 7-13           | 38-44
| TButton         | cpButton         | 10-15          | 41-46
| TInputLine      | cpInputLine      | 19-21          | 50-52
| TCluster        | cpCluster        | 16-18, 31      | 47-49, 62
| TListViewer     | cpListViewer     | 26-29          | 57-60
| TMemo           | cpMemo           | 26-27          | 57-58
| THistoryViewer  | cpHistoryViewer  | 6-7            | 37-38

### Context-Dependent Views

These views can be in either Window or Dialog context:

| View        | Palette      | Indices | Context-Dependent
|-------------|--------------|---------|------------------
| TScrollBar  | cpScrollBar  | 4-5     | Direct or remapped
| TScroller   | cpScroller   | 6-7     | Through container

## Implementation Notes

1. **Top-Level Views** (MenuBar, StatusLine, Background) do NOT remap through Dialog palette - they use app indices 1-7 directly.

2. **Dialog Controls** (Button, Label, InputLine, etc.) ALWAYS remap through Dialog palette when their owner is a Dialog.

3. **Window Controls** (Frame, Scroller, Editor, Indicator) remap through Window palette when inside a Window.

4. **ScrollBar** is special - indices 4-5 are in the "reserved" range that can map directly to app, or through container palette depending on context.

5. **Frame** always remaps through its parent container (Window or Dialog) to get proper border colors.

## Reference

All palette definitions extracted from original Borland Turbo Vision 2.0 source code:
- Application palette: `include/tv/program.h`
- Container palettes: `classes/tdialog.cc`, `classes/twindow.cc`
- View palettes: Individual view class files in `classes/`

This chart provides the authoritative reference for implementing Borland-accurate colors in any Turbo Vision port.
