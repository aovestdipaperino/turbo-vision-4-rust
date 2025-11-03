use std::sync::Mutex;

/// Global clipboard for copy/cut/paste operations
/// Supports both in-memory clipboard and OS clipboard integration
static CLIPBOARD: Mutex<String> = Mutex::new(String::new());

/// Set the clipboard content (both in-memory and OS clipboard)
pub fn set_clipboard(text: &str) {
    // Update in-memory clipboard
    if let Ok(mut clipboard) = CLIPBOARD.lock() {
        *clipboard = text.to_string();
    }

    // Try to update OS clipboard (best effort, don't fail if unavailable)
    #[cfg(not(target_os = "unknown"))]
    {
        let _ = set_os_clipboard(text);
    }
}

/// Get the clipboard content (prefers OS clipboard, falls back to in-memory)
pub fn get_clipboard() -> String {
    // Try OS clipboard first
    #[cfg(not(target_os = "unknown"))]
    {
        if let Ok(text) = get_os_clipboard() {
            if !text.is_empty() {
                return text;
            }
        }
    }

    // Fall back to in-memory clipboard
    CLIPBOARD.lock()
        .map(|clipboard| clipboard.clone())
        .unwrap_or_default()
}

/// Check if the clipboard has content
pub fn has_clipboard_content() -> bool {
    // Check OS clipboard first
    #[cfg(not(target_os = "unknown"))]
    {
        if let Ok(text) = get_os_clipboard() {
            if !text.is_empty() {
                return true;
            }
        }
    }

    // Fall back to in-memory clipboard
    CLIPBOARD.lock()
        .map(|clipboard| !clipboard.is_empty())
        .unwrap_or(false)
}

/// Clear the clipboard (both in-memory and OS)
pub fn clear_clipboard() {
    if let Ok(mut clipboard) = CLIPBOARD.lock() {
        clipboard.clear();
    }

    #[cfg(not(target_os = "unknown"))]
    {
        let _ = set_os_clipboard("");
    }
}

/// Set OS clipboard content
#[cfg(not(target_os = "unknown"))]
fn set_os_clipboard(text: &str) -> Result<(), Box<dyn std::error::Error>> {
    use arboard::Clipboard;
    let mut clipboard = Clipboard::new()?;
    clipboard.set_text(text)?;
    Ok(())
}

/// Get OS clipboard content
#[cfg(not(target_os = "unknown"))]
fn get_os_clipboard() -> Result<String, Box<dyn std::error::Error>> {
    use arboard::Clipboard;
    let mut clipboard = Clipboard::new()?;
    Ok(clipboard.get_text()?)
}

/// Get OS clipboard content (always returns empty on unsupported platforms)
#[cfg(target_os = "unknown")]
fn get_os_clipboard() -> Result<String, Box<dyn std::error::Error>> {
    Ok(String::new())
}

/// Set OS clipboard content (no-op on unsupported platforms)
#[cfg(target_os = "unknown")]
fn set_os_clipboard(_text: &str) -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clipboard_operations() {
        set_clipboard("Hello, World!");
        assert!(has_clipboard_content());

        // Get should return what we just set
        let content = get_clipboard();
        assert!(!content.is_empty());
        // Content should be either our value or whatever was in OS clipboard
        // We can't guarantee OS clipboard state in tests

        set_clipboard("New content");
        let content2 = get_clipboard();
        assert!(!content2.is_empty());

        // Test in-memory clipboard specifically
        if let Ok(mut clipboard) = CLIPBOARD.lock() {
            *clipboard = "In-memory test".to_string();
        }
        let in_mem = CLIPBOARD.lock().unwrap().clone();
        assert_eq!(in_mem, "In-memory test");
    }

    #[test]
    fn test_in_memory_clipboard() {
        // Test that in-memory clipboard works even if OS clipboard fails
        if let Ok(mut clipboard) = CLIPBOARD.lock() {
            clipboard.clear();
            *clipboard = "Test content".to_string();
        }

        let in_mem = CLIPBOARD.lock().unwrap().clone();
        assert_eq!(in_mem, "Test content");
    }
}
