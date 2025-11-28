use crate::config::pack::PackConfig;
use crate::config::settings::User;
use crate::llm::conversation::ConversationManager;
use ollama_rs::generation::chat::{ChatMessage, MessageRole};

#[allow(dead_code)]
pub struct PromptBuilder;

impl PromptBuilder {
    #[allow(dead_code)]
    pub fn build(
        pack_config: &PackConfig,
        mood: &str,
        user: &User,
        history: &ConversationManager,
        // sdk_defs: &str, // TODO: Pass SDK definitions
    ) -> Vec<ChatMessage> {
        let mut messages = Vec::new();
        let mut system_content = String::new();

        // 1. System Prompt
        system_content.push_str("# System Prompt\n");
        // TODO: Load system prompt from pack config if available, or use default
        system_content.push_str(
            "You are an AI assistant designed to help test the functionality of goon.ai.\n",
        );
        system_content.push_str(
            "You can display images, play videos, and play audio using the provided SDK.\n\n",
        );

        // 2. Mood
        system_content.push_str("# Current Mood\n");
        system_content.push_str(&format!("The user's current mood is: **{}**\n", mood));
        // Find mood description
        if let Some(m) = pack_config.moods.iter().find(|m| m.name == mood) {
            system_content.push_str(&format!("{}\n\n", m.description));
        } else {
            system_content.push('\n');
        }

        // 3. SDK Definitions
        system_content.push_str("# Available SDK Functions\n");
        system_content.push_str("```typescript\n");
        // TODO: Insert actual SDK definitions here
        system_content.push_str("// SDK definitions will go here\n");
        system_content.push_str("```\n\n");

        // 4. User Profile
        system_content.push_str("# User Profile\n");
        system_content.push_str(&format!("Name: {}\n", user.name));
        system_content.push_str(&format!("Gender: {}\n\n", user.gender));

        // 5. Task
        system_content.push_str("# Your Task\n");
        system_content.push_str(
            "Generate TypeScript code using the SDK functions above to interact with the user.\n",
        );

        messages.push(ChatMessage::new(MessageRole::System, system_content));

        // 6. History
        for msg in history.get_history() {
            let role = match msg.role.as_str() {
                "user" => MessageRole::User,
                "assistant" => MessageRole::Assistant,
                "system" => MessageRole::System,
                _ => MessageRole::User,
            };
            messages.push(ChatMessage::new(role, msg.content.clone()));
        }

        messages
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::pack::{Assets, Mood, PackMeta};
    use crate::config::settings::User;

    fn create_dummy_pack_config() -> PackConfig {
        PackConfig {
            meta: PackMeta {
                name: "Test Pack".to_string(),
                version: "1.0.0".to_string(),
                permissions: vec![],
            },
            moods: vec![Mood {
                name: "Happy".to_string(),
                description: "A happy mood description.".to_string(),
                tags: vec!["happy".to_string()],
            }],
            assets: Assets {
                image: None,
                video: None,
                audio: None,
                hypno: None,
                wallpaper: None,
            },
        }
    }

    fn create_dummy_user() -> User {
        User {
            name: "Test User".to_string(),
            dob: "1990-01-01".to_string(),
            gender: "Non-binary".to_string(),
        }
    }

    #[test]
    fn test_prompt_builder() {
        let pack_config = create_dummy_pack_config();
        let user = create_dummy_user();
        let mut history = ConversationManager::new(10);
        history.add_message("user", "Hello");
        history.add_message("assistant", "Hi there");

        let messages = PromptBuilder::build(&pack_config, "Happy", &user, &history);

        assert_eq!(messages.len(), 3); // System + User + Assistant

        let system_msg = &messages[0];
        assert_eq!(system_msg.role, MessageRole::System);
        assert!(system_msg.content.contains("# System Prompt"));
        assert!(
            system_msg
                .content
                .contains("The user's current mood is: **Happy**")
        );
        assert!(system_msg.content.contains("A happy mood description."));
        assert!(system_msg.content.contains("Name: Test User"));
        assert!(system_msg.content.contains("# Your Task"));

        let user_msg = &messages[1];
        assert_eq!(user_msg.role, MessageRole::User);
        assert_eq!(user_msg.content, "Hello");

        let assistant_msg = &messages[2];
        assert_eq!(assistant_msg.role, MessageRole::Assistant);
        assert_eq!(assistant_msg.content, "Hi there");
    }
}
