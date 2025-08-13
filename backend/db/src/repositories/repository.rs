use std::future::Future;
use std::pin::Pin;
use sea_orm::{DbErr, EntityTrait, PrimaryKeyTrait, ActiveModelTrait, DatabaseConnection, Select, QueryFilter};

pub trait Repository<E, F>: Send + Sync
where
    E: EntityTrait,
    E::Model: Sync + Send + 'static,
    E::ActiveModel: ActiveModelTrait<Entity = E> + Send,
    F: Send + Sync + 'static,
    E::Model: sea_orm::IntoActiveModel<E::ActiveModel>,
{
    fn db(&self) -> &DatabaseConnection;

    fn apply_filter(query: Select<E>, filter: &F) -> Select<E>;

    fn apply_sorting(query: Select<E>, sort_by: Option<String>) -> Select<E>;

    fn create(
        &self,
        active_model: E::ActiveModel,
    ) -> Pin<Box<dyn Future<Output = Result<E::Model, DbErr>> + Send>> {
        let db = self.db().clone();
        Box::pin(async move {
            active_model.insert(&db).await.map_err(DbErr::from)
        })
    }

    fn update(
        &self,
        active_model: E::ActiveModel,
    ) -> Pin<Box<dyn Future<Output = Result<E::Model, DbErr>> + Send>> {
        let db = self.db().clone();
        Box::pin(async move {
            active_model.update(&db).await.map_err(DbErr::from)
        })
    }

    fn delete(
        &self,
        id: <E::PrimaryKey as PrimaryKeyTrait>::ValueType,
    ) -> Pin<Box<dyn Future<Output = Result<(), DbErr>> + Send>> {
        let db = self.db().clone();
        Box::pin(async move {
            E::delete_by_id(id).exec(&db).await.map_err(DbErr::from)?;
            Ok(())
        })
    }

    fn find_by_id(
        &self,
        id: <E::PrimaryKey as PrimaryKeyTrait>::ValueType,
    ) -> Pin<Box<dyn Future<Output = Result<Option<E::Model>, DbErr>> + Send>> {
        let db = self.db().clone();
        Box::pin(async move {
            E::find_by_id(id).one(&db).await.map_err(DbErr::from)
        })
    }

    fn find_in<C, V>(
        &self,
        column: C,
        values: Vec<V>,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<E::Model>, DbErr>> + Send>>
    where
        C: sea_orm::ColumnTrait + 'static,
        V: Into<sea_orm::Value> + Send + Sync + 'static,
    {
        let db = self.db().clone();
        Box::pin(async move {
            E::find()
                .filter(column.is_in(values))
                .all(&db)
                .await
                .map_err(DbErr::from)
        })
    }

    fn find_one(
        &self,
        filter_params: F,
    ) -> Pin<Box<dyn Future<Output = Result<Option<E::Model>, DbErr>> + Send>> {
        let db = self.db().clone();
        Box::pin(async move {
            Self::apply_filter(E::find(), &filter_params)
                .one(&db)
                .await
                .map_err(DbErr::from)
        })
    }

    fn find_all(
        &self,
        filter_params: F,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<E::Model>, DbErr>> + Send>> {
        let db = self.db().clone();
        Box::pin(async move {
            Self::apply_filter(E::find(), &filter_params)
                .all(&db)
                .await
                .map_err(DbErr::from)
        })
    }

    fn filter(
        &self,
        filter_params: F,
        page: u64,
        per_page: u64,
        sort_by: Option<String>,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<E::Model>, DbErr>> + Send>> {
        let db = self.db().clone();
        Box::pin(async move {
            let query = Self::apply_filter(E::find(), &filter_params);
            let query = Self::apply_sorting(query, sort_by);
            let page_index = page.saturating_sub(1);
            let paginator = <Select<E> as sea_orm::PaginatorTrait<'_, _>>::paginate(query, &db, per_page);
            paginator
                .fetch_page(page_index)
                .await
                .map_err(DbErr::from)
        })
    }

    fn count(
        &self,
        filter_params: F,
    ) -> Pin<Box<dyn Future<Output = Result<u64, DbErr>> + Send>> {
        let db = self.db().clone();
        Box::pin(async move {
            let query = Self::apply_filter(E::find(), &filter_params);
            let count = <Select<E> as sea_orm::PaginatorTrait<'_, _>>::count(query, &db)
                .await
                .map_err(DbErr::from)?;
            Ok(count)
        })
    }

    fn exists(
        &self,
        filter_params: F,
    ) -> Pin<Box<dyn Future<Output = Result<bool, DbErr>> + Send>> {
        let db = self.db().clone();
        Box::pin(async move {
            let query = Self::apply_filter(E::find(), &filter_params);
            let count = <Select<E> as sea_orm::PaginatorTrait<'_, _>>::count(query, &db)
                .await
                .map_err(DbErr::from)?;
            Ok(count > 0)
        })
    }
}