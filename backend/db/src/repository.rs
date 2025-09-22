use crate::filter_utils::{FilterUtils, QueryUtils, SortUtils};
use crate::get_connection;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DbErr, EntityTrait, IntoActiveModel, PaginatorTrait,
    PrimaryKeyTrait, QueryFilter, Select,
};
use std::marker::PhantomData;
use std::str::FromStr;
use util::filters::{FilterParam, QueryParam};

/// Generic repository that works with any SeaORM entity
pub struct Repository<E, C>
where
    E: EntityTrait,
    E::Model: Sync + Send + 'static,
    E::ActiveModel: ActiveModelTrait<Entity = E> + Send,
    E::Model: IntoActiveModel<E::ActiveModel>,
    C: ColumnTrait + FromStr + 'static,
    C::Err: std::fmt::Display,
{
    _phantom: PhantomData<(E, C)>,
}

impl<E, C> Repository<E, C>
where
    E: EntityTrait,
    E::Model: Sync + Send + 'static,
    E::ActiveModel: ActiveModelTrait<Entity = E> + Send,
    E::Model: IntoActiveModel<E::ActiveModel>,
    C: ColumnTrait + FromStr + 'static,
    C::Err: std::fmt::Display,
{
    pub fn new() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }

    pub fn apply_filter(
        query: Select<E>,
        filter_params: &[FilterParam],
    ) -> Result<Select<E>, DbErr> {
        let condition = FilterUtils::apply_all_filters(filter_params, |column_name| {
            C::from_str(column_name)
                .map_err(|e| DbErr::Custom(format!("Invalid column name '{}': {}", column_name, e)))
        })?;

        Ok(query.filter(condition))
    }

    pub fn apply_query(query: Select<E>, query_params: &[QueryParam]) -> Result<Select<E>, DbErr> {
        let condition = QueryUtils::apply_all_queries(query_params, |column_name| {
            C::from_str(column_name)
                .map_err(|e| DbErr::Custom(format!("Invalid column name '{}': {}", column_name, e)))
        })?;

        Ok(query.filter(condition))
    }

    pub fn apply_sorting(query: Select<E>, sort_by: Option<String>) -> Select<E> {
        SortUtils::apply_sorting(query.clone(), sort_by, |column_name| {
            C::from_str(column_name)
                .map_err(|e| DbErr::Custom(format!("Invalid column name '{}': {}", column_name, e)))
        })
        .unwrap_or(query)
    }

    pub async fn create(active_model: E::ActiveModel) -> Result<E::Model, DbErr> {
        active_model
            .insert(get_connection().await)
            .await
            .map_err(DbErr::from)
    }

    pub async fn update(active_model: E::ActiveModel) -> Result<E::Model, DbErr> {
        active_model
            .update(get_connection().await)
            .await
            .map_err(DbErr::from)
    }

    pub async fn delete_by_id(
        id: <E::PrimaryKey as PrimaryKeyTrait>::ValueType,
    ) -> Result<(), DbErr> {
        E::delete_by_id(id)
            .exec(get_connection().await)
            .await
            .map_err(DbErr::from)?;
        Ok(())
    }

    pub async fn delete(
        filter_params: &[FilterParam],
        query_params: &[QueryParam],
    ) -> Result<u64, DbErr> {
        if filter_params.is_empty() && query_params.is_empty() {
            return Err(DbErr::Custom(
                "Refusing to delete without filters or queries. Provide at least one filter or query param.".to_string(),
            ));
        }

        let mut query = E::delete_many();

        if !filter_params.is_empty() {
            let condition = FilterUtils::apply_all_filters(filter_params, |column_name| {
                C::from_str(column_name).map_err(|e| {
                    DbErr::Custom(format!("Invalid column name '{}': {}", column_name, e))
                })
            })?;
            query = query.filter(condition);
        }

        if !query_params.is_empty() {
            let condition = QueryUtils::apply_all_queries(query_params, |column_name| {
                C::from_str(column_name).map_err(|e| {
                    DbErr::Custom(format!("Invalid column name '{}': {}", column_name, e))
                })
            })?;
            query = query.filter(condition);
        }

        let res = query
            .exec(get_connection().await)
            .await
            .map_err(DbErr::from)?;
        Ok(res.rows_affected)
    }

    pub async fn find_by_id(
        id: <E::PrimaryKey as PrimaryKeyTrait>::ValueType,
    ) -> Result<Option<E::Model>, DbErr> {
        E::find_by_id(id)
            .one(get_connection().await)
            .await
            .map_err(DbErr::from)
    }

    pub async fn find_one(
        filter_params: &[FilterParam],
        query_params: &[QueryParam],
        sort_by: Option<String>,
    ) -> Result<Option<E::Model>, DbErr> {
        let query = Self::apply_filter(E::find(), filter_params)?;
        let query = Self::apply_query(query, query_params)?;
        let query = Self::apply_sorting(query, sort_by);
        query.one(get_connection().await).await.map_err(DbErr::from)
    }

    pub async fn find_all(
        filter_params: &[FilterParam],
        query_params: &[QueryParam],
        sort_by: Option<String>,
    ) -> Result<Vec<E::Model>, DbErr> {
        let query = Self::apply_filter(E::find(), filter_params)?;
        let query = Self::apply_query(query, query_params)?;
        let query = Self::apply_sorting(query, sort_by);
        query.all(get_connection().await).await.map_err(DbErr::from)
    }

    pub async fn filter(
        filter_params: &[FilterParam],
        query_params: &[QueryParam],
        page: u64,
        per_page: u64,
        sort_by: Option<String>,
    ) -> Result<(Vec<E::Model>, u64), DbErr> {
        let query = Self::apply_filter(E::find(), filter_params)?;
        let query = Self::apply_query(query, query_params)?;
        let query = Self::apply_sorting(query, sort_by);

        let paginator = query.paginate(get_connection().await, per_page);
        let total = paginator.num_items().await?;
        let items = paginator
            .fetch_page(page.saturating_sub(1))
            .await
            .map_err(DbErr::from)?;

        Ok((items, total))
    }

    pub async fn count(
        filter_params: &[FilterParam],
        query_params: &[QueryParam],
    ) -> Result<u64, DbErr> {
        let query = Self::apply_filter(E::find(), filter_params)?;
        let query = Self::apply_query(query, query_params)?;
        let count = <Select<E> as PaginatorTrait<'_, _>>::count(query, get_connection().await)
            .await
            .map_err(DbErr::from)?;
        Ok(count)
    }

    pub async fn exists(
        filter_params: &[FilterParam],
        query_params: &[QueryParam],
    ) -> Result<bool, DbErr> {
        let query = Self::apply_filter(E::find(), filter_params)?;
        let query = Self::apply_query(query, query_params)?;
        let count = <Select<E> as PaginatorTrait<'_, _>>::count(query, get_connection().await)
            .await
            .map_err(DbErr::from)?;
        Ok(count > 0)
    }
}
