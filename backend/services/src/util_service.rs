use db::test_utils::clean_db;

pub struct UtilService {}

impl UtilService {
    pub async fn clean_db() -> Result<(), db::DbErr> {
        clean_db().await
    }
}