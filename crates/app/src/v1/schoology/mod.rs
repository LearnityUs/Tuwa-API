use actix_web::web;

mod login;
mod request_token;

pub fn create_schoology_service() -> actix_web::Scope {
    web::scope("/schoology")
        .service(request_token::get_request_token)
        .service(login::post_login)
}
