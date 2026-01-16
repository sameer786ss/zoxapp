use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: String,
}

/// Manages conversation context with automatic pruning
pub struct ContextWindow {
    pub max_tokens: usize,
    history: Vec<Message>,
    /// Rough estimate of tokens (1 token ≈ 4 chars)
    estimated_tokens: usize,
}

impl ContextWindow {
    pub fn new(max_tokens: usize) -> Self {
        Self {
            max_tokens,
            history: Vec::new(),
            estimated_tokens: 0,
        }
    }

    /// Add a message to the context window
    pub fn add_message(&mut self, msg: Message) {
        // Rough token estimate: ~4 characters per token
        let msg_tokens = msg.content.len() / 4 + 10; // +10 for role overhead
        self.estimated_tokens += msg_tokens;
        self.history.push(msg);
        self.prune();
    }

    /// Get all messages in history
    pub fn get_history(&self) -> &[Message] {
        &self.history
    }

    /// Get recent N messages
    pub fn get_recent(&self, n: usize) -> &[Message] {
        let start = self.history.len().saturating_sub(n);
        &self.history[start..]
    }

    /// Clear all history
    pub fn clear(&mut self) {
        self.history.clear();
        self.estimated_tokens = 0;
    }

    /// Prune old messages to stay under token limit
    /// Removes messages in pairs to maintain conversation coherence
    fn prune(&mut self) {
        // Keep pruning until we're under the limit
        // Always keep at least 4 messages (2 exchanges) for context
        while self.estimated_tokens > self.max_tokens && self.history.len() > 4 {
            // Remove messages in pairs (starting from index 1 to preserve initial prompt)
            // This keeps tool call/result pairs together
            let mut tokens_to_remove = 0;
            let mut messages_to_remove = 0;
            
            // Calculate tokens for first removable pair (indices 1 and 2 if they exist)
            if self.history.len() > 2 {
                if let Some(msg1) = self.history.get(1) {
                    tokens_to_remove += msg1.content.len() / 4 + 10;
                    messages_to_remove += 1;
                }
                if let Some(msg2) = self.history.get(2) {
                    tokens_to_remove += msg2.content.len() / 4 + 10;
                    messages_to_remove += 1;
                }
            }
            
            if messages_to_remove == 0 {
                break;
            }
            
            // Remove the pair
            for _ in 0..messages_to_remove {
                if self.history.len() > 1 {
                    self.history.remove(1);
                }
            }
            self.estimated_tokens = self.estimated_tokens.saturating_sub(tokens_to_remove);
        }
    }

    /// Get estimated token count
    pub fn token_count(&self) -> usize {
        self.estimated_tokens
    }

    /// Check if context has any messages
    pub fn is_empty(&self) -> bool {
        self.history.is_empty()
    }

    /// Get the number of messages
    pub fn len(&self) -> usize {
        self.history.len()
    }

    /// Get the last message if any
    pub fn last(&self) -> Option<&Message> {
        self.history.last()
    }

    /// Serialize history to JSON for storage
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(&self.history)
    }

    /// Load history from JSON
    pub fn from_json(&mut self, json: &str) -> Result<(), serde_json::Error> {
        self.history = serde_json::from_str(json)?;
        // Recalculate token estimate
        self.estimated_tokens = self.history.iter()
            .map(|m| m.content.len() / 4 + 10)
            .sum();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_and_get() {
        let mut ctx = ContextWindow::new(10000);
        ctx.add_message(Message {
            role: "user".into(),
            content: "Hello".into(),
        });
        ctx.add_message(Message {
            role: "model".into(),
            content: "Hi there!".into(),
        });
        
        assert_eq!(ctx.len(), 2);
        assert_eq!(ctx.get_history()[0].role, "user");
    }

    #[test]
    fn test_pruning() {
        let mut ctx = ContextWindow::new(100); // Very small limit
        
        // Add messages that exceed the limit
        for i in 0..20 {
            ctx.add_message(Message {
                role: "user".into(),
                content: format!("Message {} with some longer content to exceed tokens", i),
            });
        }
        
        // Should have pruned some messages
        assert!(ctx.len() < 20);
    }

    #[test]
    fn test_clear() {
        let mut ctx = ContextWindow::new(10000);
        ctx.add_message(Message { role: "user".into(), content: "test".into() });
        ctx.clear();
        assert!(ctx.is_empty());
    }
}
