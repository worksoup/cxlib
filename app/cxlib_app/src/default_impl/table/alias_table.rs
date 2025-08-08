use std::{borrow::Borrow, collections::HashMap};

use crate::{BinCode, NormalTableTrait, StoreError, TableDefinitionTrait};
use cxlib_error_utils::MaybeFatalError;
use cxlib_internal::types::{__private::UnhandledGeoaddr, Geolocation};
use log::warn;
use redb::ReadableTable;

pub struct AliasTable;

impl AliasTable {
    pub fn has_alias(
        table: &impl ReadableTable<
            <Self as TableDefinitionTrait>::Key,
            <Self as TableDefinitionTrait>::Value,
        >,
        alias: &str,
    ) -> Result<bool, StoreError> {
        Self::contains_key(table, &alias.to_owned())
    }

    pub fn delete_alias(
        table: &mut redb::Table<
            <Self as TableDefinitionTrait>::Key,
            <Self as TableDefinitionTrait>::Value,
        >,
        alias: &str,
    ) -> Result<(), StoreError> {
        table.remove(&alias.to_owned())?;
        Ok(())
    }

    pub fn add_alias(
        table: &mut redb::Table<
            <Self as TableDefinitionTrait>::Key,
            <Self as TableDefinitionTrait>::Value,
        >,
        alias: &str,
        location: impl for<'a> Borrow<
            <<AliasTable as TableDefinitionTrait>::Value as redb::Value>::SelfType<'a>,
        >,
    ) -> Result<(), StoreError> {
        table.insert(alias.to_owned(), location)?;
        Ok(())
    }
    pub fn update_alias_and<
        A: Fn(
            &mut redb::Table<
                <Self as TableDefinitionTrait>::Key,
                <Self as TableDefinitionTrait>::Value,
            >,
            &str,
            &<<AliasTable as TableDefinitionTrait>::Value as redb::Value>::SelfType<'_>,
        ),
    >(
        table: &mut redb::Table<
            <Self as TableDefinitionTrait>::Key,
            <Self as TableDefinitionTrait>::Value,
        >,
        alias: &str,
        location: &<<AliasTable as TableDefinitionTrait>::Value as redb::Value>::SelfType<'_>,
        and: A,
    ) -> Result<(), StoreError> {
        let old = table
            .insert(alias.to_owned(), location)?
            .map(|inner| inner.value());
        if let Some(old) = old {
            and(table, alias, &old);
        }
        Ok(())
    }
    pub fn get_all_aliases(
        table: &impl ReadableTable<
            <Self as TableDefinitionTrait>::Key,
            <Self as TableDefinitionTrait>::Value,
        >,
    ) -> Result<
        HashMap<String, <<AliasTable as TableDefinitionTrait>::Value as redb::Value>::SelfType<'_>>,
        StoreError,
    > {
        let iter = table.iter()?;
        let mut map = HashMap::new();
        for i in iter {
            match i {
                Ok(i) => {
                    map.insert(i.0.value(), i.1.value());
                }
                Err(e) => {
                    let e = StoreError::from(e);
                    if e.is_fatal() {
                        Err(e)?
                    } else {
                        continue;
                    }
                }
            }
        }
        Ok(map)
    }
    pub fn get_aliases(
        table: &impl ReadableTable<
            <Self as TableDefinitionTrait>::Key,
            <Self as TableDefinitionTrait>::Value,
        >,
        location: impl Borrow<Geolocation>,
    ) -> Result<Vec<String>, StoreError> {
        Self::get_aliases_and(table, location, |a| a)
    }
    pub fn get_aliases_and<A: Fn(String) -> B, B>(
        table: &impl ReadableTable<
            <Self as TableDefinitionTrait>::Key,
            <Self as TableDefinitionTrait>::Value,
        >,
        location: impl Borrow<Geolocation>,
        and: A,
    ) -> Result<Vec<B>, StoreError> {
        let iter = table.iter()?;
        let mut results = Vec::new();
        for r in iter {
            match r {
                Ok((k, v)) => {
                    if v.value().geolocation().eq(location.borrow()) {
                        results.push(and(k.value()));
                    }
                }
                Err(e) => {
                    let e = StoreError::from(e);
                    if e.is_fatal() {
                        return Err(e);
                    }
                    warn!("{e}");
                }
            }
        }
        Ok(results)
    }

    pub fn get_location<'a>(
        table: &impl ReadableTable<
            <Self as TableDefinitionTrait>::Key,
            <Self as TableDefinitionTrait>::Value,
        >,
        alias: &str,
    ) -> Result<
        Option<<<AliasTable as TableDefinitionTrait>::Value as redb::Value>::SelfType<'a>>,
        StoreError,
    > {
        Ok(table.get(alias.to_owned())?.map(|v| v.value()))
    }
}
impl NormalTableTrait for AliasTable {}
impl TableDefinitionTrait for AliasTable {
    type Key = String;
    type Value = BinCode<UnhandledGeoaddr>;
    type Context<'cxt> = ();
    const NAME: &'static str = "alias";
}
