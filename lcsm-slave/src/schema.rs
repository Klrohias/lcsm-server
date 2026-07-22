use sea_orm::{DatabaseConnection, DbErr};

use crate::entities::instance;

pub async fn sync_schema(db: &DatabaseConnection) -> Result<(), DbErr> {
    db.get_schema_builder()
        .register(instance::Entity)
        .sync(db)
        .await
}
