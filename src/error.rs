use thiserror::Error;

pub type AppResult<T> = Result<T, AppError>;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    JsonError(String),

    #[error("JSON serialization error: {0}")]
    JsonSerializationError(#[from] serde_json::Error),

    #[error("Validation error: {0}")]
    ValidationError(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_database_error_display() {
        let sqlx_err = sqlx::Error::RowNotFound;
        let app_err = AppError::DatabaseError(sqlx_err);
        let msg = format!("{}", app_err);
        assert!(msg.contains("Database error:"));
        assert!(msg.contains("row"));
    }

    #[test]
    fn test_io_error_display() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let app_err = AppError::IoError(io_err);
        assert!(format!("{}", app_err).contains("IO error:"));
        assert!(format!("{}", app_err).contains("file not found"));
    }

    #[test]
    fn test_json_error_display() {
        let app_err = AppError::JsonError("invalid syntax".to_string());
        assert_eq!(format!("{}", app_err), "JSON error: invalid syntax");
    }

    #[test]
    fn test_json_serialization_error_display() {
        // Create an invalid JSON number to trigger serialization error
        let invalid_json = "NaN";
        let result: Result<serde_json::Value, _> = serde_json::from_str(invalid_json);
        if let Err(e) = result {
            let app_err = AppError::JsonSerializationError(e);
            assert!(format!("{}", app_err).contains("JSON serialization error:"));
        }
    }

    #[test]
    fn test_validation_error_display() {
        let app_err = AppError::ValidationError("amount must be positive".to_string());
        assert_eq!(format!("{}", app_err), "Validation error: amount must be positive");
    }

    #[test]
    fn test_from_sqlx_error() {
        let sqlx_err = sqlx::Error::RowNotFound;
        let app_err: AppError = sqlx_err.into();
        assert!(matches!(app_err, AppError::DatabaseError(_)));
    }

    #[test]
    fn test_from_io_error() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "test");
        let app_err: AppError = io_err.into();
        assert!(matches!(app_err, AppError::IoError(_)));
    }

    #[test]
    fn test_from_serde_json_error() {
        let invalid = serde_json::from_str::<serde_json::Value>("invalid");
        assert!(invalid.is_err());
        let app_err: AppError = invalid.unwrap_err().into();
        assert!(matches!(app_err, AppError::JsonSerializationError(_)));
    }
}
