use actix_web::web;

pub mod types;

pub fn create_v1_service() -> actix_web::Scope {
    web::scope("/v1")
}
