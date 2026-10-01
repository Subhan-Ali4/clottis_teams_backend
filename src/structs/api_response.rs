use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct ApiResponse<T>
where
    T: Serialize,
{
    pub success: bool,
    pub data: T,
}

impl<T> ApiResponse<T>
where
    T: Serialize,
{
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub success: bool,
    pub error: ErrorBody,
}

#[derive(Debug, Serialize)]
pub struct ErrorBody {
    pub code: &'static str,
    pub message: String,
}

impl ErrorResponse {
    pub fn new(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            success: false,
            error: ErrorBody {
                code,
                message: message.into(),
            },
        }
    }
}
