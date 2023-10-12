use actix_web::{body, http::header, web};
use serde::{de, Serialize};

use crate::{database, utils};

use self::types::{ErrorFault, ErrorResponseStatus, ResponseData};

pub mod schoology;
pub mod types;

async fn not_found() -> actix_web::HttpResponse {
    let response: ResponseData<(), ()> = ResponseData::route_error(ErrorResponseStatus::NotFound);

    response.into_response()
}

#[derive(Clone, Copy, PartialEq)]
pub enum Authentication {
    NoAuth,
    UserAuth,
    AdminAuth,
    RootAuth,
}

pub enum ResponseError<T>
where
    T: Serialize,
{
    ClientError(T),
    ServerError(T),
    RequestError(ErrorResponseStatus),
}

pub struct RequestData<T>
where
    T: de::DeserializeOwned,
{
    pub session: Option<orm::sessions::Model>,
    pub user: Option<orm::users::Model>,
    pub auth: Authentication,
    pub data: T,
    pub http_request: actix_web::HttpRequest,
}

/// The GET wrapper (because get has no body)
/// Returns a async function that returns a actix_web::HttpResponse
pub async fn get_util(
    http_request: actix_web::HttpRequest,
    auth: Authentication,
) -> Result<RequestData<()>, ErrorResponseStatus> {
    // Get the session
    let session = http_request.headers().get(header::AUTHORIZATION);

    // Strip the bearer
    let session = match session {
        Some(session) => match session.to_str() {
            Ok(session) => {
                let session = session.strip_prefix("Bearer ");

                match session {
                    Some(session) => Some(session.to_string()),
                    None => None,
                }
            }
            Err(_) => {
                debug!("Failed to convert session to string");
                return Err(ErrorResponseStatus::BadRequest);
            }
        },
        None => None,
    };

    // Get the database client
    let db_client = database::get_db_client();

    // Get the session
    let session = match session {
        Some(session) => match utils::sessions::verify(db_client, &session).await {
            Ok(session) => session,
            Err(_) => {
                debug!("Failed to decode session");
                return Err(ErrorResponseStatus::BadRequest);
            }
        },
        None => None,
    };

    // Check required authentication
    if auth != Authentication::NoAuth {
        return Err(ErrorResponseStatus::Forbidden);
    }

    // Get the user
    let user = match session {
        Some(ref session) => match utils::users::get(db_client, session.user_id).await {
            Ok(user) => user,
            Err(_) => {
                debug!("Failed to get user");
                return Err(ErrorResponseStatus::BadRequest);
            }
        },
        None => None,
    };

    Ok(RequestData {
        session,
        user,
        auth,
        data: (),
        http_request,
    })
}

pub async fn post_util<T>(
    body: web::Bytes,
    http_request: actix_web::HttpRequest,
    auth: Authentication,
) -> Result<RequestData<T>, ErrorResponseStatus>
where
    T: de::DeserializeOwned,
{
    let base = get_util(http_request, auth).await?;

    // Get the request data
    let bytes: &[u8] = &body;

    let json = serde_json::from_slice::<T>(bytes).map_err(|_| {
        debug!("Failed to parse request data");
        ErrorResponseStatus::BadRequest
    })?;

    Ok(RequestData {
        session: base.session,
        user: base.user,
        auth: base.auth,
        data: json,
        http_request: base.http_request,
    })
}

pub async fn encode_response<T, E>(response: Result<T, ResponseError<E>>) -> actix_web::HttpResponse
where
    T: Serialize,
    E: Serialize,
{
    match response {
        Ok(response) => ResponseData::success(response),
        Err(err) => match err {
            ResponseError::ClientError(err) => {
                let response: ResponseData<T, E> = ResponseData::error(ErrorFault::Client, err);

                response
            }
            ResponseError::ServerError(err) => {
                let response: ResponseData<T, E> = ResponseData::error(ErrorFault::Server, err);

                response
            }
            ResponseError::RequestError(status) => {
                let response: ResponseData<T, E> = ResponseData::route_error(status);

                response
            }
        },
    }
    .into_response()
}

#[macro_export]
macro_rules! v1_get {
    ($name: ident, $fn_name: ident, $auth: ident, $res: ty, $err: ty) => {
        pub async fn $name(req: actix_web::HttpRequest) -> actix_web::HttpResponse {
            use crate::v1::{encode_response, get_util, Authentication, ResponseError};
            // Get the request data
            let request_data = match get_util(req, Authentication::$auth).await {
                Ok(request_data) => request_data,
                Err(err) => {
                    let err: Result<$res, ResponseError<$err>> =
                        Err(ResponseError::RequestError(err));
                    return encode_response(err).await;
                }
            };

            // Pass the request data to the handler
            let response: Result<$res, ResponseError<$err>> = $fn_name(request_data).await;

            // Encode the response
            let response = encode_response(response).await;

            response
        }
    };
}

#[macro_export]
macro_rules! v1_post {
    ($name: ident, $fn_name: ident, $auth: ident, $req: ty, $res: ty, $err: ty) => {
        pub async fn $name(
            bytes: actix_web::web::Bytes,
            req: actix_web::HttpRequest,
        ) -> actix_web::HttpResponse {
            use crate::v1::{encode_response, post_util, Authentication, ResponseError};
            // Get the request data
            let request_data = match post_util::<$req>(bytes, req, Authentication::$auth).await {
                Ok(request_data) => request_data,
                Err(err) => {
                    let err: Result<$res, ResponseError<$err>> =
                        Err(ResponseError::RequestError(err));
                    return encode_response(err).await;
                }
            };

            // Pass the request data to the handler
            let response: Result<$res, ResponseError<$err>> = $fn_name(request_data).await;

            // Encode the response
            let response = encode_response(response).await;

            response
        }
    };
}

pub fn create_v1_service() -> actix_web::Scope {
    web::scope("/v1")
        .service(schoology::create_schoology_service())
        .default_service(web::route().to(not_found))
}
