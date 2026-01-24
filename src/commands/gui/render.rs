//! GUI Rendering
//!
//! This module contains the rendering logic for the `egui` application.

use super::messages::GuiSystemEvent;
use super::models::{EditorState, LogFilter, RequestLogEntry, ServiceStatus};
use super::state::GuiAppState;
use apicentric::simulator::config::ConfigLoader;
use apicentric::simulator::manager::ApiSimulatorManager;
use egui::{CentralPanel, ScrollArea, SidePanel, TopBottomPanel};
use rand::Rng;
use std::sync::Arc;

pub fn render(ctx: &egui::Context, state: &mut GuiAppState, _manager: &Arc<ApiSimulatorManager>) {
    // Poll system events
    while let Ok(event) = state.system_event_rx.try_recv() {
        match event {
            GuiSystemEvent::SimulatorStarted => {
                state.is_simulator_running = true;
                state.add_log("Simulator started successfully.".to_string());
            }
            GuiSystemEvent::SimulatorStopped => {
                state.is_simulator_running = false;
                state.add_log("Simulator stopped.".to_string());
            }
            GuiSystemEvent::ServicesLoaded(services) => {
                state.services = services;
                state.add_log(format!("Loaded {} services.", state.services.len()));
            }
            GuiSystemEvent::ServicesRefreshed(services) => {
                state.services = services;
                state.refreshing_services = false;
                state.add_log(format!(
                    "Refreshed services: {} found.",
                    state.services.len()
                ));
            }
            GuiSystemEvent::Error(err) => {
                state.add_log(format!("Error: {}", err));
                state.refreshing_services = false;
            }
            GuiSystemEvent::Log(msg) => {
                state.add_log(msg);
            }
        }
    }

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
                        &service.name,
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
                        if service.can_start()
                            && ui.small_button("▶").on_hover_text("Start").clicked()
                        {
                            services_to_start.push(service.name.clone());
                        }
                        if service.can_stop()
                            && ui.small_button("⏹").on_hover_text("Stop").clicked()
                        {
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
                // Trigger start via manager logic
                // Since we don't have individual service start in this loop yet connected to manager async task,
                // we will rely on existing logic or implement similar spawn pattern if needed.
                // For now, keeping existing logic but maybe should use spawn?
                // The existing logic only updates state optimistically.
                // Ideally we should use the same pattern as Start Simulator.
                // But let's focus on the "Start Simulator" button as requested.
                if let Some(service) = state.services.iter_mut().find(|s| s.name == service_name) {
                    let _ = service.start();
                    state.add_log(format!("Starting service: {}", service_name));

                    let manager = _manager.clone();
                    let tx = state.system_event_tx.clone();
                    let name = service_name.clone();
                    tokio::spawn(async move {
                        if let Err(e) = manager.start_service(&name).await {
                            let _ = tx.send(GuiSystemEvent::Error(format!(
                                "Failed to start service {}: {}",
                                name, e
                            )));
                        } else {
                            let _ =
                                tx.send(GuiSystemEvent::Log(format!("Service {} started.", name)));
                        }
                    });
                }
            }
            for service_name in services_to_stop {
                if let Some(service) = state.services.iter_mut().find(|s| s.name == service_name) {
                    let _ = service.stop();
                    state.add_log(format!("Stopping service: {}", service_name));

                    let manager = _manager.clone();
                    let tx = state.system_event_tx.clone();
                    let name = service_name.clone();
                    tokio::spawn(async move {
                        if let Err(e) = manager.stop_service(&name).await {
                            let _ = tx.send(GuiSystemEvent::Error(format!(
                                "Failed to stop service {}: {}",
                                name, e
                            )));
                        } else {
                            let _ =
                                tx.send(GuiSystemEvent::Log(format!("Service {} stopped.", name)));
                        }
                    });
                }
            }
            if let Some(service_name) = service_to_edit {
                state.load_service_in_editor(
                    service_name,
                    "# Service content would be loaded here".to_string(),
                );
            }
        });
    });

    SidePanel::right("actions_panel").show(ctx, |ui| {
        ui.heading("Actions");

        // Simulator controls
        ui.collapsing("Simulator", |ui| {
            if ui.button("Start Simulator").clicked() {
                if !state.is_simulator_running {
                    state.add_log("Starting simulator...".to_string());
                    let manager = _manager.clone();
                    let tx = state.system_event_tx.clone();
                    let services_dir = state.config.services_directory.clone();
                    let default_port = state.config.default_port;

                    tokio::spawn(async move {
                        if let Err(e) = manager.load_services().await {
                            let _ = tx.send(GuiSystemEvent::Error(format!(
                                "Failed to load services: {}",
                                e
                            )));
                            return;
                        }
                        if let Err(e) = manager.start().await {
                            let _ = tx.send(GuiSystemEvent::Error(format!(
                                "Failed to start simulator: {}",
                                e
                            )));
                        } else {
                            let _ = tx.send(GuiSystemEvent::SimulatorStarted);

                            // Reload services to update UI
                            let config_loader = ConfigLoader::new(services_dir.clone());
                            match config_loader.load_all_services() {
                                Ok(defs) => {
                                    let mut services = Vec::new();
                                    for def in defs {
                                        let name = def.name.clone();
                                        let path = services_dir.join(format!("{}.yaml", name));
                                        let port = def
                                            .server
                                            .as_ref()
                                            .and_then(|s| s.port)
                                            .unwrap_or(default_port);
                                        let mut info =
                                            super::models::ServiceInfo::new(name, path, port);
                                        if let Some(endpoints) = def.endpoints {
                                            for ep in endpoints {
                                                info.endpoints.push(super::models::EndpointInfo {
                                                    method: ep.method,
                                                    path: ep.path,
                                                });
                                            }
                                        }
                                        // TODO: Check if running using manager.service_registry()
                                        info.status = ServiceStatus::Stopped; // Default
                                        services.push(info);
                                    }

                                    // Check which services are actually running
                                    let registry = manager.service_registry().read().await;
                                    for svc in &mut services {
                                        if let Some(inst) = registry.get_service(&svc.name) {
                                            if inst.read().await.is_running() {
                                                svc.mark_running();
                                            }
                                        }
                                    }

                                    let _ = tx.send(GuiSystemEvent::ServicesLoaded(services));
                                }
                                Err(e) => {
                                    let _ = tx.send(GuiSystemEvent::Error(format!(
                                        "Failed to list services: {}",
                                        e
                                    )));
                                }
                            }
                        }
                    });
                } else {
                    state.add_log("Simulator is already running.".to_string());
                }
            }
            if ui.button("Stop Simulator").clicked() {
                if state.is_simulator_running {
                    state.add_log("Stopping simulator...".to_string());
                    let manager = _manager.clone();
                    let tx = state.system_event_tx.clone();
                    tokio::spawn(async move {
                        if let Err(e) = manager.stop().await {
                            let _ = tx.send(GuiSystemEvent::Error(format!(
                                "Failed to stop simulator: {}",
                                e
                            )));
                        } else {
                            let _ = tx.send(GuiSystemEvent::SimulatorStopped);
                        }
                    });
                } else {
                    state.add_log("Simulator is not running.".to_string());
                }
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
                    state.refreshing_services = true;
                    let manager = _manager.clone();
                    let tx = state.system_event_tx.clone();
                    let services_dir = state.config.services_directory.clone();
                    let default_port = state.config.default_port;

                    tokio::spawn(async move {
                        if let Err(e) = manager.reload_services().await {
                            // If simulator is not running, we might still want to load services?
                            // manager.reload_services checks if active.
                            // If not active, we can call load_services?
                            // Try load_services if reload fails because not active
                            if e.to_string().contains("simulator is not running") {
                                if let Err(load_err) = manager.load_services().await {
                                    let _ = tx.send(GuiSystemEvent::Error(format!(
                                        "Failed to refresh services: {}",
                                        load_err
                                    )));
                                    return;
                                }
                            } else {
                                let _ = tx.send(GuiSystemEvent::Error(format!(
                                    "Failed to refresh services: {}",
                                    e
                                )));
                                return;
                            }
                        }

                        // Reload services to update UI (Same logic as above, can be extracted to a helper function if not in different tasks, but here we duplicate for simplicity)
                        let config_loader = ConfigLoader::new(services_dir.clone());
                        match config_loader.load_all_services() {
                            Ok(defs) => {
                                let mut services = Vec::new();
                                for def in defs {
                                    let name = def.name.clone();
                                    let path = services_dir.join(format!("{}.yaml", name));
                                    let port = def
                                        .server
                                        .as_ref()
                                        .and_then(|s| s.port)
                                        .unwrap_or(default_port);
                                    let mut info =
                                        super::models::ServiceInfo::new(name, path, port);
                                    if let Some(endpoints) = def.endpoints {
                                        for ep in endpoints {
                                            info.endpoints.push(super::models::EndpointInfo {
                                                method: ep.method,
                                                path: ep.path,
                                            });
                                        }
                                    }
                                    services.push(info);
                                }

                                // Check which services are actually running
                                let registry = manager.service_registry().read().await;
                                for svc in &mut services {
                                    if let Some(inst) = registry.get_service(&svc.name) {
                                        if inst.read().await.is_running() {
                                            svc.mark_running();
                                        }
                                    }
                                }

                                let _ = tx.send(GuiSystemEvent::ServicesRefreshed(services));
                            }
                            Err(e) => {
                                let _ = tx.send(GuiSystemEvent::Error(format!(
                                    "Failed to list services: {}",
                                    e
                                )));
                            }
                        }
                    });
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
                ui.label(if state.editor_state.dirty {
                    "Unsaved changes"
                } else {
                    "Saved"
                });
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
                    duration,
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
                        format!("{}", log.status_code),
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
            format!(
                "Editor - {} (Loading...)",
                state
                    .editor_state
                    .selected_service
                    .as_deref()
                    .unwrap_or("Unknown")
            )
        } else {
            format!(
                "Editor - {}",
                state
                    .editor_state
                    .selected_service
                    .as_deref()
                    .unwrap_or("Unknown")
            )
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
                                    .code_editor(),
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
