use orm::users;
use sea_orm::{ActiveModelTrait, ActiveValue, DatabaseConnection, EntityTrait};

pub async fn get(db_client: &DatabaseConnection, id: i32) -> Result<Option<users::Model>, ()> {
    // Query the database
    match users::Entity::find_by_id(id)
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
pub async fn create(db_client: &DatabaseConnection) -> Result<users::Model, ()> {
    // Create the user
    let user = users::ActiveModel {
        id: ActiveValue::NotSet,
        is_admin: ActiveValue::Set(false),
        is_root: ActiveValue::Set(false),
        created_at: ActiveValue::Set(chrono::Utc::now().naive_utc()),
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
