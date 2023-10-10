use actix_web::web;

use self::types::{ErrorResponseStatus, ResponseData};

pub mod schoology;
pub mod types;

async fn not_found() -> actix_web::HttpResponse {
    let response: ResponseData<(), ()> = ResponseData::error(
        (),
        Some("Rawr 🦖! This page was not found!".to_string()),
        ErrorResponseStatus::NotFound,
    );

    response.into_response()
}

pub fn create_v1_service() -> actix_web::Scope {
    web::scope("/v1")
        .service(schoology::create_schoology_service())
        .default_service(web::route().to(not_found))
}
