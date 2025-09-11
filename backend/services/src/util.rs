use db::test_utils::clean_db;
use sea_orm::DbErr;

pub struct UtilService {}

impl UtilService {
    pub async fn clean_db() -> Result<(), DbErr> {
        clean_db().await
    }
}