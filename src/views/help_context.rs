// (C) 2025 - Enzo Lombardi
// help_context - Context-sensitive help support
//
// Maps help context IDs to help topics.
// Allows F1 key to show context-appropriate help.

use std::collections::HashMap;

/// Help context ID type
pub type HelpContextId = u16;

/// Special help context IDs
pub const HC_NO_CONTEXT: HelpContextId = 0;

/// HelpContext - Maps context IDs to help topic IDs
pub struct HelpContext {
    /// Map from context ID to topic ID
    map: HashMap<HelpContextId, String>,
}

impl HelpContext {
    /// Create a new help context manager
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    /// Register a help context
    pub fn register(&mut self, context_id: HelpContextId, topic_id: &str) {
        self.map.insert(context_id, topic_id.to_string());
    }

    /// Get the topic ID for a context
    pub fn get_topic(&self, context_id: HelpContextId) -> Option<&str> {
        self.map.get(&context_id).map(|s| s.as_str())
    }

    /// Check if a context is registered
    pub fn has_context(&self, context_id: HelpContextId) -> bool {
        self.map.contains_key(&context_id)
    }

    /// Remove a context registration
    pub fn unregister(&mut self, context_id: HelpContextId) -> bool {
        self.map.remove(&context_id).is_some()
    }

    /// Clear all registrations
    pub fn clear(&mut self) {
        self.map.clear();
    }

    /// Get the number of registered contexts
    pub fn count(&self) -> usize {
        self.map.len()
    }
}

impl Default for HelpContext {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_help_context_new() {
        let ctx = HelpContext::new();
        assert_eq!(ctx.count(), 0);
    }

    #[test]
    fn test_register_and_get() {
        let mut ctx = HelpContext::new();
        ctx.register(100, "file-open");
        ctx.register(101, "file-save");

        assert_eq!(ctx.get_topic(100), Some("file-open"));
        assert_eq!(ctx.get_topic(101), Some("file-save"));
        assert_eq!(ctx.get_topic(102), None);
    }

    #[test]
    fn test_has_context() {
        let mut ctx = HelpContext::new();
        ctx.register(100, "test-topic");

        assert!(ctx.has_context(100));
        assert!(!ctx.has_context(101));
    }

    #[test]
    fn test_unregister() {
        let mut ctx = HelpContext::new();
        ctx.register(100, "test-topic");
        assert!(ctx.has_context(100));

        assert!(ctx.unregister(100));
        assert!(!ctx.has_context(100));

        // Unregistering again returns false
        assert!(!ctx.unregister(100));
    }

    #[test]
    fn test_clear() {
        let mut ctx = HelpContext::new();
        ctx.register(100, "topic1");
        ctx.register(101, "topic2");
        assert_eq!(ctx.count(), 2);

        ctx.clear();
        assert_eq!(ctx.count(), 0);
    }

    #[test]
    fn test_overwrite_registration() {
        let mut ctx = HelpContext::new();
        ctx.register(100, "topic1");
        ctx.register(100, "topic2");

        assert_eq!(ctx.get_topic(100), Some("topic2"));
        assert_eq!(ctx.count(), 1);
    }
}
