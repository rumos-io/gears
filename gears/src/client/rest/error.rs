use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Serialize;

#[derive(Debug)]
pub struct Error {
    status: StatusCode,
    description: String, // set to (unless overridden): https://github.com/SergioBenitez/Rocket/blob/91f6288ea4aeb3d5a502b2f18b2b9677a85463ea/core/lib/src/catcher/catcher.rs#L369-L416
}

#[derive(Serialize)]
struct PrintErrorCore {
    pub code: u16,
    pub reason: String,
    pub description: String,
}

#[derive(Serialize)]
struct PrintError {
    error: PrintErrorCore,
}

impl Error {
    pub fn bad_request(description: String) -> Error {
        Error {
            status: StatusCode::BAD_REQUEST,
            description,
        }
    }

    pub fn gateway_timeout() -> Error {
        Error {
            status: StatusCode::GATEWAY_TIMEOUT,
            description: "The server did not receive a timely response from an upstream server."
                .into(),
        }
    }

    pub fn bad_gateway() -> Error {
        Error {
            status: StatusCode::BAD_GATEWAY,
            description: "The server received an invalid response from an upstream server".into(),
        }
    }

    pub fn bad_gateway_with_msg(description: String) -> Error {
        Error {
            status: StatusCode::BAD_GATEWAY,
            description,
        }
    }

    fn to_serializable(self) -> PrintError {
        PrintError {
            error: PrintErrorCore {
                code: self.status.as_u16(),
                reason: self
                    .status
                    .canonical_reason()
                    .unwrap_or_default()
                    .to_string(),
                description: self.description,
            },
        }
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        (self.status, Json(self.to_serializable())).into_response()
    }
}
