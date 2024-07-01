use sea_orm::{Database, DatabaseConnection, DbErr};

async fn main() -> Result<DatabaseConnection, DbErr> {
    let db = Database::connect("sqlite:memory:").await?;

}