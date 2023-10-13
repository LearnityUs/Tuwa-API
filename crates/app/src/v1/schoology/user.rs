//! /docs/api/v1/schoology/user

use serde::Serialize;

use crate::{
    database::get_db_client,
    schoology::get_schoology_client,
    utils,
    v1::{types::ErrorResponseStatus, RequestData, ResponseError},
    v1_get,
};

#[derive(Serialize)]
struct Response {
    first_name: String,
    last_name: String,
    picture_url: String,
}

#[derive(Debug, Serialize)]
enum Error {
    SchoologyNotLinked,
    DatabaseError,
}

async fn get(data: RequestData<()>) -> Result<Response, ResponseError<Error>> {
    let id = match data.user {
        Some(user) => user.id,
        None => {
            return Err(ResponseError::RequestError(
                ErrorResponseStatus::Unauthorized,
            ))
        }
    };

    let db_client = get_db_client();

    // Fetch the user from the database
    let user = utils::schoology_link::get_by_user_id(db_client, id)
        .await
        .map_err(|_| ResponseError::ServerError(Error::DatabaseError))?
        .ok_or(ResponseError::ServerError(Error::SchoologyNotLinked))?;

    // Fetch the user from Schoology
    let schoology_client = get_schoology_client();

    let user = schoology::users::get_schoology_user(
        &schoology_client,
        &schoology::SchoologyTokenPair {
            access_token: user.access_token
                .ok_or(ResponseError::ServerError(Error::SchoologyNotLinked))?,
            token_secret: user.token_secret
                .ok_or(ResponseError::ServerError(Error::SchoologyNotLinked))?,
        },
        user.schoology_id as usize
    )
    .await
    .map_err(|_| ResponseError::ServerError(Error::SchoologyNotLinked))?;

    Ok(Response {
        first_name: user.name_first,
        last_name: user.name_last,
        picture_url: user.picture_url,
    })
}

v1_get!(get_handler, get, UserAuth, Response, Error);
