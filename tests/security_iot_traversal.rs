use apicentric::cloud::iot_handlers::{delete_twin, save_twin, SaveTwinRequest};
use axum::extract::{Json, Path};
use std::env;
use std::fs;
use std::path::PathBuf;

#[tokio::test]
async fn test_iot_security_traversal() {
    // --- Part 1: Save Twin Traversal ---
    {
        println!("Running save_twin traversal test...");
        // Setup
        let temp_iot_dir = "temp_iot_traversal_test_save";
        let target_dir = "target";
        let pwned_file = "pwned.yaml";

        // Ensure cleanup
        let _ = fs::remove_dir_all(temp_iot_dir);
        let target_pwned_path = PathBuf::from(target_dir).join(pwned_file);
        let _ = fs::remove_file(&target_pwned_path);

        env::set_var("APICENTRIC_IOT_DIR", temp_iot_dir);

        // Payload
        let yaml = r#"
twin:
  name: "Test"
  physics: []
  transports: []
"#;
        let request = SaveTwinRequest {
            yaml: yaml.to_string(),
        };

        // Attack path: ../target/pwned
        // Should be sanitized to "pwned"
        let attack_path = format!("../{}/pwned", target_dir);

        // Call
        let result = save_twin(Path(attack_path.clone()), Json(request)).await;

        // It should succeed because we sanitize and save
        assert!(result.is_ok(), "save_twin failed: {:?}", result.err());

        // Verification 1: File should NOT exist in target (vulnerability blocked)
        if target_pwned_path.exists() {
            // Cleanup
            let _ = fs::remove_file(&target_pwned_path);
            let _ = fs::remove_dir_all(temp_iot_dir);
            panic!(
                "Vulnerability STILL EXIST: File created at {:?}",
                target_pwned_path
            );
        }

        // Verification 2: File SHOULD exist in temp_iot_dir (sanitization worked)
        let sanitized_path = PathBuf::from(temp_iot_dir).join(pwned_file);
        if !sanitized_path.exists() {
            let _ = fs::remove_dir_all(temp_iot_dir);
            panic!(
                "Sanitization failed: File NOT created at {:?}",
                sanitized_path
            );
        }

        println!(
            "Security Fix Verified: File created at {:?}",
            sanitized_path
        );

        // Cleanup
        let _ = fs::remove_dir_all(temp_iot_dir);
    }

    // --- Part 2: Delete Twin Traversal ---
    {
        println!("Running delete_twin traversal test...");
        // Setup
        let temp_iot_dir = "temp_iot_traversal_test_delete";
        let target_dir = "target";
        let victim_file = "victim.yaml";

        // Create victim file in target
        let target_victim_path = PathBuf::from(target_dir).join(victim_file);
        // Ensure parent exists (target always exists)
        fs::write(&target_victim_path, "important data").unwrap();

        // Ensure cleanup of temp dir
        let _ = fs::remove_dir_all(temp_iot_dir);

        env::set_var("APICENTRIC_IOT_DIR", temp_iot_dir);
        // Create temp dir
        fs::create_dir_all(temp_iot_dir).unwrap();

        // Attack path: ../target/victim
        let attack_path = format!("../{}/victim", target_dir);

        // Call delete_twin
        // This should attempt to delete "temp_iot_delete_traversal_test/victim.yaml" (sanitized)
        // It should NOT delete target_victim_path.

        // Let's create the sanitized file first to verify it deletes that one instead.
        let sanitized_path = PathBuf::from(temp_iot_dir).join("victim.yaml");
        fs::write(&sanitized_path, "decoy data").unwrap();

        let result = delete_twin(Path(attack_path.clone())).await;

        assert!(result.is_ok(), "delete_twin failed: {:?}", result.err());

        // Verification 1: Victim in target MUST still exist
        if !target_victim_path.exists() {
            let _ = fs::remove_dir_all(temp_iot_dir);
            // Restore victim file just in case
            fs::write(&target_victim_path, "important data").unwrap();
            panic!(
                "Vulnerability STILL EXIST: File deleted at {:?}",
                target_victim_path
            );
        }

        // Verification 2: Decoy in temp MUST be deleted
        if sanitized_path.exists() {
            let _ = fs::remove_file(&target_victim_path);
            let _ = fs::remove_dir_all(temp_iot_dir);
            panic!(
                "Sanitization failed: Decoy file NOT deleted at {:?}",
                sanitized_path
            );
        }

        println!("Security Fix Verified (Delete): Target file preserved, decoy deleted.");

        // Cleanup
        let _ = fs::remove_file(&target_victim_path);
        let _ = fs::remove_dir_all(temp_iot_dir);
    }
}
