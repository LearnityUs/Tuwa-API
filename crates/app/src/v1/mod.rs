use actix_web::{get, web};
use schoology::SchoologyRequest;

use crate::schoology::get_schoology_client;

use self::types::{ErrorResponseStatus, ResponseData};

pub mod types;

async fn not_found() -> actix_web::HttpResponse {
    let response: ResponseData<(), ()> = ResponseData::error(
        (),
        Some("Rawr 🦖! This page was not found!".to_string()),
        ErrorResponseStatus::NotFound,
    );

    response.into_response()
}

#[derive(serde::Serialize)]
struct TestStruct {
    test: String,
}

#[get("/test")]
pub async fn test() -> actix_web::HttpResponse {
    let schoology_client = get_schoology_client();

    let data = schoology_client
        .get("/v1/oauth/request_token", SchoologyRequest::empty())
        .await
        .map(|response| response.text());

    let response: ResponseData<TestStruct, ()> = match data {
        Ok(data) => match data.await {
            Ok(data) => ResponseData::success(TestStruct { test: data }),
            Err(_) => ResponseData::error((), None, ErrorResponseStatus::InternalServerError),
        },
        Err(_) => ResponseData::error((), None, ErrorResponseStatus::InternalServerError),
    };

    response.into_response()
}

pub fn create_v1_service() -> actix_web::Scope {
    web::scope("/v1")
        .service(test)
        .default_service(web::route().to(not_found))
}
