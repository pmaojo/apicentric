//! GUI Rendering
//!
//! This module contains the rendering logic for the `egui` application.

#![cfg(feature = "gui")]

use super::state::GuiAppState;
use super::GuiMessage;
use egui::{CentralPanel, TopBottomPanel, SidePanel, ScrollArea};
use apicentric::simulator::manager::ApiSimulatorManager;
use std::sync::Arc;
use tokio::runtime::Handle;
use tokio::sync::mpsc;

pub fn render(
    ctx: &egui::Context,
    state: &mut GuiAppState,
    manager: &Arc<ApiSimulatorManager>,
    sender: &mpsc::Sender<GuiMessage>,
) {
    TopBottomPanel::top("top_panel").show(ctx, |ui| {
        ui.heading("Apicentric Control Panel");
    });

    SidePanel::left("service_panel").show(ctx, |ui| {
        ui.heading("Services");
        ScrollArea::vertical().show(ui, |ui| {
            for service in &state.services {
                ui.horizontal(|ui| {
                    // Status indicator
                    let (color, icon) = match &service.status {
                        super::state::ServiceStatus::Running => (egui::Color32::GREEN, "🟢"),
                        super::state::ServiceStatus::Stopped => (egui::Color32::GRAY, "⚪"),
                        super::state::ServiceStatus::Failed(_) => (egui::Color32::RED, "🔴"),
                        super::state::ServiceStatus::Starting => (egui::Color32::YELLOW, "🟡"),
                        super::state::ServiceStatus::Stopping => (egui::Color32::YELLOW, "🟡"),
                    };
                    ui.colored_label(color, icon);
                    ui.label(&service.name);
                });
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
            let sender = sender.clone();
            tokio::spawn(async move {
                sender.send(GuiMessage::AiGenerate(prompt)).await.ok();
            });
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

    if let Some(yaml) = &state.ai_generated_yaml {
        egui::Window::new("Generated YAML").show(ctx, |ui| {
            ScrollArea::vertical().show(ui, |ui| {
                ui.text_edit_multiline(&mut yaml.clone());
            });
            if ui.button("Apply YAML").clicked() {
                let sender = sender.clone();
                let yaml = yaml.clone();
                tokio::spawn(async move {
                    sender.send(GuiMessage::AiApplyYaml(yaml)).await.ok();
                });
            }
        });
    }
}
