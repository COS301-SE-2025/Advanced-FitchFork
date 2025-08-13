use std::future::Future;
use std::pin::Pin;
use sea_orm::{DbErr, EntityTrait, PrimaryKeyTrait, ActiveModelTrait};
use db::repositories::repository::Repository;

pub trait ToActiveModel<E>
where
    E: EntityTrait,
{
    fn into_active_model(self) -> Result<<E as EntityTrait>::ActiveModel, DbErr>;
}

pub trait Service<'a, E, C, U, F, R>: Send + Sync
where
    E: EntityTrait,
    C: Send + 'static + ToActiveModel<E>,
    U: Send + 'static + ToActiveModel<E>,
    F: Send + Sync + 'static,
    R: Repository<E, F> + 'a,
    E::ActiveModel: ActiveModelTrait<Entity = E> + Send,
    E::Model: Send + Sync + sea_orm::IntoActiveModel<E::ActiveModel>,
{
    fn repository(&'a self) -> &'a R;

    fn create(
        &'a self,
        params: C,
    ) -> Pin<Box<dyn Future<Output = Result<E::Model, DbErr>> + Send + 'a>> {
        let repo = self.repository();
        Box::pin(async move {
            repo.create(params.into_active_model()?).await.map_err(DbErr::from)
        })
    }

    fn update(
        &'a self,
        params: U,
    ) -> Pin<Box<dyn Future<Output = Result<E::Model, DbErr>> + Send + 'a>> {
        let repo = self.repository();
        Box::pin(async move {
            repo.update(params.into_active_model()?).await.map_err(DbErr::from)
        })
    }

    fn delete(
        &'a self,
        id: <E::PrimaryKey as PrimaryKeyTrait>::ValueType,
    ) -> Pin<Box<dyn Future<Output = Result<(), DbErr>> + Send + 'a>> {
        let repo = self.repository();
        Box::pin(async move {
            repo.delete(id).await.map_err(DbErr::from)
        })
    }

    fn find_by_id(
        &'a self,
        id: <E::PrimaryKey as PrimaryKeyTrait>::ValueType,
    ) -> Pin<Box<dyn Future<Output = Result<Option<E::Model>, DbErr>> + Send + 'a>> {
        let repo = self.repository();
        Box::pin(async move {
            repo.find_by_id(id).await.map_err(DbErr::from)
        })
    }

    fn find_in<Col, Val>(
        &'a self,
        column: Col,
        values: Vec<Val>,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<E::Model>, DbErr>> + Send + 'a>>
    where
        Col: sea_orm::ColumnTrait + 'static,
        Val: Into<sea_orm::Value> + Send + Sync + 'static,
    {
        let repo = self.repository();
        Box::pin(async move {
            repo.find_in(column, values).await.map_err(DbErr::from)
        })
    }

    fn find_one(
        &'a self,
        filter_params: F,
    ) -> Pin<Box<dyn Future<Output = Result<Option<E::Model>, DbErr>> + Send + 'a>> {
        let repo = self.repository();
        Box::pin(async move {
            repo.find_one(filter_params).await.map_err(DbErr::from)
        })
    }

    /// Find all entities matching the filter
    fn find_all(
        &'a self,
        filter_params: F,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<E::Model>, DbErr>> + Send + 'a>> {
        let repo = self.repository();
        Box::pin(async move {
            repo.find_all(filter_params).await.map_err(DbErr::from)
        })
    }

    /// Find entities with pagination and sorting
    fn filter(
        &'a self,
        filter_params: F,
        page: u64,
        per_page: u64,
        sort_by: Option<String>,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<E::Model>, DbErr>> + Send + 'a>> {
        let repo = self.repository();
        Box::pin(async move {
            repo.filter(filter_params, page, per_page, sort_by)
                .await
                .map_err(DbErr::from)
        })
    }

    /// Count entities matching the filter
    fn count(
        &'a self,
        filter_params: F,
    ) -> Pin<Box<dyn Future<Output = Result<u64, DbErr>> + Send + 'a>> {
        let repo = self.repository();
        Box::pin(async move {
            repo.count(filter_params).await.map_err(DbErr::from)
        })
    }

    /// Check if any entities exist matching the filter
    fn exists(
        &'a self,
        filter_params: F,
    ) -> Pin<Box<dyn Future<Output = Result<bool, DbErr>> + Send + 'a>> {
        let repo = self.repository();
        Box::pin(async move {
            repo.exists(filter_params).await.map_err(DbErr::from)
        })
    }
}