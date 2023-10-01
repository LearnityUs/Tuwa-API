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
    Error(ErrorResponseData<Error>),
}

impl<Data, Error> ResponseData<Data, Error>
where
    Data: Serialize,
    Error: Serialize,
{
    pub fn success(data: Data) -> Self {
        ResponseData::Success(SuccessResponseData { data })
    }

    pub fn error(error: Error, message: Option<String>, status: ErrorResponseStatus) -> Self {
        ResponseData::Error(ErrorResponseData {
            error: status,
            message,
            data: error,
        })
    }

    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(&self)
    }

    pub fn into_response(self) -> actix_web::HttpResponse {
        let body = self.to_json().unwrap();

        let mut response = match self {
            ResponseData::Success(_) => actix_web::HttpResponse::Ok(),
            ResponseData::Error(error) => match error.error {
                ErrorResponseStatus::AuthenticationError => actix_web::HttpResponse::Unauthorized(),
                ErrorResponseStatus::PermissionError => actix_web::HttpResponse::Forbidden(),
                ErrorResponseStatus::RequestError => actix_web::HttpResponse::BadRequest(),
                ErrorResponseStatus::ServerError => actix_web::HttpResponse::InternalServerError(),
                ErrorResponseStatus::OtherError => actix_web::HttpResponse::InternalServerError(),
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
#[serde(rename_all = "camelCase")]
pub enum ErrorResponseStatus {
    AuthenticationError,
    PermissionError,
    RequestError,
    ServerError,
    OtherError,
}

#[derive(Serialize)]
pub struct ErrorResponseData<Data>
where
    Data: Serialize,
{
    pub error: ErrorResponseStatus,
    pub message: Option<String>,
    pub data: Data,
}
