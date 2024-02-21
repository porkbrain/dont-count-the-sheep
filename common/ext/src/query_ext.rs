//! Useful extension on [`Query`].

use bevy::ecs::{
    query::{QueryData, QueryFilter, ROQueryItem},
    system::Query,
};

/// Useful extension on [`Query`].
pub trait QueryExt<Q: QueryData> {
    /// Panics if there is more than one entity.
    fn get_single_or_none(&self) -> Option<ROQueryItem<'_, Q>>;

    /// Panics if there is more than one entity.
    fn get_single_mut_or_none(&mut self) -> Option<Q::Item<'_>>;
}

impl<Q: QueryData, F: QueryFilter> QueryExt<Q> for Query<'_, '_, Q, F> {
    fn get_single_or_none(&self) -> Option<ROQueryItem<'_, Q>> {
        match self.get_single() {
            Ok(item) => Some(item),
            Err(bevy::ecs::query::QuerySingleError::MultipleEntities(_)) => {
                panic!("There should only be one entity")
            }
            Err(bevy::ecs::query::QuerySingleError::NoEntities(_)) => None,
        }
    }

    fn get_single_mut_or_none(&mut self) -> Option<Q::Item<'_>> {
        match self.get_single_mut() {
            Ok(item) => Some(item),
            Err(bevy::ecs::query::QuerySingleError::MultipleEntities(_)) => {
                panic!("There should only be one entity")
            }
            Err(bevy::ecs::query::QuerySingleError::NoEntities(_)) => None,
        }
    }
}
