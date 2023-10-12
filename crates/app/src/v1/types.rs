use actix_web::http;
use serde::Serialize;

#[derive(Serialize)]
#[serde(tag = "type")]
pub enum ResponseData<Data, Error>
where
    Data: Serialize,
    Error: Serialize,
{
    Success(SuccessResponseData<Data>),
    RequestError(ErrorResponseStatusData),
    RouteError(ErrorResponseData<Error>),
}

impl<Data, Error> ResponseData<Data, Error>
where
    Data: Serialize,
    Error: Serialize,
{
    /// If the request was successful, return a success response.
    pub fn success(data: Data) -> Self {
        ResponseData::Success(SuccessResponseData { data })
    }

    /// If the request was unsuccessful, return an error response.
    pub fn error(fault: ErrorFault, data: Error) -> Self {
        ResponseData::RouteError(ErrorResponseData { data, fault })
    }

    /// Route error
    pub fn route_error(data: ErrorResponseStatus) -> Self {
        ResponseData::RequestError(ErrorResponseStatusData { status: data })
    }

    /// Convert the response to a JSON string.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(&self)
    }

    /// Convert the response to an actix_web::HttpResponse.
    pub fn into_response(self) -> actix_web::HttpResponse {
        let body = self.to_json().unwrap();

        let mut response = match self {
            ResponseData::Success(_) => actix_web::HttpResponse::Ok(),
            ResponseData::RequestError(error) => match error.status {
                ErrorResponseStatus::NotFound => actix_web::HttpResponse::NotFound(),
                ErrorResponseStatus::Unauthorized => actix_web::HttpResponse::Unauthorized(),
                ErrorResponseStatus::Forbidden => actix_web::HttpResponse::Forbidden(),
                ErrorResponseStatus::BadRequest => actix_web::HttpResponse::BadRequest(),
                ErrorResponseStatus::InternalServerError => {
                    actix_web::HttpResponse::InternalServerError()
                }
            },
            ResponseData::RouteError(error) => match error.fault {
                ErrorFault::Client => actix_web::HttpResponse::BadRequest(),
                ErrorFault::Server => actix_web::HttpResponse::InternalServerError(),
            },
        };

        response
            .append_header((http::header::CONTENT_TYPE, "application/json"))
            .body(body)
    }
}

#[derive(Serialize)]
pub struct SuccessResponseData<Data>
where
    Data: Serialize,
{
    pub data: Data,
}

#[derive(Serialize)]
pub enum ErrorResponseStatus {
    /// Self-explanatory; the requested resource was not found. (Used only when a path parameter is used).
    NotFound,
    /// The user is not authenticated.
    Unauthorized,
    /// The user is authenticated, but does not have the required credentials.
    Forbidden,
    /// The request was malformed.
    BadRequest,
    /// The server encountered an internal error.
    InternalServerError,
}

#[derive(Serialize)]
pub struct ErrorResponseStatusData {
    pub status: ErrorResponseStatus,
}

pub enum ErrorFault {
    Client,
    Server,
}

#[derive(Serialize)]
pub struct ErrorResponseData<Data>
where
    Data: Serialize,
{
    #[serde(skip)]
    pub fault: ErrorFault,
    pub data: Data,
}
