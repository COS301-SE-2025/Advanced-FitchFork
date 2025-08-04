#[cfg(test)]
mod tests {
    use crate::helpers::make_test_app;

    struct TestData {}

    async fn setup_test_data(db: &sea_orm::DatabaseConnection) -> TestData {
        TestData {}
    }
    
    #[tokio::test]
    async fn create_ticket_test() {
        let (app, app_state) = make_test_app().await;
        let data = setup_test_data(app_state.db()).await;
    }
}
