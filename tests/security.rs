use apicentric::cloud::handlers::{load_service, save_service, LoadServiceRequest, SaveServiceRequest};
use axum::Json;
use std::fs::File;
use std::io::Write;
use tempfile::TempDir;

#[tokio::test]
async fn test_path_traversal_prevention() {
    let temp_dir = TempDir::new().unwrap();
    let services_dir = temp_dir.path().join("services");
    std::fs::create_dir(&services_dir).unwrap();

    // Set env var to point to our temp services dir
    // Note: We need to use a mutex or run strictly sequentially if tests run in parallel,
    // but env vars are process-global. However, this is an integration test executable on its own.
    std::env::set_var("APICENTRIC_SERVICES_DIR", services_dir.to_str().unwrap());

    // 1. Test Load Service Traversal
    // Create a secret file OUTSIDE services dir
    let secret_file = temp_dir.path().join("secret.yaml");
    let mut file = File::create(&secret_file).unwrap();
    writeln!(file, "name: secret\nversion: 1.0.0").unwrap();

    // Try to load it using full path (simulating traversal/absolute path attack)
    let request = LoadServiceRequest {
        path: secret_file.to_str().unwrap().to_string(),
    };

    let result = load_service(Json(request)).await;
    match result {
        Ok(Json(response)) => {
            // It should fail because it looks for 'secret.yaml' inside services_dir, where it doesn't exist.
            // Even though 'secret.yaml' exists outside, the sanitizer forces it to look inside.
            if response.success {
                panic!("Security regression: Successfully loaded file outside services directory!");
            }
            assert!(response.error.is_some());
        }
        Err(_) => {
             // HTTP Error is also acceptable
        }
    }

    // 2. Test Save Service Traversal
    let target_file = temp_dir.path().join("pwned.yaml");
    // Try to write outside services dir
    let request = SaveServiceRequest {
        path: target_file.to_str().unwrap().to_string(),
        yaml: "name: pwned\nversion: 1.0.0".to_string(),
    };

    let result = save_service(Json(request)).await;
     match result {
        Ok(Json(response)) => {
            // It might succeed, but it should write to services_dir/pwned.yaml, NOT target_file
            if response.success {
                // Verify it didn't write to the target location
                if target_file.exists() {
                     panic!("Security regression: Successfully wrote file using traversal path!");
                }

                // Verify it DID write to the safe location (sanitization behavior)
                // The filename extracted from ".../pwned.yaml" is "pwned.yaml"
                let safe_file = services_dir.join("pwned.yaml");
                assert!(safe_file.exists(), "Should have written to safe location inside services dir");
            } else {
                // Failure is also safe
            }
        }
        Err(_) => {}
    }

    // 3. Test Valid Operation
    // Create a valid service inside services dir
    let valid_file = services_dir.join("valid.yaml");
    let mut file = File::create(&valid_file).unwrap();
    writeln!(file, "name: valid\nversion: 1.0.0").unwrap();

    let request = LoadServiceRequest {
        path: valid_file.to_str().unwrap().to_string(),
    };

    let result = load_service(Json(request)).await;
    match result {
        Ok(Json(response)) => {
            assert!(response.success, "Should successfully load valid file in services dir");
            assert!(response.data.unwrap().contains("name: valid"));
        }
        Err(e) => panic!("Request failed: {:?}", e),
    }
}
