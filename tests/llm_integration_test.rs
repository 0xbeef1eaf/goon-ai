use goon_ai::config::pack::{Assets, Mood, PackConfig, PackMeta};
use goon_ai::config::settings::User;
use goon_ai::llm::conversation::ConversationManager;
use goon_ai::llm::prompt::PromptBuilder;
use goon_ai::permissions::Permission;
use ollama_rs::generation::chat::MessageRole;

#[test]
fn test_llm_prompt_construction_flow() {
    // 1. Setup Configs
    let pack_config = PackConfig {
        meta: PackMeta {
            name: "LLMPack".to_string(),
            version: "1.0.0".to_string(),
            permissions: vec![Permission::Prompt],
        },
        moods: vec![Mood {
            name: "Curious".to_string(),
            description: "The AI is inquisitive.".to_string(),
            tags: vec!["questioning".to_string()],
        }],
        assets: Assets {
            image: None,
            video: None,
            audio: None,
            hypno: None,
            wallpaper: None,
        },
        websites: None,
    };

    let user = User {
        name: "IntegrationUser".to_string(),
        dob: "2000-01-01".to_string(),
        gender: "Robot".to_string(),
    };

    // 2. Setup Conversation
    let mut history = ConversationManager::new(5);
    history.add_message("user", "Hello AI");
    history.add_message("assistant", "Hello User");
    history.add_message("user", "What is your mood?");

    // 3. Build Prompt
    let messages = PromptBuilder::build(&pack_config, "Curious", &user, &history, "");

    // 4. Verify Structure
    // Expect: System Prompt + 3 History Messages = 4 Total
    assert_eq!(messages.len(), 4);

    // Verify System Prompt Content
    let system_msg = &messages[0];
    assert_eq!(system_msg.role, MessageRole::System);

    // Check for Mood integration
    assert!(
        system_msg
            .content
            .contains("The user's current mood is: **Curious**")
    );
    assert!(system_msg.content.contains("The AI is inquisitive."));

    // Check for User integration
    assert!(system_msg.content.contains("Name: IntegrationUser"));
    assert!(system_msg.content.contains("Gender: Robot"));

    // Verify History
    assert_eq!(messages[1].role, MessageRole::User);
    assert_eq!(messages[1].content, "Hello AI");

    assert_eq!(messages[2].role, MessageRole::Assistant);
    assert_eq!(messages[2].content, "Hello User");

    assert_eq!(messages[3].role, MessageRole::User);
    assert_eq!(messages[3].content, "What is your mood?");
}
