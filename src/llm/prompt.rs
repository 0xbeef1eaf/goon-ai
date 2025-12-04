use crate::config::pack::PackConfig;
use crate::config::settings::User;
use crate::gui::windows::types::WindowInfo;
use crate::llm::conversation::ConversationManager;
use chrono::Datelike;
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
        sdk_defs: &str,
        active_windows: &[WindowInfo],
        execution_failed: bool,
    ) -> Vec<ChatMessage> {
        let mut messages = Vec::new();
        let mut system_content = String::new();

        // 1. System Prompt
        system_content.push_str("# System Prompt\n");

        let default_system = "You are an AI assistant designed to help test the functionality of goon.ai.\n\
            You can display images, play videos, and play audio using the provided SDK.\n\
            The SDK classes (image, video, audio, etc.) are available globally. DO NOT import them.\n\
            DO NOT use 'import' statements. The code is executed in a global context where SDK is pre-loaded.\n\n";

        // Priority: 1. Mood prompt, 2. Pack prompt, 3. Default system prompt
        let mut system_used = false;

        // Check for mood-specific prompt
        if let Some(m) = pack_config.moods.iter().find(|m| m.name == mood)
            && let Some(prompt) = &m.prompt
        {
            system_content.push_str(prompt);
            system_content.push_str("\n\n");
            system_used = true;
        }

        // If no mood prompt, check for pack-level prompt
        if !system_used
            && let Some(prompts) = &pack_config.prompts
            && let Some(sys) = &prompts.system
        {
            system_content.push_str(sys);
            system_content.push_str("\n\n");
            system_used = true;
        }

        // If no mood or pack prompt, use default
        if !system_used {
            system_content.push_str(default_system);
        }

        // 2. Mood
        system_content.push_str("# Moods\n");
        system_content.push_str("Moods are used to change the available media. You can change moods if you want to change up the current session.\n\n");

        system_content.push_str("## Current Mood\n");
        if let Some(m) = pack_config.moods.iter().find(|m| m.name == mood) {
            system_content.push_str(&format!("**{}**: {}\n\n", m.name, m.description));
        } else {
            system_content.push_str(&format!("**{}** (unknown mood)\n\n", mood));
        }

        system_content.push_str("## Other Moods Available\n");
        for m in &pack_config.moods {
            if m.name != mood {
                system_content.push_str(&format!("- **{}**: {}\n", m.name, m.description));
            }
        }
        system_content.push('\n');

        // 3. SDK Definitions
        system_content.push_str("# Available SDK Functions\n");
        system_content.push_str("```typescript\n");
        system_content.push_str(sdk_defs);
        system_content.push_str("\n```\n\n");

        // 4. Active Windows
        if !active_windows.is_empty() {
            system_content.push_str("# Active Windows\n");
            system_content.push_str("The following windows are currently open. You can close them using their handle ID.\n");
            for window in active_windows {
                system_content.push_str(&format!(
                    "- Type: {}, Handle: {}, Description: {}\n",
                    window.window_type, window.handle.0, window.description
                ));
            }
            system_content.push('\n');
        }

        // 5. User Profile
        system_content.push_str("# User Profile\n");
        system_content.push_str(&format!("Name: {}\n", user.name));
        system_content.push_str(&format!("Gender: {}\n\n", user.gender));

        // Add in age if DOB is valid
        if let Ok(dob) = chrono::NaiveDate::parse_from_str(&user.dob, "%Y-%m-%d") {
            let today = chrono::Utc::now().naive_utc().date();
            let age = today.year()
                - dob.year()
                - if today.ordinal() < dob.ordinal() {
                    1
                } else {
                    0
                };
            system_content.push_str(&format!("Age: {}\n\n", age));
        }

        // 5. Task
        system_content.push_str("# Your Task\n");
        system_content.push_str(
            "Generate TypeScript code using the SDK functions above to interact with the user, you must execute the code\n",
        );
        system_content
            .push_str("Output ONLY a single TypeScript code wrapped in a ```typescript``` block, previous defintions will not be evaluated.\n");
        system_content.push_str("Do not include any other text, explanations.\n");

        messages.push(ChatMessage::new(MessageRole::System, system_content));

        // 6. History - Only include if execution failed
        if execution_failed {
            for msg in history.get_history() {
                let role = match msg.role.as_str() {
                    "user" => MessageRole::User,
                    "assistant" => MessageRole::Assistant,
                    "system" => MessageRole::System,
                    _ => MessageRole::User,
                };
                messages.push(ChatMessage::new(role, msg.content.clone()));
            }
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
                prompt: None,
            }],
            assets: Assets {
                image: None,
                video: None,
                audio: None,
                hypno: None,
                wallpaper: None,
            },
            websites: None,
            prompts: None,
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

        let messages = PromptBuilder::build(
            &pack_config,
            "Happy",
            &user,
            &history,
            "class image {}",
            true,
        );

        assert_eq!(messages.len(), 3); // System + User + Assistant

        let system_msg = &messages[0];
        assert_eq!(system_msg.role, MessageRole::System);
        assert!(system_msg.content.contains("# System Prompt"));
        assert!(
            system_msg
                .content
                .contains("**Happy**: A happy mood description.")
        );
        assert!(system_msg.content.contains("Name: Test User"));
        assert!(system_msg.content.contains("# Your Task"));

        let user_msg = &messages[1];
        assert_eq!(user_msg.role, MessageRole::User);
        assert_eq!(user_msg.content, "Hello");

        let assistant_msg = &messages[2];
        assert_eq!(assistant_msg.role, MessageRole::Assistant);
        assert_eq!(assistant_msg.content, "Hi there");
    }

    #[test]
    fn test_prompt_builder_no_history() {
        let pack_config = create_dummy_pack_config();
        let user = create_dummy_user();
        let mut history = ConversationManager::new(10);
        history.add_message("user", "Hello");
        history.add_message("assistant", "Hi there");

        let messages = PromptBuilder::build(
            &pack_config,
            "Happy",
            &user,
            &history,
            "class image {}",
            false,
        );

        assert_eq!(messages.len(), 1); // System only
        assert_eq!(messages[0].role, MessageRole::System);
    }
}
