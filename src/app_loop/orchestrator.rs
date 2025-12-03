use crate::app_loop::state::{LoopState, MessageType};
use crate::assets::loader::AssetLoader;
use crate::config::pack::PackConfig;
use crate::config::settings::Settings;
use crate::gui::WindowSpawnerHandle;
use crate::llm::client::LLMClient;
use crate::llm::conversation::ConversationManager;
use crate::llm::prompt::PromptBuilder;
use crate::permissions::PermissionChecker;
use crate::runtime::runtime::{GoonRuntime, RuntimeContext};
use crate::typescript::compiler::TypeScriptCompiler;
use anyhow::Result;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

pub struct Orchestrator {
    state: LoopState,
    settings: Arc<Settings>,
    pack_config: Arc<PackConfig>,
    permissions: Arc<PermissionChecker>,
    window_spawner: WindowSpawnerHandle,
}

impl Orchestrator {
    pub fn new(
        settings: Arc<Settings>,
        pack_config: Arc<PackConfig>,
        permissions: Arc<PermissionChecker>,
        window_spawner: WindowSpawnerHandle,
    ) -> Self {
        Self {
            state: LoopState::new(),
            settings,
            pack_config,
            permissions,
            window_spawner,
        }
    }

    pub async fn run(&mut self) -> Result<()> {
        println!("Starting main loop...");

        // 1. Initialize Systems
        let registry = Arc::new(AssetLoader::load(
            &self.pack_config,
            &self.settings.runtime.pack.current,
        )?);

        let llm_client = LLMClient::new(
            &self.settings.llm_settings,
            &self.settings.llm_settings.model,
        );

        let mut history = ConversationManager::new(50); // TODO: Configurable history size
        let compiler = TypeScriptCompiler::new();

        // Generate SDK definitions (asset-free)
        let sdk_defs = crate::sdk::generate_definitions_for_permissions(&self.permissions);

        // Initialize Runtime
        let mood_name = &self.settings.runtime.pack.mood;
        let mood = self
            .pack_config
            .moods
            .iter()
            .find(|m| &m.name == mood_name)
            .cloned()
            .unwrap_or_else(|| crate::config::pack::Mood {
                name: mood_name.clone(),
                description: "Default mood".to_string(),
                tags: vec![],
                prompt: None,
            });

        let context = RuntimeContext {
            permissions: (*self.permissions).clone(),
            window_spawner: self.window_spawner.clone(),
            registry: registry.clone(),
            mood: mood.clone(),
            max_audio_concurrent: self.settings.runtime.popups.audio.max.unwrap_or(1) as usize,
        };

        let mut runtime = GoonRuntime::new(context);

        loop {
            self.state.iteration_count += 1;
            println!("Iteration: {}", self.state.iteration_count);

            // 1. Build Context (Asset-free)
            // Check retry limit
            if self.state.retry_count >= 3 {
                println!("Max retries reached. resetting retry count.");
                history.add_message("system", "Too many consecutive errors. Please stop retrying the failing code and try a different approach or wait for user input.");
                self.state.reset_retry();
            }

            let execution_failed = self.state.retry_count > 0;
            let messages = PromptBuilder::build(
                &self.pack_config,
                &mood.name,
                &self.settings.user,
                &history,
                &sdk_defs,
                execution_failed,
            );

            // 2. Call LLM
            println!("Calling LLM...");
            match llm_client.chat(messages).await {
                Ok(response) => {
                    println!("LLM Response: {}", response);
                    history.add_message("assistant", &response);
                    self.state
                        .add_message(MessageType::Assistant, response.clone());

                    // 3. Extract and Compile TS
                    // Simple extraction: look for ```typescript ... ``` or just assume code block
                    // For now, let's assume the LLM returns a code block or we parse it.
                    // The PromptBuilder asks for TypeScript code.

                    let code_block = extract_code_block(&response);
                    if let Some(code) = code_block {
                        println!("Compiling code...");
                        match compiler.compile(&code) {
                            Ok(js_code) => {
                                println!("Executing JS...");
                                match runtime.execute_script(&js_code).await {
                                    Ok(_) => {
                                        println!("Execution successful");
                                        self.state.reset_retry();
                                    }
                                    Err(e) => {
                                        eprintln!("Runtime error: {}", e);
                                        let error_msg = format!("Runtime Error: {}", e);
                                        history.add_message("system", &error_msg);
                                        self.state.add_error(error_msg);
                                    }
                                }
                            }
                            Err(e) => {
                                eprintln!("Compilation error: {}", e);
                                let error_msg = format!("Compilation Error: {}", e);
                                history.add_message("system", &error_msg);
                                self.state.add_error(error_msg);
                            }
                        }
                    } else {
                        println!("No code block found in response");
                    }
                }
                Err(e) => {
                    eprintln!("LLM Error: {}", e);
                    self.state.add_error(format!("LLM Error: {}", e));
                }
            }

            // Delay
            sleep(Duration::from_secs(5)).await;
        }
    }

    pub async fn run_script(&mut self, script: &str) -> Result<()> {
        println!("Running script in sandbox...");

        // Initialize systems - following the same pattern as run()
        let registry = Arc::new(AssetLoader::load(
            &self.pack_config,
            &self.settings.runtime.pack.current,
        )?);

        let compiler = TypeScriptCompiler::new();

        // Generate SDK definitions (asset-free)
        let _sdk_defs = crate::sdk::generate_definitions_for_permissions(&self.permissions);

        // Get mood
        let mood_name = &self.settings.runtime.pack.mood;
        let mood = self
            .pack_config
            .moods
            .iter()
            .find(|m| &m.name == mood_name)
            .cloned()
            .unwrap_or_else(|| crate::config::pack::Mood {
                name: mood_name.clone(),
                description: "Default mood".to_string(),
                tags: vec![],
                prompt: None,
            });

        let context = RuntimeContext {
            permissions: (*self.permissions).clone(),
            window_spawner: self.window_spawner.clone(),
            registry: registry.clone(),
            mood: mood.clone(),
            max_audio_concurrent: self.settings.runtime.popups.audio.max.unwrap_or(1) as usize,
        };

        let mut runtime = GoonRuntime::new(context);

        // Execute the provided script
        println!("Executing script...");
        match compiler.compile(script) {
            Ok(js_code) => {
                println!("Script compiled successfully");
                match runtime.execute_script(&js_code).await {
                    Ok(_) => {
                        println!("Script execution successful");
                    }
                    Err(e) => {
                        eprintln!("Runtime error: {}", e);
                        return Err(anyhow::anyhow!("Runtime Error: {}", e));
                    }
                }
            }
            Err(e) => {
                eprintln!("Compilation error: {}", e);
                return Err(anyhow::anyhow!("Compilation Error: {}", e));
            }
        }

        // Keep the event loop running to allow GUI elements to render
        println!("Script completed. Keeping GUI alive for rendering...");
        loop {
            sleep(Duration::from_millis(100)).await;
        }
    }
}

fn extract_code_block(response: &str) -> Option<String> {
    // Remove <think> blocks
    let mut clean_response = response.to_string();
    while let Some(start) = clean_response.find("<think>") {
        if let Some(end) = clean_response[start..].find("</think>") {
            clean_response.replace_range(start..start + end + 8, "");
        } else {
            break;
        }
    }

    // Extract code block
    let code = if let Some(start) = clean_response.find("```typescript") {
        let rest = &clean_response[start + 13..];
        rest.find("```").map(|end| rest[..end].trim().to_string())
    } else if let Some(start) = clean_response.find("```") {
        let rest = &clean_response[start + 3..];
        if let Some(end) = rest.find("```") {
            // Strip language identifier if present (e.g. "ts\n")
            if let Some(newline) = rest[..end].find('\n') {
                Some(rest[newline + 1..end].trim().to_string())
            } else {
                Some(rest[..end].trim().to_string())
            }
        } else {
            None
        }
    } else if !clean_response.trim().is_empty() {
        Some(clean_response.trim().to_string())
    } else {
        None
    };

    // Strip imports from the extracted code
    if let Some(code) = code {
        let lines: Vec<&str> = code
            .lines()
            .filter(|line| !line.trim().starts_with("import "))
            .collect();
        Some(lines.join("\n"))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_code_block() {
        let response = "Here is the code:\n```typescript\nconsole.log('hello');\n```";
        assert_eq!(
            extract_code_block(response),
            Some("console.log('hello');".to_string())
        );

        let response_with_think =
            "<think>Some thinking...</think>\n```typescript\nconsole.log('hello');\n```";
        assert_eq!(
            extract_code_block(response_with_think),
            Some("console.log('hello');".to_string())
        );

        let response_no_lang = "```\nconsole.log('hello');\n```";
        assert_eq!(
            extract_code_block(response_no_lang),
            Some("console.log('hello');".to_string())
        );

        let response_raw = "console.log('hello');";
        assert_eq!(
            extract_code_block(response_raw),
            Some("console.log('hello');".to_string())
        );

        let response_with_imports =
            "```typescript\nimport { image } from './sdk';\nconsole.log('hello');\n```";
        assert_eq!(
            extract_code_block(response_with_imports),
            Some("console.log('hello');".to_string())
        );
    }
}
