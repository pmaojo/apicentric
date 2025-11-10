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
                let _ = manager.start().await;
            });
        }
        if ui.button("Stop Simulator").clicked() {
            let manager = Arc::clone(manager);
            Handle::current().spawn(async move {
                let _ = manager.stop().await;
            });
        }
        ui.separator();
        ui.heading("AI Generation");
        ui.text_edit_multiline(&mut state.ai_prompt);
        if ui.button("Generate").clicked() {
            // TODO: Implement AI code generation
            // For now, just show a placeholder message
            state.logs.push("[AI] Code generation requested: ".to_string() + &state.ai_prompt);
        }
    });

    CentralPanel::default().show(ctx, |ui| {
        ui.heading("Logs");
        ScrollArea::vertical().show(ui, |ui| {
            for log in &state.logs {
                ui.label(log);
            }
        });
    });
}
