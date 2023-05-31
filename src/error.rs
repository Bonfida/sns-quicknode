use std::fmt::Display;

use actix_web::{
    http::{
        header::{HeaderValue, CONTENT_TYPE},
        StatusCode,
    },
    ResponseError,
};

#[derive(Debug)]
pub enum ErrorType {
    Generic,
    InvalidAuthentication,
    DbError,
    ProvisioningRecordNotFound,
    UnsupportedEndpoint,
    UnsupportedMethod,
    MalformedRequest,
    InvalidParameters,
    InvalidDomain,
    DomainNotFound
}

#[derive(Debug)]
pub struct Error {
    pub ty: ErrorType,
    pub trace: Vec<String>,
    pub info: Vec<String>,
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self.ty {
            ErrorType::InvalidAuthentication => "Invalid Authentication",
            ErrorType::ProvisioningRecordNotFound => "User has not been provisioned",
            ErrorType::UnsupportedEndpoint => "Unsupported endpoint",
            ErrorType::UnsupportedMethod => "Unsupported method",
            ErrorType::MalformedRequest => "Malformed Request",
            ErrorType::InvalidParameters => "Invalid Parameters",
            ErrorType::InvalidDomain => "Invalid Domain",
            _ => "Internal error",
        };
        f.write_str(s)
    }
}

impl ResponseError for Error {
    fn status_code(&self) -> actix_web::http::StatusCode {
        match self.ty {
            ErrorType::InvalidAuthentication | ErrorType::ProvisioningRecordNotFound => {
                StatusCode::UNAUTHORIZED
            }
            ErrorType::UnsupportedEndpoint => StatusCode::NOT_FOUND,
            ErrorType::MalformedRequest
            | ErrorType::InvalidParameters
            | ErrorType::InvalidDomain => StatusCode::BAD_REQUEST,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> actix_web::HttpResponse<actix_web::body::BoxBody> {
        let mut res = actix_web::HttpResponse::new(self.status_code())
            .set_body(actix_web::body::BoxBody::new(format!("{self}")));
        println!("Error : {self:?}");
        res.headers_mut()
            .insert(CONTENT_TYPE, HeaderValue::from_static("text/plain"));

        res
    }
}

#[macro_export]
macro_rules! trace {
    () => {
        $crate::error::Error {
            ty: $crate::Error::Generic,
            trace: vec![format!("{}:{}", file!(), line!())],
            info: vec![],
        }
    };
    ($ty:expr) => {
        $crate::error::Error {
            ty: $ty,
            trace: vec![format!("{}:{}", file!(), line!())],
            info: vec![],
        }
    };
    ($ty:expr, $expression:expr) => {
        $crate::error::Error {
            ty: $ty,
            trace: vec![format!("{}:{}", file!(), line!())],
            info: vec![format!("{:?}", $expression)],
        }
    };
}

impl Error {
    pub fn append_trace(mut self, trace: String) -> Self {
        self.trace.push(trace);
        self
    }

    pub fn append_info(mut self, info: String) -> Self {
        self.info.push(info);
        self
    }
}

#[macro_export]
macro_rules! append_trace {
    ($expression:expr) => {
        $expression.append_trace(format!("{}:{}", file!(), line!()))
    };
    ($expression:expr, $custom:expr) => {{
        $expression
            .append_trace(format!("{}:{}", file!(), line!()))
            .append_info($custom)
    }};
}
