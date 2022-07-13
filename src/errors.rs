use std::fmt;
use rocket_contrib::json::Json;
use serde::{Serialize};
use validator::{Validate, ValidationError, ValidationErrors};

/// Error for app
#[derive(Debug)]
pub struct AppError {
    /// code of error
    pub code: usize,
}

impl AppError {
    /// Returns a error with args
    ///
    /// # Examples
    ///
    /// ```
    /// use blockchain::errors::{AppError};
    /// let error = AppError::new(1000);
    /// ```
    pub fn new(code: usize) -> Self {
        Self { code }
    }
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let message = match self.code {
            1000 => "Fail to add block with invalid block",
            2000 => "Fail to sign in",
            2001 => "Fail to process transactions",
            2002 => "Fail to send transactions",
            3000 => "Fail to read private key",
            3001 => "Fail to create private key",
            3002 => "Fail to write private key",
            4000 => "Fail to add transaction pool with invalid unspent tx outs",
            4001 => "Fail to add transaction pool with invalid transaction pool",
            _ => "Unknown",
        };

        write!(f, "[{}]: {}", self.code, message)
    }
}

/// Error for api
#[derive(Debug, Serialize)]
pub struct ApiError {
    /// code of error
    code: usize,

    /// message of error
    message: String,

    /// errors of validation
    errors: Option<ValidationErrors>,
}

impl ApiError {
    /// Returns a error with args
    ///
    /// # Examples
    ///
    /// ```
    /// use blockchain::errors::{ApiError};
    /// use validator::{ValidationErrors};
    /// let error = ApiError::new(404, "Not found".to_string(), Some(ValidationErrors::new()));
    /// ```
    pub fn new(code: usize, message: String, errors: Option<ValidationErrors>) -> Self {
        Self { code, message, errors }
    }
}

pub struct FieldValidator {
    errors: ValidationErrors,
}


impl Default for FieldValidator {
    fn default() -> Self {
        Self {
            errors: ValidationErrors::new()
        }
    }
}

impl FieldValidator {
    pub fn validate<T: Validate>(model: &T) -> Self {
        Self {
            errors: model.validate().err().unwrap_or_else(ValidationErrors::new),
        }
    }

    /// Convenience method to trigger early returns with ? operator.
    pub fn check(self) -> Result<(), Json<ApiError>> {
        if self.errors.is_empty() {
            Ok(())
        } else {
            Err(Json(ApiError::new(500,"Invalid fields".to_string(), Some(self.errors))))
        }
    }

    pub fn extract<T>(&mut self, field_name: &'static str, field: Option<T>) -> T
        where
            T: Default,
    {
        field.unwrap_or_else(|| {
            self.errors
                .add(field_name, ValidationError::new("INVALID_FIELD"));
            T::default()
        })
    }
}
