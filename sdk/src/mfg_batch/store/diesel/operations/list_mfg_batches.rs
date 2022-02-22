// Copyright 2018-2020 Cargill Incorporated
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use super::MfgBatchStoreOperations;

use crate::{
    error::InternalError,
    paging::Paging,
    mfg_batch::{
        store::{
            diesel::{
                models::{MfgBatch as ModelMfgBatch, MfgBatchPropertyValue},
                schema::{mfg_batch, mfg_batch_property_value},
            },
            error::MfgBatchStoreError,
            MfgBatch, MfgBatchList, PropertyValue,
        },
        MAX_COMMIT_NUM,
    },
};

use diesel::prelude::*;
use std::convert::TryInto;

pub(in crate::mfg_batch) trait ListMfgBatchsOperation {
    fn list_mfg_batches(
        &self,
        service_id: Option<&str>,
        offset: i64,
        limit: i64,
    ) -> Result<MfgBatchList, MfgBatchStoreError>;
}

#[cfg(feature = "postgres")]
impl<'a> ListMfgBatchsOperation for MfgBatchStoreOperations<'a, diesel::pg::PgConnection> {
    fn list_mfg_batches(
        &self,
        service_id: Option<&str>,
        offset: i64,
        limit: i64,
    ) -> Result<MfgBatchList, MfgBatchStoreError> {
        self.conn.transaction::<_, MfgBatchStoreError, _>(|| {
            let db_mfg_batches = pg::list_mfg_batches(&*self.conn, service_id, offset, limit)?;

            let total = db_mfg_batches.len().try_into().map_err(|err| {
                MfgBatchStoreError::InternalError(InternalError::from_source(Box::new(err)))
            })?;

            let mut mfg_batches = Vec::new();

            for mfg_batch in db_mfg_batches {
                let root_values = pg::get_root_values(&*self.conn, &mfg_batch.mfg_batch_id)?;

                let values = pg::get_property_values(&*self.conn, root_values)?;

                mfg_batches.push(MfgBatch::from((mfg_batch, values)));
            }

            Ok(MfgBatchList::new(
                mfg_batches,
                Paging::new(offset, limit, total),
            ))
        })
    }
}

#[cfg(feature = "sqlite")]
impl<'a> ListMfgBatchsOperation for MfgBatchStoreOperations<'a, diesel::sqlite::SqliteConnection> {
    fn list_mfg_batches(
        &self,
        service_id: Option<&str>,
        offset: i64,
        limit: i64,
    ) -> Result<MfgBatchList, MfgBatchStoreError> {
        self.conn.transaction::<_, MfgBatchStoreError, _>(|| {
            let db_mfg_batches = sqlite::list_mfg_batches(&*self.conn, service_id, offset, limit)?;

            let total = db_mfg_batches.len().try_into().map_err(|err| {
                MfgBatchStoreError::InternalError(InternalError::from_source(Box::new(err)))
            })?;

            let mut mfg_batches = Vec::new();

            for mfg_batch in db_mfg_batches {
                let root_values = sqlite::get_root_values(&*self.conn, &mfg_batch.mfg_batch_id)?;

                let values = sqlite::get_property_values(&*self.conn, root_values)?;

                mfg_batches.push(MfgBatch::from((mfg_batch, values)));
            }

            Ok(MfgBatchList::new(
                mfg_batches,
                Paging::new(offset, limit, total),
            ))
        })
    }
}

#[cfg(feature = "postgres")]
mod pg {
    use super::*;

    pub fn list_mfg_batches(
        conn: &PgConnection,
        service_id: Option<&str>,
        offset: i64,
        limit: i64,
    ) -> QueryResult<Vec<ModelMfgBatch>> {
        let mut query = mfg_batch::table
            .into_boxed()
            .select(mfg_batch::all_columns)
            .limit(limit)
            .offset(offset)
            .filter(mfg_batch::end_commit_num.eq(MAX_COMMIT_NUM));

        if let Some(service_id) = service_id {
            query = query.filter(mfg_batch::service_id.eq(service_id));
        } else {
            query = query.filter(mfg_batch::service_id.is_null());
        }
        query.load::<ModelMfgBatch>(conn)
    }

    pub fn get_root_values(
        conn: &PgConnection,
        mfg_batch_id: &str,
    ) -> QueryResult<Vec<MfgBatchPropertyValue>> {
        mfg_batch_property_value::table
            .select(mfg_batch_property_value::all_columns)
            .filter(
                mfg_batch_property_value::mfg_batch_id
                    .eq(mfg_batch_id)
                    .and(mfg_batch_property_value::parent_property.is_null())
                    .and(mfg_batch_property_value::end_commit_num.eq(MAX_COMMIT_NUM)),
            )
            .load::<MfgBatchPropertyValue>(conn)
    }

    pub fn get_property_values(
        conn: &PgConnection,
        root_values: Vec<MfgBatchPropertyValue>,
    ) -> Result<Vec<PropertyValue>, MfgBatchStoreError> {
        let mut definitions = Vec::new();

        for root_value in root_values {
            let children = mfg_batch_property_value::table
                .select(mfg_batch_property_value::all_columns)
                .filter(mfg_batch_property_value::parent_property.eq(&root_value.parent_property))
                .load(conn)?;

            if children.is_empty() {
                definitions.push(PropertyValue::from(root_value));
            } else {
                definitions.push(PropertyValue::from((
                    root_value,
                    get_property_values(conn, children)?,
                )));
            }
        }

        Ok(definitions)
    }
}

#[cfg(feature = "sqlite")]
mod sqlite {
    use super::*;

    pub fn list_mfg_batches(
        conn: &SqliteConnection,
        service_id: Option<&str>,
        offset: i64,
        limit: i64,
    ) -> QueryResult<Vec<ModelMfgBatch>> {
        let mut query = mfg_batch::table
            .into_boxed()
            .select(mfg_batch::all_columns)
            .limit(limit)
            .offset(offset)
            .filter(mfg_batch::end_commit_num.eq(MAX_COMMIT_NUM));

        if let Some(service_id) = service_id {
            query = query.filter(mfg_batch::service_id.eq(service_id));
        } else {
            query = query.filter(mfg_batch::service_id.is_null());
        }
        query.load::<ModelMfgBatch>(conn)
    }

    pub fn get_root_values(
        conn: &SqliteConnection,
        mfg_batch_id: &str,
    ) -> QueryResult<Vec<MfgBatchPropertyValue>> {
        mfg_batch_property_value::table
            .select(mfg_batch_property_value::all_columns)
            .filter(
                mfg_batch_property_value::mfg_batch_id
                    .eq(mfg_batch_id)
                    .and(mfg_batch_property_value::parent_property.is_null())
                    .and(mfg_batch_property_value::end_commit_num.eq(MAX_COMMIT_NUM)),
            )
            .load::<MfgBatchPropertyValue>(conn)
    }

    pub fn get_property_values(
        conn: &SqliteConnection,
        root_values: Vec<MfgBatchPropertyValue>,
    ) -> Result<Vec<PropertyValue>, MfgBatchStoreError> {
        let mut definitions = Vec::new();

        for root_value in root_values {
            let children = mfg_batch_property_value::table
                .select(mfg_batch_property_value::all_columns)
                .filter(mfg_batch_property_value::parent_property.eq(&root_value.parent_property))
                .load(conn)?;

            if children.is_empty() {
                definitions.push(PropertyValue::from(root_value));
            } else {
                definitions.push(PropertyValue::from((
                    root_value,
                    get_property_values(conn, children)?,
                )));
            }
        }

        Ok(definitions)
    }
}
