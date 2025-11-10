//! GUI Rendering
//!
//! This module contains the rendering logic for the `egui` application.

#![cfg(feature = "gui")]

use super::state::GuiAppState;
use egui::{CentralPanel, TopBottomPanel, SidePanel, ScrollArea};
use apicentric::simulator::manager::ApiSimulatorManager;
use std::sync::Arc;
use tokio::runtime::Handle;

pub fn render(ctx: &egui::Context, state: &mut GuiAppState, manager: &Arc<ApiSimulatorManager>) {
    TopBottomPanel::top("top_panel").show(ctx, |ui| {
        ui.heading("Apicentric Control Panel");
    });

    SidePanel::left("service_panel").show(ctx, |ui| {
        ui.heading("Services");
        ScrollArea::vertical().show(ui, |ui| {
            for service in &state.services {
                ui.label(service);
            }
        });
    });

    SidePanel::right("actions_panel").show(ctx, |ui| {
        ui.heading("Actions");
        if ui.button("Start Simulator").clicked() {
            let manager = Arc::clone(manager);
            Handle::current().spawn(async move {
                match manager.start().await {
                    Ok(_) => println!("Simulator started successfully"),
                    Err(e) => eprintln!("Failed to start simulator: {}", e),
                }
            });
        }
        if ui.button("Stop Simulator").clicked() {
            let manager = Arc::clone(manager);
            Handle::current().spawn(async move {
                match manager.stop().await {
                    Ok(_) => println!("Simulator stopped successfully"),
                    Err(e) => eprintln!("Failed to stop simulator: {}", e),
                }
            });
        }
        ui.separator();
        ui.heading("AI Generation");
        ui.text_edit_multiline(&mut state.ai_prompt);
        if ui.button("Generate").clicked() {
            let prompt = state.ai_prompt.clone();
            let manager = Arc::clone(manager);
            Handle::current().spawn(async move {
                println!("Starting AI generation for prompt: {}", prompt);

                // Use the AI command logic here
                use apicentric::ai::{AiProvider, GeminiAiProvider};
                use apicentric::config::{AiProviderKind, ApicentricConfig};

                // Load config
                let config = ApicentricConfig::builder().build().unwrap_or_else(|_| ApicentricConfig::builder().build().unwrap());
                println!("Config loaded, AI config present: {}", config.ai.is_some());

                if let Some(ai_cfg) = &config.ai {
                    println!("AI provider: {:?}", ai_cfg.provider);
                    let provider: Box<dyn AiProvider> = match ai_cfg.provider {
                        AiProviderKind::Gemini => {
                            let key = std::env::var("GEMINI_API_KEY").ok().or_else(|| ai_cfg.api_key.clone());
                            println!("API key present: {}", key.is_some());
                            if let Some(k) = key {
                                let model = ai_cfg.model.clone().unwrap_or_else(|| "gemini-2.0-flash-exp".to_string());
                                println!("Creating Gemini provider with model: {}", model);
                                Box::new(GeminiAiProvider::new(k, model))
                            } else {
                                eprintln!("No Gemini API key found");
                                return;
                            }
                        }
                        _ => {
                            eprintln!("Unsupported AI provider");
                            return;
                        }
                    };

                    println!("Calling generate_yaml...");
                    match provider.generate_yaml(&prompt).await {
                        Ok(yaml) => {
                            println!("YAML generated successfully, length: {}", yaml.len());
                            println!("Generated YAML:\n{}", yaml);
                            match manager.apply_service_yaml(&yaml).await {
                                Ok(_) => {
                                    println!("AI generation successful: Service applied");
                                }
                                Err(e) => {
                                    eprintln!("Failed to apply service YAML: {}", e);
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("AI generation failed: {}", e);
                        }
                    }
                } else {
                    eprintln!("No AI configuration found");
                }
            });
            // Add a log entry to show the button was clicked
            state.logs.push(format!("[{}] Generate button clicked with prompt: {}", chrono::Utc::now().format("%H:%M:%S"), state.ai_prompt));
        }
    });

    CentralPanel::default().show(ctx, |ui| {
        ui.heading("Logs");
        ScrollArea::vertical().show(ui, |ui| {
            for log in &state.logs {
                ui.label(log);
            }
        });
        // Force a repaint to update the UI
        ctx.request_repaint();
    });
}
