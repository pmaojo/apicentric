use mockforge::simulator::config::ServiceDefinition;
use mockforge::simulator::openapi::{from_openapi, to_openapi};

#[test]
fn import_openapi_to_service() {
    let spec = openapi::from_path("tests/data/petstore.yaml").unwrap();
    let service = from_openapi(&spec);
    assert_eq!(service.name, "Test Service");
    assert_eq!(service.server.base_path, "/api");
    assert_eq!(service.endpoints.len(), 1);
    let ep = &service.endpoints[0];
    assert_eq!(ep.method, "GET");
    assert_eq!(ep.path, "/pets");
}

#[test]
fn export_service_to_openapi() {
    let yaml = std::fs::read_to_string("tests/data/service.yaml").unwrap();
    let service: ServiceDefinition = serde_yaml::from_str(&yaml).unwrap();
    let spec = to_openapi(&service);
    assert_eq!(spec.info.title, "Test Service");
    assert!(spec.paths.contains_key("/pets"));
    let ops = spec.paths.get("/pets").unwrap();
    assert!(ops.get.is_some());
    let op = ops.get.as_ref().unwrap();
    assert!(op.responses.contains_key("200"));
}
