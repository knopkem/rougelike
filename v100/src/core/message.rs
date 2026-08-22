//! Message log: a ring buffer of recent game messages, newest last.

use serde::{Deserialize, Serialize};

/// Maximum number of messages retained in the log.
pub const MAX_MESSAGES: usize = 256;

/// A single message with a severity for coloring.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Severity {
    /// Normal narrative text.
    Normal,
    /// Good news (level up, quest complete, pickup).
    Good,
    /// Bad news (damage, traps, death).
    Bad,
    /// System / UI notices (stairs, doors, shop).
    System,
    /// Magic / special effects.
    Magic,
}

/// A message in the log.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Message {
    pub text: String,
    pub severity: Severity,
    /// The turn the message was logged (for the history panel).
    pub turn: u64,
}

/// A bounded ring buffer of messages.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MessageLog {
    messages: Vec<Message>,
}

impl MessageLog {
    pub fn new() -> Self {
        Self::default()
    }

    /// Append a message, evicting the oldest if over capacity.
    pub fn push(&mut self, text: impl Into<String>, severity: Severity, turn: u64) {
        self.messages.push(Message {
            text: text.into(),
            severity,
            turn,
        });
        if self.messages.len() > MAX_MESSAGES {
            let excess = self.messages.len() - MAX_MESSAGES;
            self.messages.drain(0..excess);
        }
    }

    /// The most recent `n` messages, oldest first.
    pub fn recent(&self, n: usize) -> &[Message] {
        let start = self.messages.len().saturating_sub(n);
        &self.messages[start..]
    }

    /// All messages (for the history panel), oldest first.
    pub fn all(&self) -> &[Message] {
        &self.messages
    }

    pub fn len(&self) -> usize {
        self.messages.len()
    }

    pub fn is_empty(&self) -> bool {
        self.messages.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn evicts_oldest_over_capacity() {
        let mut log = MessageLog::new();
        for i in 0..(MAX_MESSAGES + 10) {
            log.push(format!("msg {i}"), Severity::Normal, i as u64);
        }
        assert_eq!(log.len(), MAX_MESSAGES);
        assert_eq!(log.all()[0].text, "msg 10");
        assert_eq!(
            log.all().last().unwrap().text,
            format!("msg {}", MAX_MESSAGES + 9)
        );
    }

    #[test]
    fn recent_returns_tail() {
        let mut log = MessageLog::new();
        for i in 0..10 {
            log.push(format!("m{i}"), Severity::Normal, i);
        }
        let r = log.recent(3);
        assert_eq!(r.len(), 3);
        assert_eq!(r[0].text, "m7");
    }
}
