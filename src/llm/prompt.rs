use crate::config::pack::PackConfig;
use crate::config::settings::User;
use crate::llm::conversation::ConversationManager;

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
    ) -> String {
        let mut prompt = String::new();

        // 1. System Prompt
        prompt.push_str("# System Prompt\n");
        // TODO: Load system prompt from pack config if available, or use default
        prompt.push_str("You are an AI assistant designed to help test the functionality of goon.ai.\n");
        prompt.push_str("You can display images, play videos, and play audio using the provided SDK.\n\n");

        // 2. Mood
        prompt.push_str("# Current Mood\n");
        prompt.push_str(&format!("The user's current mood is: **{}**\n", mood));
        // Find mood description
        if let Some(m) = pack_config.moods.iter().find(|m| m.name == mood) {
             prompt.push_str(&format!("{}\n\n", m.description));
        } else {
             prompt.push_str("\n");
        }

        // 3. SDK Definitions
        prompt.push_str("# Available SDK Functions\n");
        prompt.push_str("```typescript\n");
        // TODO: Insert actual SDK definitions here
        prompt.push_str("// SDK definitions will go here\n");
        prompt.push_str("```\n\n");

        // 4. User Profile
        prompt.push_str("# User Profile\n");
        prompt.push_str(&format!("Name: {}\n", user.name));
        prompt.push_str(&format!("Gender: {}\n\n", user.gender));

        // 5. History
        prompt.push_str("# Recent History\n");
        for msg in history.get_history() {
            prompt.push_str(&format!("{}: {}\n", msg.role, msg.content));
        }
        prompt.push_str("\n");

        // 6. Task
        prompt.push_str("# Your Task\n");
        prompt.push_str("Generate TypeScript code using the SDK functions above to interact with the user.\n");

        prompt
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::pack::{PackMeta, Mood, Assets};
    use crate::config::settings::User;

    fn create_dummy_pack_config() -> PackConfig {
        PackConfig {
            meta: PackMeta {
                name: "Test Pack".to_string(),
                version: "1.0.0".to_string(),
                permissions: vec![],
            },
            moods: vec![
                Mood {
                    name: "Happy".to_string(),
                    description: "A happy mood description.".to_string(),
                    tags: vec!["happy".to_string()],
                }
            ],
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

        let prompt = PromptBuilder::build(
            &pack_config,
            "Happy",
            &user,
            &history,
        );

        assert!(prompt.contains("# System Prompt"));
        assert!(prompt.contains("The user's current mood is: **Happy**"));
        assert!(prompt.contains("A happy mood description."));
        assert!(prompt.contains("Name: Test User"));
        assert!(prompt.contains("user: Hello"));
        assert!(prompt.contains("assistant: Hi there"));
    }
}
