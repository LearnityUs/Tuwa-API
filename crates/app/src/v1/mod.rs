use actix_web::web;

use self::types::{ErrorResponseData, ErrorResponseStatus, ResponseData};

pub mod types;

async fn not_found() -> actix_web::HttpResponse {
    ResponseData::<(), _>::Error(ErrorResponseData::<()> {
        error: ErrorResponseStatus::NotFoundError,
        message: Some("Not found".to_string()),
        data: (),
    })
    .into_response()
}

pub fn create_v1_service() -> actix_web::Scope {
    web::scope("/v1").default_service(web::route().to(not_found))
}
