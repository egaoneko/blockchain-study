use std::fmt;
use serde::{Serialize};

/// Error for app
#[derive(Debug)]
pub struct AppError {
    /// code of error
    code: usize,
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
            1000 => "Fail to add",
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
    message: &'static str,
}

impl ApiError {
    /// Returns a error with args
    ///
    /// # Examples
    ///
    /// ```
    /// use blockchain::errors::{ApiError};
    /// let error = ApiError::new(404, "Not found");
    /// ```
    pub fn new(code: usize, message: &'static str) -> Self {
        Self { code, message }
    }
}
