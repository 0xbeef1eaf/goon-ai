use crate::app_loop::state::{LoopState, MessageType};
use crate::assets::loader::AssetLoader;
use crate::config::pack::PackConfig;
use crate::config::settings::Settings;
use crate::gui::window_manager::GuiInterface;
use crate::llm::client::LLMClient;
use crate::llm::conversation::ConversationManager;
use crate::llm::prompt::PromptBuilder;
use crate::permissions::PermissionChecker;
use crate::runtime::runtime::{GoonRuntime, RuntimeContext};
use crate::typescript::compiler::TypeScriptCompiler;
use crate::typescript::sdk_generator::SdkGenerator;
use anyhow::Result;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

pub struct Orchestrator {
    state: LoopState,
    settings: Arc<Settings>,
    pack_config: Arc<PackConfig>,
    permissions: Arc<PermissionChecker>,
    gui_controller: Arc<dyn GuiInterface>,
}

impl Orchestrator {
    pub fn new(
        settings: Arc<Settings>,
        pack_config: Arc<PackConfig>,
        permissions: Arc<PermissionChecker>,
        gui_controller: Arc<dyn GuiInterface>,
    ) -> Self {
        Self {
            state: LoopState::new(),
            settings,
            pack_config,
            permissions,
            gui_controller,
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
        // TODO: Filter modules based on permissions
        let allowed_modules = vec![
            "system".to_string(),
            "image".to_string(),
            "video".to_string(),
            "audio".to_string(),
            "hypno".to_string(),
            "wallpaper".to_string(),
            "prompt".to_string(),
            "website".to_string(),
        ];
        let sdk_defs = SdkGenerator::generate_definitions(&allowed_modules);

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
            });

        let context = RuntimeContext {
            permissions: (*self.permissions).clone(),
            gui_controller: self.gui_controller.clone(),
            registry: registry.clone(),
            mood: mood.clone(),
            max_audio_concurrent: self.settings.runtime.popups.audio.max.unwrap_or(1) as usize,
            max_video_concurrent: self.settings.runtime.popups.video.max.unwrap_or(1) as usize,
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

            let messages = PromptBuilder::build(
                &self.pack_config,
                &mood.name,
                &self.settings.user,
                &history,
                &sdk_defs,
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
}

fn extract_code_block(response: &str) -> Option<String> {
    // Simple extractor for ```typescript ... ``` or ``` ... ```
    if let Some(start) = response.find("```") {
        let rest = &response[start + 3..];
        let end = rest.find("```")?;
        let content = &rest[..end];
        // Strip language identifier if present
        if let Some(newline) = content.find('\n') {
            Some(content[newline + 1..].to_string())
        } else {
            Some(content.to_string())
        }
    } else {
        None
    }
}
