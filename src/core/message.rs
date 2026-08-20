//! Ring-buffer message log with turn stamps.

use serde::{Deserialize, Serialize};

pub const MAX_MESSAGES: usize = 400;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Message {
    pub turn: u64,
    pub text: String,
    pub kind: MessageKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MessageKind {
    Normal,
    Combat,
    Good,
    Bad,
    Quest,
    System,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MessageLog {
    messages: Vec<Message>,
}

impl MessageLog {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, turn: u64, kind: MessageKind, text: impl Into<String>) {
        self.messages.push(Message {
            turn,
            text: text.into(),
            kind,
        });
        if self.messages.len() > MAX_MESSAGES {
            self.messages.remove(0);
        }
    }

    /// Messages oldest → newest.
    pub fn all(&self) -> &[Message] {
        &self.messages
    }

    /// Last n messages, oldest first.
    pub fn tail(&self, n: usize) -> Vec<Message> {
        let start = self.messages.len().saturating_sub(n);
        self.messages[start..].to_vec()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn caps_at_max() {
        let mut log = MessageLog::new();
        for i in 0..(MAX_MESSAGES + 50) {
            log.push(i as u64, MessageKind::Normal, format!("m{i}"));
        }
        assert_eq!(log.all().len(), MAX_MESSAGES);
        assert_eq!(log.all().first().unwrap().turn, 50);
    }

    #[test]
    fn tail_order() {
        let mut log = MessageLog::new();
        for i in 0..5u64 {
            log.push(i, MessageKind::Normal, format!("m{i}"));
        }
        let t = log.tail(3);
        assert_eq!(
            t.iter().map(|m| m.turn).collect::<Vec<_>>(),
            vec![2, 3, 4]
        );
    }
}
