use std::future::Future;
use std::pin::Pin;
use std::str::FromStr;
use sea_orm::{DbErr, EntityTrait, PrimaryKeyTrait, ActiveModelTrait};
use db::repositories::repository::Repository;
use util::filters::FilterParam;

pub trait ToActiveModel<E>
where
    E: EntityTrait,
{
    fn into_active_model(self) -> impl Future<Output = Result<<E as EntityTrait>::ActiveModel, DbErr>> + Send;
}

pub trait Service<'a, E, C, U, R>: Send + Sync
where
    E: EntityTrait,
    C: Send + 'static + ToActiveModel<E>,
    U: Send + 'static + ToActiveModel<E>,
    E::ActiveModel: ActiveModelTrait<Entity = E> + Send,
    E::Model: Send + Sync + sea_orm::IntoActiveModel<E::ActiveModel>,
    E::Column: FromStr + Send + Sync,
    R: Repository<E> + Send + Sync + 'static,
{
    fn create(
        params: C,
    ) -> Pin<Box<dyn Future<Output = Result<E::Model, DbErr>> + Send + 'a>> {
        Box::pin(async move {
            R::create(params.into_active_model().await?).await.map_err(DbErr::from)
        })
    }

    fn update(
        params: U,
    ) -> Pin<Box<dyn Future<Output = Result<E::Model, DbErr>> + Send + 'a>> {
        Box::pin(async move {
            R::update(params.into_active_model().await?).await.map_err(DbErr::from)
        })
    }

    fn delete(
        id: <E::PrimaryKey as PrimaryKeyTrait>::ValueType,
    ) -> Pin<Box<dyn Future<Output = Result<(), DbErr>> + Send + 'a>> {
        Box::pin(async move {
            R::delete(id).await.map_err(DbErr::from)
        })
    }

    fn find_by_id(
        id: <E::PrimaryKey as PrimaryKeyTrait>::ValueType,
    ) -> Pin<Box<dyn Future<Output = Result<Option<E::Model>, DbErr>> + Send + 'a>> {
        Box::pin(async move {
            R::find_by_id(id).await.map_err(DbErr::from)
        })
    }

    fn find_in<V>(
        column: String,
        values: Vec<V>,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<E::Model>, DbErr>> + Send + 'a>>
    where
        V: Into<sea_orm::Value> + Send + Sync + 'static,
    {
        Box::pin(async move {
            let column_enum = E::Column::from_str(&column)
                .map_err(|_| DbErr::Custom(format!("Invalid column name: {}", column)))?;
            
            R::find_in(column_enum, values).await.map_err(DbErr::from)
        })
    }

    fn find_one(
        filter_params: &'a [FilterParam],
        sort_by: Option<String>,
    ) -> Pin<Box<dyn Future<Output = Result<Option<E::Model>, DbErr>> + Send + 'a>> {
        Box::pin(async move {
            R::find_one(filter_params, sort_by).await.map_err(DbErr::from)
        })
    }

    fn find_all(
        filter_params: &'a [FilterParam],
        sort_by: Option<String>,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<E::Model>, DbErr>> + Send + 'a>> {
        Box::pin(async move {
            R::find_all(filter_params, sort_by).await.map_err(DbErr::from)
        })
    }

    fn filter(
        filter_params: &'a [FilterParam],
        page: u64,
        per_page: u64,
        sort_by: Option<String>,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<E::Model>, DbErr>> + Send + 'a>> {
        Box::pin(async move {
            R::filter(filter_params, page, per_page, sort_by)
                .await
                .map_err(DbErr::from)
        })
    }

    fn count(
        filter_params: &'a [FilterParam],
    ) -> Pin<Box<dyn Future<Output = Result<u64, DbErr>> + Send + 'a>> {
        Box::pin(async move {
            R::count(filter_params).await.map_err(DbErr::from)
        })
    }

    fn exists(
        filter_params: &'a [FilterParam],
    ) -> Pin<Box<dyn Future<Output = Result<bool, DbErr>> + Send + 'a>> {
        Box::pin(async move {
            R::exists(filter_params).await.map_err(DbErr::from)
        })
    }
}