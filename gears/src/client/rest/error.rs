use rocket::serde::json::Json;
use rocket::{
    http::Status,
    response::{self, status::Custom, Responder},
    Request,
};
use serde::Serialize;

pub struct Error {
    pub status: Status,
    pub description: Option<String>, // use default description if set to None, see https://github.com/SergioBenitez/Rocket/blob/91f6288ea4aeb3d5a502b2f18b2b9677a85463ea/core/lib/src/catcher/catcher.rs#L369-L416
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
            status: Status::BadRequest,
            description: Some(description),
        }
    }

    pub fn gateway_timeout() -> Error {
        Error {
            status: Status::GatewayTimeout,
            description: None,
        }
    }

    pub fn bad_gateway() -> Error {
        Error {
            status: Status::BadGateway,
            description: None,
        }
    }

    fn to_serializable(self) -> PrintError {
        PrintError {
            error: PrintErrorCore {
                code: self.status.code,
                reason: self.status.reason_lossy().to_string(),
                description: self.description.unwrap_or_default(),
            },
        }
    }
}

impl<'r, 'o> Responder<'r, 'o> for Error
where
    'o: 'r,
{
    fn respond_to(self, request: &'r Request<'_>) -> response::Result<'o> {
        if self.description.is_none() {
            // trigger use of default description
            return Err(self.status);
        }
        Custom(self.status, Json(self.to_serializable())).respond_to(request)
    }
}
