use async_trait::async_trait;
use apicentric::adapters::service_spec_loader::YamlServiceSpecLoader;
use apicentric::domain::ports::contract::{ServiceSpecLoader, ServiceSpec, EndpointSpec, ResponseSpec};
use apicentric::domain::contract_testing::HttpMethod;
use apicentric::utils::FileReader;
use std::path::Path;
use std::sync::Arc;

// Mock file reader used for testing
struct MockFileReader {
    content: String,
}

#[async_trait]
impl FileReader for MockFileReader {
    async fn read_to_string(&self, _path: &Path) -> std::io::Result<String> {
        Ok(self.content.clone())
    }
}

#[tokio::test]
async fn load_with_mock_reader() {
    let yaml_content = r#"
name: test-service
port: 8080
endpoints:
  - path: "/hello"
    method: "GET"
    response:
      status: 200
"#;
    let reader = Arc::new(MockFileReader { content: yaml_content.to_string() });
    let loader = YamlServiceSpecLoader::with_file_reader(reader);
    let spec = loader.load("ignored.yaml").await.unwrap();
    assert_eq!(spec.name, "test-service");
    assert_eq!(spec.endpoints.len(), 1);
}

#[tokio::test]
async fn test_load_valid_yaml_spec() {
    let yaml_content = r#"
name: "test-service"
port: 3000
basePath: "/api/v1"
fixtures:
  users:
    - id: 1
      name: "John Doe"
      email: "john@example.com"
    - id: 2
      name: "Jane Smith"
      email: "jane@example.com"
endpoints:
  - path: "/users"
    method: "GET"
    conditions:
      - "limit=10"
      - "offset=0"
    response:
      status: 200
      headers:
        Content-Type: "application/json"
      body: |
        {
          "users": {{#each fixtures.users}}
          {
            "id": {{id}},
            "name": "{{name}}",
            "email": "{{email}}"
          }{{#unless @last}},{{/unless}}
          {{/each}}
        }
  - path: "/users/{id}"
    method: "GET"
    response:
      status: 200
      headers:
        Content-Type: "application/json"
      body: |
        {
          "id": {{id}},
          "name": "User {{id}}",
          "email": "user{{id}}@example.com"
        }
"#;
    let mut temp_file = tempfile::NamedTempFile::new().unwrap();
    use std::io::Write;
    temp_file.write_all(yaml_content.as_bytes()).unwrap();
    temp_file.flush().unwrap();
    let loader = YamlServiceSpecLoader::new();
    let spec = loader
        .load(temp_file.path().to_str().unwrap())
        .await
        .unwrap();
    assert_eq!(spec.name, "test-service");
    assert_eq!(spec.port, 3000);
    assert_eq!(spec.base_path, "/api/v1");
    assert_eq!(spec.endpoints.len(), 2);
}

#[tokio::test]
async fn test_load_invalid_yaml() {
    let invalid_yaml = r#"
name: "test-service"
port: "invalid-port"
endpoints: "not-an-array"
"#;
    let mut temp_file = tempfile::NamedTempFile::new().unwrap();
    use std::io::Write;
    temp_file.write_all(invalid_yaml.as_bytes()).unwrap();
    temp_file.flush().unwrap();
    let loader = YamlServiceSpecLoader::new();
    let result = loader.load(temp_file.path().to_str().unwrap()).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_validate_missing_fields() {
    let loader = YamlServiceSpecLoader::new();
    let spec = ServiceSpec {
        name: "".to_string(),
        port: 3000,
        base_path: "/api".to_string(),
        fixtures: serde_json::json!({}),
        endpoints: vec![EndpointSpec {
            path: "/".to_string(),
            method: HttpMethod::GET,
            conditions: vec![],
            response: ResponseSpec { status: 200, headers: Default::default(), body_template: String::new() },
        }],
    };
    let result = loader.validate(&spec).await;
    assert!(result.is_err());
}


