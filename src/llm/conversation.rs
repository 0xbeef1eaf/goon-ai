use std::collections::VecDeque;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Message {
    pub role: String, // "system", "user", "assistant", "error"
    pub content: String,
}

#[allow(dead_code)]
pub struct ConversationManager {
    history: VecDeque<Message>,
    max_history: usize,
}

impl ConversationManager {
    #[allow(dead_code)]
    pub fn new(max_history: usize) -> Self {
        Self {
            history: VecDeque::new(),
            max_history,
        }
    }

    #[allow(dead_code)]
    pub fn add_message(&mut self, role: &str, content: &str) {
        if self.history.len() >= self.max_history {
            self.history.pop_front();
        }
        self.history.push_back(Message {
            role: role.to_string(),
            content: content.to_string(),
        });
    }

    #[allow(dead_code)]
    pub fn get_history(&self) -> &VecDeque<Message> {
        &self.history
    }

    #[allow(dead_code)]
    pub fn clear(&mut self) {
        self.history.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_conversation_manager() {
        let mut mgr = ConversationManager::new(3);
        mgr.add_message("user", "1");
        mgr.add_message("assistant", "2");
        mgr.add_message("user", "3");

        assert_eq!(mgr.get_history().len(), 3);
        assert_eq!(mgr.get_history()[0].content, "1");

        mgr.add_message("assistant", "4");
        assert_eq!(mgr.get_history().len(), 3);
        assert_eq!(mgr.get_history()[0].content, "2");
        assert_eq!(mgr.get_history()[2].content, "4");
    }
}
