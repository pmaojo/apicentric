use apicentric::utils::{DirectoryScanner, FileSystemUtils};
use std::fs::File;
use std::io::Write;
use tempfile::TempDir;

fn create_test_structure() -> TempDir {
    let temp_dir = TempDir::new().unwrap();
    let base = temp_dir.path();
    std::fs::create_dir_all(base.join("app/routes/login/test")).unwrap();
    std::fs::create_dir_all(base.join("app/routes/dashboard/test")).unwrap();
<<<<<<< HEAD
    std::fs::create_dir_all(base.join("screenshots")).unwrap();
    std::fs::create_dir_all(base.join("videos")).unwrap();
    File::create(base.join("app/routes/login/test/login.test.ts")).unwrap();
    File::create(base.join("app/routes/dashboard/test/dashboard.spec.ts")).unwrap();
    File::create(base.join("screenshots/test.png")).unwrap();
    File::create(base.join("videos/test.mp4")).unwrap();
=======
    std::fs::create_dir_all(base.join("cypress/screenshots")).unwrap();
    std::fs::create_dir_all(base.join("cypress/videos")).unwrap();
    File::create(base.join("app/routes/login/test/login.cy.ts")).unwrap();
    File::create(base.join("app/routes/dashboard/test/dashboard.cy.ts")).unwrap();
    File::create(base.join("cypress/screenshots/test.png")).unwrap();
    File::create(base.join("cypress/videos/test.mp4")).unwrap();
>>>>>>> origin/main
    File::create(base.join("app/routes/login/route.tsx")).unwrap();
    temp_dir
}

#[test]
fn test_resolve_glob_pattern_success() {
    let temp_dir = create_test_structure();
<<<<<<< HEAD
    let pattern = format!("{}/**/test/*.test.ts", temp_dir.path().display());
    let result = FileSystemUtils::resolve_glob_pattern(&pattern, None);
    assert!(result.is_ok());
    let paths = result.unwrap();
    assert_eq!(paths.len(), 1);
    assert!(paths
        .iter()
        .any(|p| p.to_string_lossy().contains("login.test.ts")));
=======
    let pattern = format!("{}/**/test/*.cy.ts", temp_dir.path().display());
    let result = FileSystemUtils::resolve_glob_pattern(&pattern, None);
    assert!(result.is_ok());
    let paths = result.unwrap();
    assert_eq!(paths.len(), 2);
    assert!(paths.iter().any(|p| p.to_string_lossy().contains("login.cy.ts")));
    assert!(paths.iter().any(|p| p.to_string_lossy().contains("dashboard.cy.ts")));
>>>>>>> origin/main
}

#[test]
fn test_resolve_glob_pattern_invalid_syntax() {
    let result = FileSystemUtils::resolve_glob_pattern("[invalid", None);
    assert!(result.is_err());
<<<<<<< HEAD
    if let Err(apicentric::errors::ApicentricError::FileSystem {
        message,
        suggestion,
    }) = result
    {
=======
    if let Err(apicentric::errors::ApicentricError::FileSystem { message, suggestion }) = result {
>>>>>>> origin/main
        assert!(message.contains("Invalid glob pattern syntax"));
        assert!(suggestion.is_some());
    } else {
        panic!("Expected FileSystem error");
    }
}

#[test]
fn test_resolve_glob_pattern_no_matches() {
    let temp_dir = create_test_structure();
<<<<<<< HEAD
    let pattern = format!("{}/**/nonexistent/*.test.ts", temp_dir.path().display());
    let result = FileSystemUtils::resolve_glob_pattern(&pattern, None);
    assert!(result.is_err());
    if let Err(apicentric::errors::ApicentricError::FileSystem {
        message,
        suggestion: _,
    }) = result
    {
=======
    let pattern = format!("{}/**/nonexistent/*.cy.ts", temp_dir.path().display());
    let result = FileSystemUtils::resolve_glob_pattern(&pattern, None);
    assert!(result.is_err());
    if let Err(apicentric::errors::ApicentricError::FileSystem { message, suggestion: _ }) = result {
>>>>>>> origin/main
        assert!(message.contains("No valid files found"));
    }
}

#[test]
fn test_validate_test_files() {
    let temp_dir = create_test_structure();
    let paths = vec![
<<<<<<< HEAD
        temp_dir.path().join("app/routes/login/test/login.test.ts"),
        temp_dir.path().join("screenshots/test.png"),
=======
        temp_dir.path().join("app/routes/login/test/login.cy.ts"),
        temp_dir.path().join("cypress/screenshots/test.png"),
>>>>>>> origin/main
        temp_dir.path().join("app/routes/login/route.tsx"),
    ];
    let (valid_files, issues) = FileSystemUtils::validate_test_files(&paths);
    assert_eq!(valid_files.len(), 1);
    assert_eq!(issues.len(), 2);
<<<<<<< HEAD
    assert!(valid_files[0].to_string_lossy().contains("login.test.ts"));
=======
    assert!(valid_files[0].to_string_lossy().contains("login.cy.ts"));
>>>>>>> origin/main
}

#[cfg(unix)]
#[test]
fn test_validate_single_test_file_posix() {
    let temp_dir = create_test_structure();
<<<<<<< HEAD
    let valid_file = temp_dir.path().join("app/routes/login/test/login.test.ts");
    assert!(FileSystemUtils::validate_single_test_file(&valid_file).is_ok());
    let invalid_file = temp_dir.path().join("app/routes/login/route.tsx");
    assert!(FileSystemUtils::validate_single_test_file(&invalid_file).is_err());
    let excluded_file = temp_dir.path().join("screenshots/test.png");
=======
    let valid_file = temp_dir.path().join("app/routes/login/test/login.cy.ts");
    assert!(FileSystemUtils::validate_single_test_file(&valid_file).is_ok());
    let invalid_file = temp_dir.path().join("app/routes/login/route.tsx");
    assert!(FileSystemUtils::validate_single_test_file(&invalid_file).is_err());
    let excluded_file = temp_dir.path().join("cypress/screenshots/test.png");
>>>>>>> origin/main
    assert!(FileSystemUtils::validate_single_test_file(&excluded_file).is_err());
}

#[cfg(windows)]
#[test]
fn test_validate_single_test_file_windows() {
    let temp_dir = create_test_structure();
<<<<<<< HEAD
    let valid_file = temp_dir
        .path()
        .join("app\\routes\\login\\test\\login.test.ts");
    assert!(FileSystemUtils::validate_single_test_file(&valid_file).is_ok());
    let invalid_file = temp_dir.path().join("app\\routes\\login\\route.tsx");
    assert!(FileSystemUtils::validate_single_test_file(&invalid_file).is_err());
    let excluded_file = temp_dir.path().join("screenshots\\test.png");
=======
    let valid_file = temp_dir.path().join("app\\routes\\login\\test\\login.cy.ts");
    assert!(FileSystemUtils::validate_single_test_file(&valid_file).is_ok());
    let invalid_file = temp_dir.path().join("app\\routes\\login\\route.tsx");
    assert!(FileSystemUtils::validate_single_test_file(&invalid_file).is_err());
    let excluded_file = temp_dir.path().join("cypress\\screenshots\\test.png");
>>>>>>> origin/main
    assert!(FileSystemUtils::validate_single_test_file(&excluded_file).is_err());
}

#[test]
fn test_ensure_directory_exists() {
    let temp_dir = TempDir::new().unwrap();
    let new_dir = temp_dir.path().join("new/nested/directory");
    assert!(!new_dir.exists());
    let result = FileSystemUtils::ensure_directory_exists(&new_dir);
    assert!(result.is_ok());
    assert!(new_dir.exists());
    assert!(new_dir.is_dir());
}

#[test]
fn test_safe_read_file() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.txt");
    let mut file = File::create(&file_path).unwrap();
    writeln!(file, "test content").unwrap();
    let result = FileSystemUtils::safe_read_file(&file_path);
    assert!(result.is_ok());
    assert!(result.unwrap().contains("test content"));
}

#[test]
fn test_safe_write_file() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("nested/test.txt");
    let result = FileSystemUtils::safe_write_file(&file_path, "test content");
    assert!(result.is_ok());
    assert!(file_path.exists());
    let content = std::fs::read_to_string(&file_path).unwrap();
    assert_eq!(content, "test content");
}

#[test]
fn test_validate_directory_access() {
    let temp_dir = create_test_structure();
    let valid_dir = temp_dir.path().join("app");
    assert!(FileSystemUtils::validate_directory_access(&valid_dir).is_ok());
    let invalid_dir = temp_dir.path().join("nonexistent");
    assert!(FileSystemUtils::validate_directory_access(&invalid_dir).is_err());
    let file_path = temp_dir.path().join("app/routes/login/route.tsx");
    assert!(FileSystemUtils::validate_directory_access(&file_path).is_err());
}

#[test]
fn test_directory_scanner() {
    let temp_dir = create_test_structure();
    let scanner = DirectoryScanner::new(temp_dir.path().to_path_buf());
<<<<<<< HEAD
    let result = scanner.scan_for_files("**/test/*.test.ts");
    assert!(result.is_ok());
    let files = result.unwrap();
    assert_eq!(files.len(), 1);
=======
    let result = scanner.scan_for_files("**/test/*.cy.ts");
    assert!(result.is_ok());
    let files = result.unwrap();
    assert_eq!(files.len(), 2);
>>>>>>> origin/main
}

#[test]
fn test_count_files_in_directory() {
    let temp_dir = create_test_structure();
    let test_dir = temp_dir.path().join("app/routes/login/test");
<<<<<<< HEAD
    let result = FileSystemUtils::count_files_in_directory(&test_dir, Some(".test.ts"));
=======
    let result = FileSystemUtils::count_files_in_directory(&test_dir, Some(".cy.ts"));
>>>>>>> origin/main
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 1);
}
