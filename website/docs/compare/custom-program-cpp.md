# Theoretical Customizations: Deriving from TProgram

This document shows examples of custom application classes that derive from `TProgram` instead of `TApplication`, demonstrating selective subsystem initialization.

## Background

In Borland Turbo Vision, `TApplication` derives from `TProgram` and adds five subsystems:
1. Memory manager (`InitMemory/DoneMemory`)
2. Video manager (`InitVideo/DoneVideo`)
3. Event manager (`InitEvents/DoneEvents`)
4. System error handler (`InitSysError/DoneSysError`)
5. History list manager (`InitHistory/DoneHistory`)

By deriving directly from `TProgram`, you can selectively enable only the subsystems you need.

---

## Example 1: Minimal Application (No History Lists)

**Use Case:** A simple utility that doesn't need input field history.

```cpp
// C++ (Borland Turbo Vision style)
#include <tv.h>

class TMinimalApp : public TProgram
{
public:
    TMinimalApp();
    virtual ~TMinimalApp();

    static TStatusLine *initStatusLine(TRect r);
    static TMenuBar *initMenuBar(TRect r);
    static TDeskTop *initDeskTop(TRect r);
};

TMinimalApp::TMinimalApp() :
    TProgInit(&TMinimalApp::initStatusLine,
              &TMinimalApp::initMenuBar,
              &TMinimalApp::initDeskTop)
{
    // Initialize only the subsystems we need
    InitMemory();   // Always need memory manager for safety pool
    InitVideo();    // Need video for screen management
    InitEvents();   // Need events for keyboard/mouse
    InitSysError(); // Need error handling

    // SKIP InitHistory() - we don't need input field history!

    // Now call TProgram constructor behavior
    // (This is already done by TProgram's constructor)
}

TMinimalApp::~TMinimalApp()
{
    // Clean up subsystems in reverse order
    DoneSysError();
    DoneEvents();
    DoneVideo();
    DoneMemory();

    // NO DoneHistory() call - we never initialized it
}

TStatusLine *TMinimalApp::initStatusLine(TRect r)
{
    r.a.y = r.b.y - 1;
    return new TStatusLine(r,
        *new TStatusDef(0, 0xFFFF) +
            *new TStatusItem("~F10~ Menu", kbF10, cmMenu) +
            *new TStatusItem("~Alt-X~ Exit", kbAltX, cmQuit)
    );
}

TMenuBar *TMinimalApp::initMenuBar(TRect r)
{
    r.b.y = r.a.y + 1;
    return new TMenuBar(r,
        *new TSubMenu("~F~ile", kbAltF) +
            *new TMenuItem("E~x~it", cmQuit, kbAltX, hcNoContext, "Alt-X")
    );
}

TDeskTop *TMinimalApp::initDeskTop(TRect r)
{
    r.a.y++;
    r.b.y--;
    return new TDeskTop(r);
}

// Usage
int main()
{
    TMinimalApp app;
    app.run();
    return 0;
}
```

**Memory Savings:** Without history lists, saves ~4KB of heap space (default `HistorySize`).

---

## Example 2: Read-Only Display Application

**Use Case:** A log viewer or status monitor that only displays information (no user input).

```cpp
#include <tv.h>

class TDisplayApp : public TProgram
{
public:
    TDisplayApp();
    virtual ~TDisplayApp();

    // Override getEvent to only handle quit keys
    virtual void getEvent(TEvent& event);

    static TStatusLine *initStatusLine(TRect r);
    static TMenuBar *initMenuBar(TRect r) { return 0; } // No menu
    static TDeskTop *initDeskTop(TRect r);
};

TDisplayApp::TDisplayApp() :
    TProgInit(&TDisplayApp::initStatusLine,
              &TDisplayApp::initMenuBar,
              &TDisplayApp::initDeskTop)
{
    InitMemory();   // Need memory manager
    InitVideo();    // Need video for display

    // SKIP InitEvents() - minimal event handling
    // SKIP InitSysError() - we'll handle errors ourselves
    // SKIP InitHistory() - no user input
}

TDisplayApp::~TDisplayApp()
{
    DoneVideo();
    DoneMemory();
}

void TDisplayApp::getEvent(TEvent& event)
{
    // Simplified event handling - only check for quit keys
    TProgram::getEvent(event);

    if (event.what == evKeyDown)
    {
        if (event.keyDown.keyCode == kbEsc ||
            event.keyDown.keyCode == kbAltX)
        {
            event.what = evCommand;
            event.message.command = cmQuit;
        }
    }
}

TStatusLine *TDisplayApp::initStatusLine(TRect r)
{
    r.a.y = r.b.y - 1;
    return new TStatusLine(r,
        *new TStatusDef(0, 0xFFFF) +
            *new TStatusItem("~Esc~ Exit", kbEsc, cmQuit)
    );
}

TDeskTop *TDisplayApp::initDeskTop(TRect r)
{
    r.a.y = 1;  // No menu bar
    r.b.y--;
    return new TDeskTop(r);
}

// Usage: Display-only application
int main()
{
    TDisplayApp app;

    // Create a read-only window
    TRect r(10, 5, 70, 20);
    TWindow *w = new TWindow(r, "System Status", wnNoNumber);

    // Add static text (no input controls)
    TStaticText *st = new TStaticText(r, "Monitoring system...");
    w->insert(st);

    app.deskTop->insert(w);
    app.run();

    return 0;
}
```

---

## Example 3: Embedded System Application

**Use Case:** Running on embedded hardware with limited memory and no persistent storage.

```cpp
#include <tv.h>

class TEmbeddedApp : public TProgram
{
private:
    bool lowMemoryMode;

public:
    TEmbeddedApp(unsigned availableMemory);
    virtual ~TEmbeddedApp();

    virtual void idle();
    virtual void outOfMemory();

    static TStatusLine *initStatusLine(TRect r);
    static TMenuBar *initMenuBar(TRect r);
    static TDeskTop *initDeskTop(TRect r);
};

TEmbeddedApp::TEmbeddedApp(unsigned availableMemory) :
    TProgInit(&TEmbeddedApp::initStatusLine,
              &TEmbeddedApp::initMenuBar,
              &TEmbeddedApp::initDeskTop)
{
    // Determine which subsystems we can afford
    lowMemoryMode = (availableMemory < 64 * 1024); // Less than 64KB

    if (lowMemoryMode)
    {
        // Minimal subsystems
        InitMemory();  // Essential
        InitVideo();   // Essential for display
        InitEvents();  // Essential for input

        // SKIP InitSysError() - handle inline
        // SKIP InitHistory() - no room for history
    }
    else
    {
        // Full subsystems
        InitMemory();
        InitVideo();
        InitEvents();
        InitSysError();
        InitHistory();
    }
}

TEmbeddedApp::~TEmbeddedApp()
{
    if (!lowMemoryMode)
    {
        DoneHistory();
        DoneSysError();
    }

    DoneEvents();
    DoneVideo();
    DoneMemory();
}

void TEmbeddedApp::idle()
{
    TProgram::idle();

    // In embedded systems, yield CPU to other tasks
    // or enter low-power mode during idle
    #ifdef EMBEDDED_RTOS
        taskYield();
    #endif
}

void TEmbeddedApp::outOfMemory()
{
    // Embedded-specific out-of-memory handling
    messageBox("Critical: Out of memory! Restarting...",
               mfError | mfOKButton);

    // In embedded systems, might trigger a watchdog reset
    #ifdef EMBEDDED_SYSTEM
        systemReset();
    #else
        exit(1);
    #endif
}

TStatusLine *TEmbeddedApp::initStatusLine(TRect r)
{
    r.a.y = r.b.y - 1;
    return new TStatusLine(r,
        *new TStatusDef(0, 0xFFFF) +
            *new TStatusItem("", kbF10, cmMenu) +
            *new TStatusItem("Quit", kbAltX, cmQuit)
    );
}

TMenuBar *TEmbeddedApp::initMenuBar(TRect r)
{
    r.b.y = r.a.y + 1;
    return new TMenuBar(r,
        *new TSubMenu("System", kbAltS) +
            *new TMenuItem("Status", cmStatus, kbF1) +
            *new TMenuItem("Reset", cmReset, kbCtrlR) +
            *new TMenuLine() +
            *new TMenuItem("Exit", cmQuit, kbAltX)
    );
}

TDeskTop *TEmbeddedApp::initDeskTop(TRect r)
{
    r.a.y++;
    r.b.y--;
    return new TDeskTop(r);
}
```

---

## Example 4: Testing Framework Application

**Use Case:** Automated testing harness that doesn't need full UI subsystems.

```cpp
#include <tv.h>

class TTestApp : public TProgram
{
private:
    bool headlessMode;
    int testsPassed;
    int testsFailed;

public:
    TTestApp(bool headless = false);
    virtual ~TTestApp();

    void runTest(const char *name, bool (*testFunc)());
    void printResults();

    static TStatusLine *initStatusLine(TRect r);
    static TMenuBar *initMenuBar(TRect r);
    static TDeskTop *initDeskTop(TRect r);
};

TTestApp::TTestApp(bool headless) :
    TProgInit(&TTestApp::initStatusLine,
              &TTestApp::initMenuBar,
              &TTestApp::initDeskTop),
    headlessMode(headless),
    testsPassed(0),
    testsFailed(0)
{
    InitMemory();  // Need memory management

    if (!headlessMode)
    {
        InitVideo();   // Only init video if not headless
    }

    InitEvents();  // Need event system for test coordination

    // SKIP InitSysError() - tests handle their own errors
    // SKIP InitHistory() - not needed for testing
}

TTestApp::~TTestApp()
{
    DoneEvents();

    if (!headlessMode)
    {
        DoneVideo();
    }

    DoneMemory();
}

void TTestApp::runTest(const char *name, bool (*testFunc)())
{
    if (!headlessMode)
    {
        // Visual feedback
        TRect r(10, 5, 70, 10);
        TWindow *w = new TWindow(r, name, wnNoNumber);
        deskTop->insert(w);
    }

    bool result = testFunc();

    if (result)
    {
        testsPassed++;
    }
    else
    {
        testsFailed++;
    }

    if (!headlessMode)
    {
        // Remove test window
        deskTop->current->setState(sfVisible, False);
    }
}

void TTestApp::printResults()
{
    printf("Tests Passed: %d\n", testsPassed);
    printf("Tests Failed: %d\n", testsFailed);
}

TStatusLine *TTestApp::initStatusLine(TRect r)
{
    r.a.y = r.b.y - 1;
    return new TStatusLine(r,
        *new TStatusDef(0, 0xFFFF) +
            *new TStatusItem("Running tests...", kbNoKey, cmNone)
    );
}

TMenuBar *TTestApp::initMenuBar(TRect r)
{
    return 0; // No menu in test mode
}

TDeskTop *TTestApp::initDeskTop(TRect r)
{
    r.a.y = 1;
    r.b.y--;
    return new TDeskTop(r);
}

// Usage
bool testAddition() { return (2 + 2 == 4); }
bool testSubtraction() { return (5 - 3 == 2); }

int main(int argc, char *argv[])
{
    bool headless = (argc > 1 && strcmp(argv[1], "--headless") == 0);

    TTestApp app(headless);

    app.runTest("Addition Test", testAddition);
    app.runTest("Subtraction Test", testSubtraction);

    app.printResults();

    return 0;
}
```

---

## Summary

These examples demonstrate why `TProgram` exists as an intermediate class:

| Example | Skipped Subsystems | Benefit |
|---------|-------------------|---------|
| Minimal App | History | ~4KB memory savings |
| Display-Only | Events, SysError, History | Simpler code, faster startup |
| Embedded | SysError, History (conditional) | Adapts to memory constraints |
| Test Framework | Video (conditional), SysError, History | Headless testing support |

**In Practice:** These use cases are rare. Most developers just used `TApplication` because:
1. The memory overhead is negligible on modern systems
2. You almost always need all subsystems
3. Selective initialization is error-prone
4. The flexibility isn't worth the complexity

This is why the Rust port correctly merges `TProgram` and `TApplication` into a single `Application` struct - the theoretical flexibility of `TProgram` provides no practical value in modern development.
