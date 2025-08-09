use std::{fmt::Display, sync::Arc};

use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, DatabaseConnection, DbErr, EntityTrait,
    IntoActiveModel, QueryFilter,
};

use crate::entities::user;

fn extract_user(input: Result<Option<user::Model>, DbErr>) -> Result<user::Model, DbErr> {
    match input {
        Err(e) => Err(e),
        Ok(v) => match v {
            None => Err(DbErr::RecordNotFound(String::new())),
            Some(v) => Ok(v),
        },
    }
}

pub type UserServiceRef = Arc<UserService>;

pub struct UserService {
    database_connection: DatabaseConnection,
}

impl UserService {
    pub fn new(database_connection: DatabaseConnection) -> Self {
        Self {
            database_connection,
        }
    }

    pub async fn verify_user_creds(
        &self,
        username: impl AsRef<str>,
        password: impl AsRef<str>,
    ) -> Result<user::Model, CredsVerificationError> {
        // try find user by username
        let user = match self.find_user_by_username(username.as_ref()).await {
            Err(DbErr::RecordNotFound(_)) => match self.find_user_by_email(username.as_ref()).await
            {
                Ok(v) => v,
                Err(e) => return Err(e.into()),
            },
            Err(other) => return Err(other.into()),
            Ok(v) => v,
        };

        if !bcrypt::verify(password.as_ref(), &user.password_hash)? {
            return Err(CredsVerificationError::VerifyError);
        }

        Ok(user)
    }

    pub async fn set_user_banned(&self, user_id: i32, banned: bool) -> Result<(), DbErr> {
        let mut user = self.find_user_by_id(user_id).await?.into_active_model();
        user.banned = ActiveValue::Set(banned);
        user.update(&self.database_connection).await?;

        Ok(())
    }

    pub async fn find_user_by_id(&self, user_id: i32) -> Result<user::Model, DbErr> {
        let result = user::Entity::find_by_id(user_id)
            .one(&self.database_connection)
            .await;

        extract_user(result)
    }

    pub async fn find_user_by_username(
        &self,
        username: impl AsRef<str>,
    ) -> Result<user::Model, DbErr> {
        let result = user::Entity::find()
            .filter(user::Column::Name.eq(username.as_ref()))
            .one(&self.database_connection)
            .await;

        extract_user(result)
    }

    pub async fn find_user_by_email(
        &self,
        username: impl AsRef<str>,
    ) -> Result<user::Model, DbErr> {
        let result = user::Entity::find()
            .filter(user::Column::Email.eq(username.as_ref()))
            .one(&self.database_connection)
            .await;

        extract_user(result)
    }
}

#[derive(Debug)]
pub enum CredsVerificationError {
    DbErr(DbErr),
    BcryptError(bcrypt::BcryptError),
    VerifyError,
}

impl Display for CredsVerificationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BcryptError(e) => e.fmt(f),
            Self::DbErr(e) => e.fmt(f),
            Self::VerifyError => write!(f, "Failed to verify given creds"),
        }
    }
}

impl std::error::Error for CredsVerificationError {}

impl From<DbErr> for CredsVerificationError {
    fn from(value: DbErr) -> Self {
        Self::DbErr(value)
    }
}

impl From<bcrypt::BcryptError> for CredsVerificationError {
    fn from(value: bcrypt::BcryptError) -> Self {
        Self::BcryptError(value)
    }
}
