use std::future::Future;
use std::pin::Pin;
use sea_orm::{DbErr, EntityTrait, PrimaryKeyTrait, ActiveModelTrait, Select, QueryFilter};
use util::filters::FilterParam;
use crate::get_connection;

pub trait Repository<E>: Send + Sync
where
    E: EntityTrait,
    E::Model: Sync + Send + 'static,
    E::ActiveModel: ActiveModelTrait<Entity = E> + Send,
    E::Model: sea_orm::IntoActiveModel<E::ActiveModel>,
{
    fn apply_filter(query: Select<E>, filter_params: &[FilterParam]) -> Result<Select<E>, DbErr>;

    fn apply_sorting(query: Select<E>, sort_by: Option<String>) -> Select<E>;

    fn create(
        active_model: E::ActiveModel,
    ) -> Pin<Box<dyn Future<Output = Result<E::Model, DbErr>> + Send>> {
        Box::pin(async move {
            active_model.insert(get_connection().await).await.map_err(DbErr::from)
        })
    }

    fn update(
        active_model: E::ActiveModel,
    ) -> Pin<Box<dyn Future<Output = Result<E::Model, DbErr>> + Send>> {
        Box::pin(async move {
            active_model.update(get_connection().await).await.map_err(DbErr::from)
        })
    }

    fn delete(
        id: <E::PrimaryKey as PrimaryKeyTrait>::ValueType,
    ) -> Pin<Box<dyn Future<Output = Result<(), DbErr>> + Send>> {
        Box::pin(async move {
            E::delete_by_id(id).exec(get_connection().await).await.map_err(DbErr::from)?;
            Ok(())
        })
    }

    fn find_by_id(
        id: <E::PrimaryKey as PrimaryKeyTrait>::ValueType,
    ) -> Pin<Box<dyn Future<Output = Result<Option<E::Model>, DbErr>> + Send>> {
        Box::pin(async move {
            E::find_by_id(id).one(get_connection().await).await.map_err(DbErr::from)
        })
    }

    fn find_in<C, V>(
        column: C,
        values: Vec<V>,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<E::Model>, DbErr>> + Send>>
    where
        C: sea_orm::ColumnTrait + 'static,
        V: Into<sea_orm::Value> + Send + Sync + 'static,
    {
        Box::pin(async move {
            E::find()
                .filter(column.is_in(values))
                .all(get_connection().await)
                .await
                .map_err(DbErr::from)
        })
    }

    fn find_one<'a>(
        filter_params: &'a [FilterParam],
        sort_by: Option<String>,
    ) -> Pin<Box<dyn Future<Output = Result<Option<E::Model>, DbErr>> + Send + 'a>> {
        Box::pin(async move {
            let query =  Self::apply_filter(E::find(), &filter_params)?;
            let query = Self::apply_sorting(query, sort_by);
            query.one(get_connection().await)
                .await
                .map_err(DbErr::from)
        })
    }

    fn find_all<'a>(
        filter_params: &'a [FilterParam],
        sort_by: Option<String>,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<E::Model>, DbErr>> + Send + 'a>> {
        Box::pin(async move {
            let query = Self::apply_filter(E::find(), &filter_params)?;
            let query = Self::apply_sorting(query, sort_by);
            query.all(get_connection().await)
                .await
                .map_err(DbErr::from)
        })
    }

    fn filter<'a>(
        filter_params: &'a [FilterParam],
        page: u64,
        per_page: u64,
        sort_by: Option<String>,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<E::Model>, DbErr>> + Send + 'a>> {
        Box::pin(async move {
            let query = Self::apply_filter(E::find(), &filter_params)?;
            let query = Self::apply_sorting(query, sort_by);
            let page_index = page.saturating_sub(1);
            let paginator = <Select<E> as sea_orm::PaginatorTrait<'_, _>>::paginate(query, get_connection().await, per_page);
            paginator
                .fetch_page(page_index)
                .await
                .map_err(DbErr::from)
        })
    }

    fn count<'a>(
        filter_params: &'a [FilterParam],
    ) -> Pin<Box<dyn Future<Output = Result<u64, DbErr>> + Send + 'a>> {
        Box::pin(async move {
            let query = Self::apply_filter(E::find(), &filter_params)?;
            let count = <Select<E> as sea_orm::PaginatorTrait<'_, _>>::count(query, get_connection().await)
                .await
                .map_err(DbErr::from)?;
            Ok(count)
        })
    }

    fn exists<'a>(
        filter_params: &'a [FilterParam],
    ) -> Pin<Box<dyn Future<Output = Result<bool, DbErr>> + Send + 'a>> {
        Box::pin(async move {
            let query = Self::apply_filter(E::find(), &filter_params)?;
            let count = <Select<E> as sea_orm::PaginatorTrait<'_, _>>::count(query, get_connection().await)
                .await
                .map_err(DbErr::from)?;
            Ok(count > 0)
        })
    }
}