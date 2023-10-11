use orm::schoology_link;
use sea_orm::{ActiveModelTrait, ActiveValue, DatabaseConnection, EntityTrait};

/// Gets a Schoology link from the database
pub async fn get(
    db_client: &DatabaseConnection,
    id: i32,
) -> Result<Option<schoology_link::Model>, ()> {
    // Query the database
    match schoology_link::Entity::find_by_id(id)
        .one(db_client)
        .await
        .map_err(|err| {
            debug!("Failed to get user: {:?}", err);
        })? {
        Some(user) => Ok(Some(user)),
        None => return Ok(None),
    }
}

/// Creates a user in the database
pub async fn create(
    db_client: &DatabaseConnection,
    user_id: i32,
    schoology_id: i32,
    first_name: Option<String>,
    last_name: Option<String>,
    email: Option<String>,
    picture_url: Option<String>,
    access_token: Option<String>,
    token_secret: Option<String>,
) -> Result<schoology_link::Model, ()> {
    // Create the user
    let user = schoology_link::ActiveModel {
        user_id: ActiveValue::Set(user_id),
        schoology_id: ActiveValue::Set(schoology_id),
        first_name: ActiveValue::Set(first_name),
        last_name: ActiveValue::Set(last_name),
        email: ActiveValue::Set(email),
        picture_url: ActiveValue::Set(picture_url),
        access_token: ActiveValue::Set(access_token),
        token_secret: ActiveValue::Set(token_secret),
    };

    // Insert the user into the database
    let user = user.insert(db_client).await;

    match user {
        Ok(user) => Ok(user),
        Err(err) => {
            warn!("Failed to create user: {:?}", err);
            Err(())
        }
    }
}

/// Updates a user in the database
pub async fn update(
    db_client: &DatabaseConnection,
    schoology_id: i32,
    first_name: Option<String>,
    last_name: Option<String>,
    email: Option<String>,
    picture_url: Option<String>,
    access_token: Option<String>,
    token_secret: Option<String>,
) -> Result<schoology_link::Model, ()> {
    fn convert_to_active_value(value: Option<String>) -> ActiveValue<Option<String>> {
        match value {
            Some(value) => ActiveValue::Set(Some(value)),
            None => ActiveValue::NotSet,
        }
    }

    // Update the user
    let user = schoology_link::ActiveModel {
        user_id: ActiveValue::NotSet,
        schoology_id: ActiveValue::Set(schoology_id),
        first_name: convert_to_active_value(first_name),
        last_name: convert_to_active_value(last_name),
        email: convert_to_active_value(email),
        picture_url: convert_to_active_value(picture_url),
        access_token: convert_to_active_value(access_token),
        token_secret: convert_to_active_value(token_secret),
    };

    user.update(db_client).await.map_err(|err| {
        warn!("Failed to update user: {:?}", err);
    })
}
