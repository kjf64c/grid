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

use crate::mfg_batch::{
    store::{
        diesel::{
            models::{MfgBatch as ModelMfgBatch, MfgBatchPropertyValue},
            schema::{mfg_batch, mfg_batch_property_value},
        },
        error::MfgBatchStoreError,
        MfgBatch, PropertyValue,
    },
    MAX_COMMIT_NUM,
};
use diesel::{prelude::*, result::Error::NotFound};

pub(in crate::mfg_batch) trait GetMfgBatchOperation {
    fn get_mfg_batch(
        &self,
        mfg_batch_id: &str,
        service_id: Option<&str>,
    ) -> Result<Option<MfgBatch>, MfgBatchStoreError>;
}

#[cfg(feature = "postgres")]
impl<'a> GetMfgBatchOperation for MfgBatchStoreOperations<'a, diesel::pg::PgConnection> {
    fn get_mfg_batch(
        &self,
        mfg_batch_id: &str,
        service_id: Option<&str>,
    ) -> Result<Option<MfgBatch>, MfgBatchStoreError> {
        self.conn.transaction::<_, MfgBatchStoreError, _>(|| {
            let mfg_batch =
                if let Some(mfg_batch) = pg::get_mfg_batch(&*self.conn, mfg_batch_id, service_id)? {
                    mfg_batch
                } else {
                    return Ok(None);
                };

            let root_values = pg::get_root_values(&*self.conn, mfg_batch_id)?;

            let values = pg::get_property_values(&*self.conn, root_values)?;

            Ok(Some(MfgBatch::from((mfg_batch, values))))
        })
    }
}

#[cfg(feature = "sqlite")]
impl<'a> GetMfgBatchOperation for MfgBatchStoreOperations<'a, diesel::sqlite::SqliteConnection> {
    fn get_mfg_batch(
        &self,
        mfg_batch_id: &str,
        service_id: Option<&str>,
    ) -> Result<Option<MfgBatch>, MfgBatchStoreError> {
        self.conn.transaction::<_, MfgBatchStoreError, _>(|| {
            let mfg_batch =
                if let Some(mfg_batch) = sqlite::get_mfg_batch(&*self.conn, mfg_batch_id, service_id)? {
                    mfg_batch
                } else {
                    return Ok(None);
                };

            let root_values = sqlite::get_root_values(&*self.conn, mfg_batch_id)?;

            let values = sqlite::get_property_values(&*self.conn, root_values)?;

            Ok(Some(MfgBatch::from((mfg_batch, values))))
        })
    }
}

#[cfg(feature = "postgres")]
mod pg {
    use super::*;

    pub fn get_mfg_batch(
        conn: &PgConnection,
        mfg_batch_id: &str,
        service_id: Option<&str>,
    ) -> QueryResult<Option<ModelMfgBatch>> {
        let mut query = mfg_batch::table
            .into_boxed()
            .select(mfg_batch::all_columns)
            .filter(
                mfg_batch::mfg_batch_id
                    .eq(mfg_batch_id)
                    .and(mfg_batch::end_commit_num.eq(MAX_COMMIT_NUM)),
            );

        if let Some(service_id) = service_id {
            query = query.filter(mfg_batch::service_id.eq(service_id));
        } else {
            query = query.filter(mfg_batch::service_id.is_null());
        }

        query
            .first(conn)
            .map(Some)
            .or_else(|err| if err == NotFound { Ok(None) } else { Err(err) })
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

    pub fn get_mfg_batch(
        conn: &SqliteConnection,
        mfg_batch_id: &str,
        service_id: Option<&str>,
    ) -> QueryResult<Option<ModelMfgBatch>> {
        let mut query = mfg_batch::table
            .into_boxed()
            .select(mfg_batch::all_columns)
            .filter(
                mfg_batch::mfg_batch_id
                    .eq(mfg_batch_id)
                    .and(mfg_batch::end_commit_num.eq(MAX_COMMIT_NUM)),
            );

        if let Some(service_id) = service_id {
            query = query.filter(mfg_batch::service_id.eq(service_id));
        } else {
            query = query.filter(mfg_batch::service_id.is_null());
        }

        query
            .first(conn)
            .map(Some)
            .or_else(|err| if err == NotFound { Ok(None) } else { Err(err) })
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
