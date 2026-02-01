use axum::{
    extract::{Multipart, Path},
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};

use crate::cloud::error::{ApiError, ApiErrorCode, ErrorResponse};
use crate::cloud::handlers::ApiResponse;
use crate::iot::config::TwinConfig;

fn sanitize_twin_name(name: &str) -> Result<String, ApiError> {
    std::path::Path::new(name)
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| {
            ApiError::bad_request(
                ApiErrorCode::InvalidParameter,
                "Invalid twin name".to_string(),
            )
        })
        .map(|s| s.to_string())
}

/// Lists all available twins.
pub async fn list_twins() -> Result<Json<ApiResponse<Vec<String>>>, ApiError> {
    let iot_dir = std::env::var("APICENTRIC_IOT_DIR").unwrap_or_else(|_| "iot".to_string());
    let path = std::path::Path::new(&iot_dir);

    if !path.exists() {
        std::fs::create_dir_all(path).map_err(|e| {
            ApiError::internal_server_error(format!("Failed to create IoT directory: {}", e))
        })?;
    }

    let mut twins = Vec::new();
    let entries = std::fs::read_dir(path).map_err(|e| {
        ApiError::internal_server_error(format!("Failed to read IoT directory: {}", e))
    })?;

    for entry in entries.flatten() {
        let path = entry.path();
        if let Some(ext) = path.extension() {
            if ext == "yaml" || ext == "yml" {
                if let Some(stem) = path.file_stem() {
                    if let Some(name) = stem.to_str() {
                        twins.push(name.to_string());
                    }
                }
            }
        }
    }

    Ok(Json(ApiResponse::success(twins)))
}

#[derive(Serialize)]
pub struct TwinDetailResponse {
    pub name: String,
    pub yaml: String,
    pub config: TwinConfig,
}

/// Gets a specific twin definition.
pub async fn get_twin(
    Path(name): Path<String>,
) -> Result<Json<ApiResponse<TwinDetailResponse>>, ApiError> {
    let iot_dir = std::env::var("APICENTRIC_IOT_DIR").unwrap_or_else(|_| "iot".to_string());

    // Sentinel: Sanitize filename to prevent directory traversal
    let safe_name = sanitize_twin_name(&name)?;

    let path = std::path::Path::new(&iot_dir).join(format!("{}.yaml", safe_name));

    if !path.exists() {
        return Err(ErrorResponse::service_not_found(&name).into()); // Reuse service not found or generic not found
    }

    let content = std::fs::read_to_string(&path)
        .map_err(|e| ApiError::internal_server_error(format!("Failed to read twin file: {}", e)))?;

    let config: TwinConfig = serde_yaml::from_str(&content).map_err(|e| {
        ApiError::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            ApiErrorCode::InvalidYaml,
            format!("Invalid YAML: {}", e),
        )
    })?;

    Ok(Json(ApiResponse::success(TwinDetailResponse {
        name,
        yaml: content,
        config,
    })))
}

#[derive(Deserialize)]
pub struct SaveTwinRequest {
    pub yaml: String,
}

/// Creates or updates a twin.
pub async fn save_twin(
    Path(name): Path<String>,
    Json(request): Json<SaveTwinRequest>,
) -> Result<Json<ApiResponse<String>>, ApiError> {
    let iot_dir = std::env::var("APICENTRIC_IOT_DIR").unwrap_or_else(|_| "iot".to_string());
    let dir_path = std::path::Path::new(&iot_dir);

    if !dir_path.exists() {
        std::fs::create_dir_all(dir_path).map_err(|e| {
            ApiError::internal_server_error(format!("Failed to create IoT directory: {}", e))
        })?;
    }

    // Sentinel: Sanitize filename to prevent directory traversal
    let safe_name = sanitize_twin_name(&name)?;

    let path = dir_path.join(format!("{}.yaml", safe_name));

    // Validate YAML
    let _: TwinConfig = serde_yaml::from_str(&request.yaml).map_err(|e| {
        ApiError::new(
            StatusCode::BAD_REQUEST,
            ApiErrorCode::InvalidYaml,
            format!("Invalid YAML: {}", e),
        )
    })?;

    std::fs::write(&path, &request.yaml).map_err(|e| {
        ApiError::internal_server_error(format!("Failed to write twin file: {}", e))
    })?;

    Ok(Json(ApiResponse::success(format!("Twin '{}' saved", name))))
}

/// Deletes a twin.
pub async fn delete_twin(Path(name): Path<String>) -> Result<Json<ApiResponse<String>>, ApiError> {
    let iot_dir = std::env::var("APICENTRIC_IOT_DIR").unwrap_or_else(|_| "iot".to_string());

    // Sentinel: Sanitize filename to prevent directory traversal
    let safe_name = sanitize_twin_name(&name)?;

    let path = std::path::Path::new(&iot_dir).join(format!("{}.yaml", safe_name));

    if !path.exists() {
        return Err(ErrorResponse::service_not_found(&name).into());
    }

    std::fs::remove_file(&path).map_err(|e| {
        ApiError::internal_server_error(format!("Failed to delete twin file: {}", e))
    })?;

    Ok(Json(ApiResponse::success(format!(
        "Twin '{}' deleted",
        name
    ))))
}

/// Uploads a CSV file for replay strategy.
pub async fn upload_replay_data(
    mut multipart: Multipart,
) -> Result<Json<ApiResponse<String>>, ApiError> {
    let iot_dir = std::env::var("APICENTRIC_IOT_DIR").unwrap_or_else(|_| "iot".to_string());
    let dir_path = std::path::Path::new(&iot_dir);

    if !dir_path.exists() {
        std::fs::create_dir_all(dir_path).map_err(|e| {
            ApiError::internal_server_error(format!("Failed to create IoT directory: {}", e))
        })?;
    }

    let mut filename = String::new();
    let mut saved_path = String::new();

    while let Some(mut field) = multipart.next_field().await.map_err(|e| {
        ApiError::bad_request(
            ApiErrorCode::InvalidParameter,
            format!("Multipart error: {}", e),
        )
    })? {
        let name = field.name().unwrap_or("").to_string();

        if name == "file" {
            let raw_file_name = field.file_name().unwrap_or("data.csv");

            // Sanitize filename to prevent directory traversal
            let file_name = std::path::Path::new(raw_file_name)
                .file_name()
                .ok_or_else(|| {
                    ApiError::bad_request(
                        ApiErrorCode::InvalidParameter,
                        "Invalid filename".to_string(),
                    )
                })?
                .to_string_lossy()
                .to_string();

            // Sentinel: Enforce .csv extension to prevent uploading executables or other malicious files
            if !file_name.to_lowercase().ends_with(".csv") {
                return Err(ApiError::bad_request(
                    ApiErrorCode::InvalidParameter,
                    "Only CSV files are allowed for replay data".to_string(),
                ));
            }

            filename = file_name.clone();

            let path = dir_path.join(&file_name);
            let mut file = std::fs::File::create(&path).map_err(|e| {
                ApiError::internal_server_error(format!("Failed to create file: {}", e))
            })?;

            let mut total_size = 0;
            const MAX_SIZE: usize = 10 * 1024 * 1024;

            while let Some(chunk) = field.chunk().await.map_err(|e| {
                ApiError::internal_server_error(format!("Failed to read file chunk: {}", e))
            })? {
                total_size += chunk.len();
                if total_size > MAX_SIZE {
                    // Close file handle and delete partial file
                    drop(file);
                    let _ = std::fs::remove_file(&path);
                    return Err(ApiError::bad_request(
                        ApiErrorCode::InvalidParameter,
                        "File size exceeds 10MB limit".to_string(),
                    ));
                }

                use std::io::Write;
                file.write_all(&chunk).map_err(|e| {
                    ApiError::internal_server_error(format!("Failed to write file chunk: {}", e))
                })?;
            }

            saved_path = path.to_string_lossy().to_string();
        }
    }

    if saved_path.is_empty() {
        return Err(ApiError::bad_request(
            ApiErrorCode::InvalidParameter,
            "No file uploaded".to_string(),
        ));
    }

    Ok(Json(ApiResponse::success(filename)))
}

#[derive(Serialize)]
pub struct GraphNode {
    id: String,
    label: String,
    data: GraphNodeData,
    position: GraphNodePosition,
    #[serde(rename = "type")]
    node_type: String,
}

#[derive(Serialize)]
pub struct GraphNodeData {
    label: String,
}

#[derive(Serialize)]
pub struct GraphNodePosition {
    x: i32,
    y: i32,
}

#[derive(Serialize)]
pub struct GraphEdge {
    id: String,
    source: String,
    target: String,
    label: Option<String>,
    animated: bool,
}

#[derive(Serialize)]
pub struct GraphResponse {
    nodes: Vec<GraphNode>,
    edges: Vec<GraphEdge>,
}

fn mqtt_match(subscription: &str, topic: &str) -> bool {
    if subscription == "#" {
        return true;
    }
    if subscription == topic {
        return true;
    }

    let sub_parts: Vec<&str> = subscription.split('/').collect();
    let topic_parts: Vec<&str> = topic.split('/').collect();

    for (i, sub_part) in sub_parts.iter().enumerate() {
        if *sub_part == "#" {
            return true;
        }
        if i >= topic_parts.len() {
            return false;
        }
        if *sub_part != "+" && *sub_part != topic_parts[i] {
            return false;
        }
    }

    if topic_parts.len() > sub_parts.len() {
        return false;
    }

    true
}

/// Generates the IoT system graph.
pub async fn get_iot_graph() -> Result<Json<ApiResponse<GraphResponse>>, ApiError> {
    let iot_dir = std::env::var("APICENTRIC_IOT_DIR").unwrap_or_else(|_| "iot".to_string());
    let path = std::path::Path::new(&iot_dir);

    if !path.exists() {
        return Ok(Json(ApiResponse::success(GraphResponse {
            nodes: vec![],
            edges: vec![],
        })));
    }

    let entries = std::fs::read_dir(path).map_err(|e| {
        ApiError::internal_server_error(format!("Failed to read IoT directory: {}", e))
    })?;

    struct TwinInfo {
        id: String,
        name: String,
        pub_topics: Vec<String>,
        sub_topics: Vec<String>,
    }

    let mut twins_info: Vec<TwinInfo> = Vec::new();

    for entry in entries.flatten() {
        let path = entry.path();
        if let Some(ext) = path.extension() {
            if ext == "yaml" || ext == "yml" {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    if let Ok(config) = serde_yaml::from_str::<TwinConfig>(&content) {
                        let mut pub_topics = Vec::new();
                        let mut sub_topics = Vec::new();

                        for transport in config.twin.transports {
                            let prefix = transport
                                .params
                                .get("topic_prefix")
                                .and_then(|v| v.as_str())
                                .unwrap_or("sensors")
                                .to_string();

                            for physics in &config.twin.physics {
                                pub_topics.push(format!("{}/{}", prefix, physics.variable));
                            }

                            if let Some(subs) = transport.params.get("subscriptions") {
                                if let Some(subs_seq) = subs.as_sequence() {
                                    for sub in subs_seq {
                                        if let Some(topic) = sub.as_str() {
                                            sub_topics.push(topic.to_string());
                                        }
                                    }
                                }
                            }
                        }

                        twins_info.push(TwinInfo {
                            id: config.twin.name.clone(),
                            name: config.twin.name,
                            pub_topics,
                            sub_topics,
                        });
                    }
                }
            }
        }
    }

    let mut nodes = Vec::new();
    let mut edges = Vec::new();

    for (i, twin) in twins_info.iter().enumerate() {
        nodes.push(GraphNode {
            id: twin.id.clone(),
            label: twin.name.clone(),
            data: GraphNodeData {
                label: twin.name.clone(),
            },
            position: GraphNodePosition {
                x: (i as i32) * 250,
                y: 100 + (if i % 2 == 0 { 50 } else { -50 }),
            },
            node_type: "default".to_string(),
        });
    }

    let mut edge_count = 0;
    for source in &twins_info {
        for target in &twins_info {
            if source.id == target.id {
                continue;
            }

            for pub_topic in &source.pub_topics {
                for sub_topic in &target.sub_topics {
                    if mqtt_match(sub_topic, pub_topic) {
                        edges.push(GraphEdge {
                            id: format!("e{}-{}", edge_count, source.id),
                            source: source.id.clone(),
                            target: target.id.clone(),
                            label: Some(pub_topic.clone()),
                            animated: true,
                        });
                        edge_count += 1;
                    }
                }
            }
        }
    }

    Ok(Json(ApiResponse::success(GraphResponse { nodes, edges })))
}
