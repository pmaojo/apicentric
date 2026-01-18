use apicentric::simulator::config::ServiceDefinition;
use apicentric::simulator::openapi::{from_openapi, to_openapi};
<<<<<<< HEAD
use openapiv3::ReferenceOr;
=======
>>>>>>> origin/main
use serde_yaml::Value;

fn load_spec(path: &str) -> Value {
    let content = std::fs::read_to_string(path).unwrap();
    serde_yaml::from_str(&content).unwrap()
}

#[test]
fn import_openapi_to_service() {
    let spec = load_spec("tests/data/petstore.yaml");
    let service = from_openapi(&spec);
    assert_eq!(service.name, "Test Service");
<<<<<<< HEAD
    // OpenAPI v3 importer uses `servers` for base path; when missing, defaults to "/"
    assert_eq!(service.server.base_path, "/");
=======
    assert_eq!(service.server.base_path, "/api");
>>>>>>> origin/main
    assert_eq!(service.endpoints.len(), 1);
    let ep = &service.endpoints[0];
    assert_eq!(ep.method, "GET");
    assert_eq!(ep.path, "/pets");
}

mod response_examples {
    use super::*;

    fn response_body(service: &ServiceDefinition) -> &str {
        service.endpoints[0]
            .responses
            .get(&200)
            .unwrap()
            .body
            .as_str()
    }

    #[test]
    fn prefers_openapi2_examples_over_description() {
<<<<<<< HEAD
        // Test using OpenAPI v3 example fixture (v3 syntax)
        let spec = load_spec("tests/data/openapi3_examples.yaml");
        let service = from_openapi(&spec);
        assert!(response_body(&service).contains("\"message\": \"hello\""));
=======
        let spec = load_spec("tests/data/petstore_examples.yaml");
        let service = from_openapi(&spec);
        assert!(response_body(&service).contains("\"status\": \"ok\""));
>>>>>>> origin/main
    }

    #[test]
    fn generates_payload_when_only_schema_available() {
<<<<<<< HEAD
        // Use OpenAPI v3 schema fixture
        let spec = load_spec("tests/data/openapi3_schema.yaml");
        let service = from_openapi(&spec);
        let body = response_body(&service);
        assert!(body.contains("\"value\""));
        assert!(body.contains("\"count\""));
=======
        let spec = load_spec("tests/data/petstore_schema.yaml");
        let service = from_openapi(&spec);
        let body = response_body(&service);
        assert!(body.contains("\"id\""));
        assert!(body.contains("\"name\""));
>>>>>>> origin/main
    }

    #[test]
    fn supports_openapi3_examples_and_content() {
        let spec = load_spec("tests/data/openapi3_examples.yaml");
        let service = from_openapi(&spec);
        let body = response_body(&service);
        assert!(body.contains("\"message\": \"hello\""));
    }

    #[test]
    fn supports_openapi3_schema_generation() {
        let spec = load_spec("tests/data/openapi3_schema.yaml");
        let service = from_openapi(&spec);
        let body = response_body(&service);
        assert!(body.contains("\"value\""));
        assert!(body.contains("\"count\""));
    }
}

#[test]
fn export_service_to_openapi() {
    let yaml = std::fs::read_to_string("tests/data/service.yaml").unwrap();
    let service: ServiceDefinition = serde_yaml::from_str(&yaml).unwrap();
    let spec = to_openapi(&service);
    assert_eq!(spec.info.title, "Test Service");
<<<<<<< HEAD
    assert!(spec.paths.paths.contains_key("/pets"));
    let ops_ref = spec.paths.paths.get("/pets").unwrap();
    let ops = match ops_ref {
        ReferenceOr::Item(item) => item,
        ReferenceOr::Reference { reference: _ } => panic!("expected inline PathItem for /pets"),
    };
    assert!(ops.get.is_some());
    let op = ops.get.as_ref().unwrap();
    assert!(op
        .responses
        .responses
        .contains_key(&openapiv3::StatusCode::Code(200)));
=======
    assert!(spec.paths.contains_key("/pets"));
    let ops = spec.paths.get("/pets").unwrap();
    assert!(ops.get.is_some());
    let op = ops.get.as_ref().unwrap();
    assert!(op.responses.contains_key("200"));
>>>>>>> origin/main
}
