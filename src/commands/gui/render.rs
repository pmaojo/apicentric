//! GUI Rendering
//!
//! This module contains the rendering logic for the `egui` application.

#![cfg(feature = "gui")]

use super::state::GuiAppState;
use super::models::{ServiceStatus, LogFilter, EditorState, RequestLogEntry};
use egui::{CentralPanel, TopBottomPanel, SidePanel, ScrollArea};
use apicentric::simulator::manager::ApiSimulatorManager;
use std::sync::Arc;
use rand::Rng;


pub fn render(
    ctx: &egui::Context,
    state: &mut GuiAppState,
    manager: &Arc<ApiSimulatorManager>,
) {
    TopBottomPanel::top("top_panel").show(ctx, |ui| {
        ui.heading("Apicentric Control Panel");
    });

    SidePanel::left("service_panel").show(ctx, |ui| {
        ui.heading("Services");
        ScrollArea::vertical().show(ui, |ui| {
            let mut selected_service = state.selected_service.clone();
            let mut services_to_start = Vec::new();
            let mut services_to_stop = Vec::new();
            let mut service_to_edit = None;

            for service in &state.services {
                ui.horizontal(|ui| {
                    // Status indicator
                    let (color, icon) = match &service.status {
                        ServiceStatus::Running => (egui::Color32::GREEN, "🟢"),
                        ServiceStatus::Stopped => (egui::Color32::GRAY, "⚪"),
                        ServiceStatus::Failed(_) => (egui::Color32::RED, "🔴"),
                        ServiceStatus::Starting => (egui::Color32::YELLOW, "🟡"),
                        ServiceStatus::Stopping => (egui::Color32::YELLOW, "🟡"),
                    };
                    ui.colored_label(color, icon);

                    // Service name with click to edit
                    let response = ui.selectable_label(
                        selected_service.as_ref() == Some(&service.name),
                        &service.name
                    );

                    if response.clicked() {
                        selected_service = Some(service.name.clone());
                    }

                    // Double-click to open editor
                    if response.double_clicked() {
                        service_to_edit = Some(service.name.clone());
                    }

                    // Start/Stop buttons
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if service.can_start() && ui.small_button("▶").on_hover_text("Start").clicked() {
                            services_to_start.push(service.name.clone());
                        }
                        if service.can_stop() && ui.small_button("⏹").on_hover_text("Stop").clicked() {
                            services_to_stop.push(service.name.clone());
                        }
                    });
                });

                // Show endpoints if service is selected
                if selected_service.as_ref() == Some(&service.name) {
                    ui.indent("endpoints", |ui| {
                        for endpoint in &service.endpoints {
                            ui.horizontal(|ui| {
                                ui.label(&endpoint.method);
                                ui.label(&endpoint.path);
                            });
                        }
                    });
                }
            }

            // Apply changes after the loop to avoid borrowing issues
            state.selected_service = selected_service;
            for service_name in services_to_start {
                if let Some(service) = state.services.iter_mut().find(|s| s.name == service_name) {
                    let _ = service.start();
                    state.add_log(format!("Starting service: {}", service_name));
                }
            }
            for service_name in services_to_stop {
                if let Some(service) = state.services.iter_mut().find(|s| s.name == service_name) {
                    let _ = service.stop();
                    state.add_log(format!("Stopping service: {}", service_name));
                }
            }
            if let Some(service_name) = service_to_edit {
                state.load_service_in_editor(service_name, "# Service content would be loaded here".to_string());
            }
        });
    });

    SidePanel::right("actions_panel").show(ctx, |ui| {
        ui.heading("Actions");

        // Simulator controls
        ui.collapsing("Simulator", |ui| {
            if ui.button("Start Simulator").clicked() {
                // TODO: Actually start the simulator
                state.add_log("Starting simulator...".to_string());
                // For now, just add some dummy services to show the interface
                if state.services.is_empty() {
                    state.services.push(super::models::ServiceInfo::new("api-service".to_string(), std::path::PathBuf::from("services/api.yaml"), 8080));
                    state.services.push(super::models::ServiceInfo::new("user-service".to_string(), std::path::PathBuf::from("services/user.yaml"), 8081));
                    state.add_log("Added sample services".to_string());

                    // Add some sample request logs
                    state.add_request_log(RequestLogEntry::new("api-service".to_string(), "GET".to_string(), "/api/users".to_string(), 200, 45));
                    state.add_request_log(RequestLogEntry::new("api-service".to_string(), "POST".to_string(), "/api/users".to_string(), 201, 120));
                    state.add_request_log(RequestLogEntry::new("user-service".to_string(), "GET".to_string(), "/users/123".to_string(), 200, 30));
                    state.add_request_log(RequestLogEntry::new("api-service".to_string(), "GET".to_string(), "/api/orders".to_string(), 404, 25));
                    state.add_log("Added sample request logs".to_string());
                }
            }
            if ui.button("Stop Simulator").clicked() {
                state.add_log("Stopping simulator...".to_string());
            }
        });

        // Service management
        ui.collapsing("Services", |ui| {
            ui.horizontal(|ui| {
                if state.refreshing_services {
                    ui.add_enabled(false, egui::Button::new("Refreshing..."));
                    ui.add(egui::Spinner::new());
                } else if ui.button("Refresh Services").clicked() {
                    state.add_log("Refreshing services...".to_string());
                }
            });
            ui.label(format!("Loaded: {} services", state.services.len()));
        });

        ui.separator();

        // AI Generation
        ui.collapsing("AI Generation", |ui| {
            ui.text_edit_multiline(&mut state.ai_prompt);
            if ui.button("Generate").clicked() {
                state.add_log(format!("AI Generation requested: {}", state.ai_prompt));
                let yaml_content = format!(
                    "# Generated API Service\n\
                     name: {}\n\
                     port: 8080\n\
                     endpoints:\n\
                       - method: GET\n\
                         path: /api/{}\n\
                       - method: POST\n\
                         path: /api/{}\n",
                    state.ai_prompt.replace(" ", "-").to_lowercase(),
                    state.ai_prompt.to_lowercase(),
                    state.ai_prompt.to_lowercase()
                );
                state.ai_generated_yaml = Some(yaml_content);
                state.add_log("AI generation completed".to_string());
            }
        });

        // Editor controls
        if state.show_editor_window {
            ui.separator();
            ui.collapsing("Editor", |ui| {
                if ui.button("Save").clicked() {
                    state.add_log("Saving editor content...".to_string());
                    state.mark_editor_clean();
                }
                ui.label(if state.editor_state.dirty { "Unsaved changes" } else { "Saved" });
            });
        }
    });

    CentralPanel::default().show(ctx, |ui| {
        ui.heading("Logs");

        // Show log count and filter info
        ui.horizontal(|ui| {
            ui.label(format!("Total logs: {}", state.logs.len()));
            if state.request_log_count() > 0 {
                ui.label(format!("Filtered: {}", state.filtered_request_logs().len()));
            }
        });

        // Log filter controls
        ui.horizontal(|ui| {
            ui.label("Filter:");
            if ui.button("All").clicked() {
                state.set_log_filter(LogFilter::All);
            }
            if ui.button("GET").clicked() {
                state.set_log_filter(LogFilter::Method("GET".to_string()));
            }
            if ui.button("POST").clicked() {
                state.set_log_filter(LogFilter::Method("POST".to_string()));
            }
            if ui.button("200").clicked() {
                state.set_log_filter(LogFilter::StatusCode(200));
            }
            if ui.button("404").clicked() {
                state.set_log_filter(LogFilter::StatusCode(404));
            }
            if ui.button("Clear").clicked() {
                state.clear_logs();
            }
            if ui.button("Add Sample Log").clicked() {
                let methods = ["GET", "POST", "PUT", "DELETE"];
                let paths = ["/api/users", "/api/orders", "/api/products", "/api/auth"];
                let statuses = [200, 201, 400, 404, 500];

                let mut rng = rand::thread_rng();
                let method = methods[rng.gen_range(0..methods.len())];
                let path = paths[rng.gen_range(0..paths.len())];
                let status = statuses[rng.gen_range(0..statuses.len())];
                let duration = rng.gen_range(10..200);

                state.add_request_log(RequestLogEntry::new(
                    "api-service".to_string(),
                    method.to_string(),
                    path.to_string(),
                    status,
                    duration
                ));
            }
        });

        // Virtualized log display
        ScrollArea::vertical().show(ui, |ui| {
            let filtered_logs = state.filtered_request_logs();

            // Show only last 100 logs for performance, with option to show more
            let display_count = if filtered_logs.len() > 100 {
                ui.label(format!("Showing last 100 of {} logs", filtered_logs.len()));
                100
            } else {
                filtered_logs.len()
            };

            for log in filtered_logs.iter().rev().take(display_count).rev() {
                ui.horizontal(|ui| {
                    use chrono::{DateTime, Utc};
                    let datetime: DateTime<Utc> = log.timestamp.into();
                    ui.label(format!("[{}]", datetime.format("%H:%M:%S")));
                    ui.colored_label(
                        match log.status_code {
                            200..=299 => egui::Color32::GREEN,
                            300..=399 => egui::Color32::BLUE,
                            400..=499 => egui::Color32::YELLOW,
                            500..=599 => egui::Color32::RED,
                            _ => egui::Color32::GRAY,
                        },
                        format!("{}", log.status_code)
                    );
                    ui.label(&log.method);
                    ui.label(&log.path);
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(format!("{}ms", log.duration_ms));
                    });
                });
            }
        });
    });

    // Editor window
    if state.show_editor_window {
        let mut show_editor = true;
        let window_title = if state.editor_state.loading {
            format!("Editor - {} (Loading...)", state.editor_state.selected_service.as_deref().unwrap_or("Unknown"))
        } else {
            format!("Editor - {}", state.editor_state.selected_service.as_deref().unwrap_or("Unknown"))
        };

        let mut save_clicked = false;
        let mut close_clicked = false;

        egui::Window::new(window_title)
            .open(&mut show_editor)
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        // Save button with loading state
                        if state.editor_state.saving {
                            ui.add_enabled(false, egui::Button::new("Saving..."));
                        } else if ui.button("Save").clicked() {
                            save_clicked = true;
                        }

                        // Status indicator
                        if state.editor_state.loading {
                            ui.label("⏳ Loading...");
                        } else if state.editor_state.saving {
                            ui.label("💾 Saving...");
                        } else if state.editor_state.dirty {
                            ui.label("⚠ Unsaved changes");
                        } else {
                            ui.label("✓ Saved");
                        }

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button("Close").clicked() {
                                close_clicked = true;
                            }
                        });
                    });

                    ScrollArea::vertical().show(ui, |ui| {
                        if state.editor_state.loading {
                            ui.vertical_centered(|ui| {
                                ui.add(egui::Spinner::new());
                                ui.label("Loading service content...");
                            });
                        } else {
                            ui.add(
                                egui::TextEdit::multiline(&mut state.editor_state.content)
                                    .font(egui::TextStyle::Monospace)
                                    .code_editor()
                            );
                        }
                    });
                });
            });

        // Handle actions after window rendering
        if save_clicked {
            state.add_log("Saving editor content...".to_string());
            state.mark_editor_clean();
        }
        if close_clicked || !show_editor {
            state.show_editor_window = false;
            state.editor_state = EditorState::default();
        }
    }

    if let Some(yaml) = &state.ai_generated_yaml {
        let mut apply_clicked = false;
        egui::Window::new("Generated YAML").show(ctx, |ui| {
            ScrollArea::vertical().show(ui, |ui| {
                ui.text_edit_multiline(&mut yaml.clone());
            });
            if ui.button("Apply YAML").clicked() {
                apply_clicked = true;
            }
        });

        if apply_clicked {
            state.add_log("Applying generated YAML...".to_string());
            state.ai_generated_yaml = None; // Close the window
        }
    }
}
