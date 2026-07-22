use sea_orm::{DatabaseConnection, DbErr};

use crate::entities::{permission, slave, user};

pub async fn sync_schema(db: &DatabaseConnection) -> Result<(), DbErr> {
    db.get_schema_builder()
        .register(user::Entity)
        .register(slave::Entity)
        .register(permission::Entity)
        .sync(db)
        .await
}
