use std::fmt;

#[derive(Debug)]
pub struct AppError {
    code: usize,
}

impl AppError {
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
