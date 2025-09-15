use std::path::PathBuf;

/// Types of errors that can occur during service loading
#[derive(Debug, Clone, PartialEq)]
pub enum LoadErrorType {
    FileAccess,
    Parsing,
    Validation,
    DuplicateName,
}

/// Detailed error information for service loading
#[derive(Debug, Clone)]
pub struct LoadError {
    pub file_path: PathBuf,
    pub error_type: LoadErrorType,
    pub message: String,
}

/// Summary of validation results
#[derive(Debug, Clone)]
pub struct ValidationSummary {
    pub valid_count: usize,
    pub invalid_count: usize,
    pub total_files: usize,
    pub errors: Vec<LoadError>,
}

impl ValidationSummary {
    pub fn is_all_valid(&self) -> bool {
        self.invalid_count == 0 && self.valid_count > 0
    }

    pub fn success_rate(&self) -> f64 {
        if self.total_files == 0 {
            0.0
        } else {
            (self.valid_count as f64 / self.total_files as f64) * 100.0
        }
    }
}

pub fn summarize(total_files: usize, errors: Vec<LoadError>) -> ValidationSummary {
    let invalid_count = errors.len();
    let valid_count = total_files.saturating_sub(invalid_count);
    ValidationSummary {
        valid_count,
        invalid_count,
        total_files,
        errors,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn summary_calculates_success_rate() {
        let summary = summarize(4, vec![LoadError {
            file_path: PathBuf::from("a.yaml"),
            error_type: LoadErrorType::Validation,
            message: "err".into(),
        }]);
        assert_eq!(summary.valid_count, 3);
        assert_eq!(summary.invalid_count, 1);
        assert!((summary.success_rate() - 75.0).abs() < f64::EPSILON);
    }
}
