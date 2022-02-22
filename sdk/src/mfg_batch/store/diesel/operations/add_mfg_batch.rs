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
            models::{NewMfgBatch, NewMfgBatchPropertyValue},
            schema::{mfg_batch, mfg_batch_property_value},
        },
        error::MfgBatchStoreError,
        MfgBatch,
    },
    MAX_COMMIT_NUM,
};

use diesel::{
    dsl::{insert_into, update},
    prelude::*,
};

pub(in crate::mfg_batch) trait AddMfgBatchOperation {
    fn add_mfg_batch(&self, mfg_batch: MfgBatch) -> Result<(), MfgBatchStoreError>;
}

#[cfg(feature = "postgres")]
impl<'a> AddMfgBatchOperation for MfgBatchStoreOperations<'a, diesel::pg::PgConnection> {
    fn add_mfg_batch(&self, mfg_batch: MfgBatch) -> Result<(), MfgBatchStoreError> {
        let (mfg_batch_model, property_models) = mfg_batch.into();

        self.conn.transaction::<_, MfgBatchStoreError, _>(|| {
            pg::insert_mfg_batch(&*self.conn, &mfg_batch_model)?;
            pg::insert_mfg_batch_property_values(&*self.conn, &property_models)?;

            Ok(())
        })
    }
}

#[cfg(feature = "sqlite")]
impl<'a> AddMfgBatchOperation for MfgBatchStoreOperations<'a, diesel::sqlite::SqliteConnection> {
    fn add_mfg_batch(&self, mfg_batch: MfgBatch) -> Result<(), MfgBatchStoreError> {
        let (mfg_batch_model, property_models) = mfg_batch.into();

        self.conn.transaction::<_, MfgBatchStoreError, _>(|| {
            sqlite::insert_mfg_batch(&*self.conn, &mfg_batch_model)?;
            sqlite::insert_mfg_batch_property_values(&*self.conn, &property_models)?;

            Ok(())
        })
    }
}

#[cfg(feature = "postgres")]
mod pg {
    use super::*;

    pub fn insert_mfg_batch(conn: &PgConnection, mfg_batch: &NewMfgBatch) -> QueryResult<()> {
        update_prod_end_commit_num(
            conn,
            &mfg_batch.mfg_batch_id,
            mfg_batch.service_id.as_deref(),
            mfg_batch.start_commit_num,
        )?;

        insert_into(mfg_batch::table)
            .values(mfg_batch)
            .execute(conn)
            .map(|_| ())
    }

    pub fn insert_mfg_batch_property_values(
        conn: &PgConnection,
        property_values: &[NewMfgBatchPropertyValue],
    ) -> QueryResult<()> {
        for value in property_values {
            update_prod_property_values(
                conn,
                &value.mfg_batch_id,
                value.service_id.as_deref(),
                value.start_commit_num,
            )?;
        }

        insert_into(mfg_batch_property_value::table)
            .values(property_values)
            .execute(conn)
            .map(|_| ())
    }
    fn update_prod_end_commit_num(
        conn: &PgConnection,
        mfg_batch_id: &str,
        service_id: Option<&str>,
        current_commit_num: i64,
    ) -> QueryResult<()> {
        let update = update(mfg_batch::table);

        if let Some(service_id) = service_id {
            update
                .filter(
                    mfg_batch::mfg_batch_id
                        .eq(mfg_batch_id)
                        .and(mfg_batch::end_commit_num.eq(MAX_COMMIT_NUM))
                        .and(mfg_batch::service_id.eq(service_id)),
                )
                .set(mfg_batch::end_commit_num.eq(current_commit_num))
                .execute(conn)
                .map(|_| ())
        } else {
            update
                .filter(
                    mfg_batch::mfg_batch_id
                        .eq(mfg_batch_id)
                        .and(mfg_batch::end_commit_num.eq(MAX_COMMIT_NUM)),
                )
                .set(mfg_batch::end_commit_num.eq(current_commit_num))
                .execute(conn)
                .map(|_| ())
        }
    }

    fn update_prod_property_values(
        conn: &PgConnection,
        mfg_batch_id: &str,
        service_id: Option<&str>,
        current_commit_num: i64,
    ) -> QueryResult<()> {
        let update = update(mfg_batch_property_value::table);

        if let Some(service_id) = service_id {
            update
                .filter(
                    mfg_batch_property_value::mfg_batch_id
                        .eq(mfg_batch_id)
                        .and(mfg_batch_property_value::end_commit_num.eq(MAX_COMMIT_NUM))
                        .and(mfg_batch_property_value::service_id.eq(service_id)),
                )
                .set(mfg_batch_property_value::end_commit_num.eq(current_commit_num))
                .execute(conn)
                .map(|_| ())
        } else {
            update
                .filter(
                    mfg_batch_property_value::mfg_batch_id
                        .eq(mfg_batch_id)
                        .and(mfg_batch_property_value::end_commit_num.eq(MAX_COMMIT_NUM)),
                )
                .set(mfg_batch_property_value::end_commit_num.eq(current_commit_num))
                .execute(conn)
                .map(|_| ())
        }
    }
}

#[cfg(feature = "sqlite")]
mod sqlite {
    use super::*;

    pub fn insert_mfg_batch(conn: &SqliteConnection, mfg_batch: &NewMfgBatch) -> QueryResult<()> {
        update_prod_end_commit_num(
            conn,
            &mfg_batch.mfg_batch_id,
            mfg_batch.service_id.as_deref(),
            mfg_batch.start_commit_num,
        )?;

        insert_into(mfg_batch::table)
            .values(mfg_batch)
            .execute(conn)
            .map(|_| ())
    }

    pub fn insert_mfg_batch_property_values(
        conn: &SqliteConnection,
        property_values: &[NewMfgBatchPropertyValue],
    ) -> QueryResult<()> {
        for value in property_values {
            update_prod_property_values(
                conn,
                &value.mfg_batch_id,
                value.service_id.as_deref(),
                value.start_commit_num,
            )?;
        }

        insert_into(mfg_batch_property_value::table)
            .values(property_values)
            .execute(conn)
            .map(|_| ())
    }

    fn update_prod_end_commit_num(
        conn: &SqliteConnection,
        mfg_batch_id: &str,
        service_id: Option<&str>,
        current_commit_num: i64,
    ) -> QueryResult<()> {
        let update = update(mfg_batch::table);

        if let Some(service_id) = service_id {
            update
                .filter(
                    mfg_batch::mfg_batch_id
                        .eq(mfg_batch_id)
                        .and(mfg_batch::end_commit_num.eq(MAX_COMMIT_NUM))
                        .and(mfg_batch::service_id.eq(service_id)),
                )
                .set(mfg_batch::end_commit_num.eq(current_commit_num))
                .execute(conn)
                .map(|_| ())
        } else {
            update
                .filter(
                    mfg_batch::mfg_batch_id
                        .eq(mfg_batch_id)
                        .and(mfg_batch::end_commit_num.eq(MAX_COMMIT_NUM)),
                )
                .set(mfg_batch::end_commit_num.eq(current_commit_num))
                .execute(conn)
                .map(|_| ())
        }
    }

    fn update_prod_property_values(
        conn: &SqliteConnection,
        mfg_batch_id: &str,
        service_id: Option<&str>,
        current_commit_num: i64,
    ) -> QueryResult<()> {
        let update = update(mfg_batch_property_value::table);

        if let Some(service_id) = service_id {
            update
                .filter(
                    mfg_batch_property_value::mfg_batch_id
                        .eq(mfg_batch_id)
                        .and(mfg_batch_property_value::end_commit_num.eq(MAX_COMMIT_NUM))
                        .and(mfg_batch_property_value::service_id.eq(service_id)),
                )
                .set(mfg_batch_property_value::end_commit_num.eq(current_commit_num))
                .execute(conn)
                .map(|_| ())
        } else {
            update
                .filter(
                    mfg_batch_property_value::mfg_batch_id
                        .eq(mfg_batch_id)
                        .and(mfg_batch_property_value::end_commit_num.eq(MAX_COMMIT_NUM)),
                )
                .set(mfg_batch_property_value::end_commit_num.eq(current_commit_num))
                .execute(conn)
                .map(|_| ())
        }
    }
}
