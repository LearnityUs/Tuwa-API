use actix_web::web;

mod login;
mod request_token;
mod user;

pub fn create_schoology_service() -> actix_web::Scope {
    web::scope("/schoology")
        .route("/request_token", web::get().to(request_token::get_handler))
        .route("/login", web::post().to(login::post_handler))
        .route("/user", web::get().to(user::get_handler))
}
