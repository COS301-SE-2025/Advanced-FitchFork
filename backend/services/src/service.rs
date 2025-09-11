use std::future::Future;
use std::pin::Pin;
use std::str::FromStr;
use sea_orm::{DbErr, EntityTrait, ColumnTrait, PrimaryKeyTrait, ActiveModelTrait};
use db::repository::Repository;
use util::filters::FilterParam;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] DbErr),

    #[error("Unexpected database error")]
    DatabaseUnknown,
}

pub trait ToActiveModel<E>
where
    E: EntityTrait,
{
    fn into_active_model(self) -> impl Future<Output = Result<<E as EntityTrait>::ActiveModel, AppError>> + Send;
}

pub trait Service<'a, E, C, CreateParams, UpdateParams>: Send + Sync
where
    E: EntityTrait,
    C: ColumnTrait + FromStr + 'static,
    C::Err: std::fmt::Display,
    CreateParams: Send + 'static + ToActiveModel<E>,
    UpdateParams: Send + 'static + ToActiveModel<E>,
    E::ActiveModel: ActiveModelTrait<Entity = E> + Send,
    E::Model: Send + Sync + sea_orm::IntoActiveModel<E::ActiveModel>,
{
    fn create(
        params: CreateParams,
    ) -> Pin<Box<dyn Future<Output = Result<E::Model, AppError>> + Send + 'a>> {
        Box::pin(async move {
            let active_model = params.into_active_model().await?;
            Repository::<E, C>::create(active_model).await.map_err(AppError::from)
        })
    }

    fn update(
        params: UpdateParams,
    ) -> Pin<Box<dyn Future<Output = Result<E::Model, AppError>> + Send + 'a>> {
        Box::pin(async move {
            let active_model = params.into_active_model().await?;
            Repository::<E, C>::update(active_model).await.map_err(AppError::from)
        })
    }

    fn delete(
        id: <E::PrimaryKey as PrimaryKeyTrait>::ValueType,
    ) -> Pin<Box<dyn Future<Output = Result<(), AppError>> + Send + 'a>> {
        Box::pin(async move {
            Repository::<E, C>::delete(id).await.map_err(AppError::from)
        })
    }

    fn find_by_id(
        id: <E::PrimaryKey as PrimaryKeyTrait>::ValueType,
    ) -> Pin<Box<dyn Future<Output = Result<Option<E::Model>, AppError>> + Send + 'a>> {
        Box::pin(async move {
            Repository::<E, C>::find_by_id(id).await.map_err(AppError::from)
        })
    }

    fn find_in<V>(
        column: String,
        values: Vec<V>,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<E::Model>, AppError>> + Send + 'a>>
    where
        V: Into<sea_orm::Value> + Send + Sync + 'static,
    {
        Box::pin(async move {
            let column_enum = C::from_str(&column)
                .map_err(|e| DbErr::Custom(format!("Invalid column name '{}': {}", column, e)))?;
            
            Repository::<E, C>::find_in(column_enum, values).await.map_err(AppError::from)
        })
    }

    fn find_one(
        filter_params: &'a [FilterParam],
        sort_by: Option<String>,
    ) -> Pin<Box<dyn Future<Output = Result<Option<E::Model>, AppError>> + Send + 'a>> {
        Box::pin(async move {
            Repository::<E, C>::find_one(filter_params, sort_by).await.map_err(AppError::from)
        })
    }

    fn find_all(
        filter_params: &'a [FilterParam],
        sort_by: Option<String>,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<E::Model>, AppError>> + Send + 'a>> {
        Box::pin(async move {
            Repository::<E, C>::find_all(filter_params, sort_by).await.map_err(AppError::from)
        })
    }

    fn filter(
        filter_params: &'a [FilterParam],
        page: u64,
        per_page: u64,
        sort_by: Option<String>,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<E::Model>, AppError>> + Send + 'a>> {
        Box::pin(async move {
            Repository::<E, C>::filter(filter_params, page, per_page, sort_by)
                .await
                .map_err(AppError::from)
        })
    }

    fn count(
        filter_params: &'a [FilterParam],
    ) -> Pin<Box<dyn Future<Output = Result<u64, AppError>> + Send + 'a>> {
        Box::pin(async move {
            Repository::<E, C>::count(filter_params).await.map_err(AppError::from)
        })
    }

    fn exists(
        filter_params: &'a [FilterParam],
    ) -> Pin<Box<dyn Future<Output = Result<bool, AppError>> + Send + 'a>> {
        Box::pin(async move {
            Repository::<E, C>::exists(filter_params).await.map_err(AppError::from)
        })
    }
}