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
    store::{diesel::schema::mfg_batch_property_value, error::MfgBatchStoreError},
    MAX_COMMIT_NUM,
};
use diesel::{dsl::update, prelude::*};

pub(in crate::mfg_batch) trait UpdateMfgBatchOperation {
    fn update_mfg_batch(
        &self,
        mfg_batch_id: &str,
        service_id: Option<&str>,
        current_commit_num: i64,
    ) -> Result<(), MfgBatchStoreError>;
}

#[cfg(feature = "postgres")]
impl<'a> UpdateMfgBatchOperation for MfgBatchStoreOperations<'a, diesel::pg::PgConnection> {
    fn update_mfg_batch(
        &self,
        mfg_batch_id: &str,
        service_id: Option<&str>,
        current_commit_num: i64,
    ) -> Result<(), MfgBatchStoreError> {
        self.conn.transaction::<_, MfgBatchStoreError, _>(|| {
            pg::update_mfg_batch_property_values(
                &*self.conn,
                mfg_batch_id,
                service_id,
                current_commit_num,
            )?;

            Ok(())
        })
    }
}

#[cfg(feature = "sqlite")]
impl<'a> UpdateMfgBatchOperation for MfgBatchStoreOperations<'a, diesel::sqlite::SqliteConnection> {
    fn update_mfg_batch(
        &self,
        mfg_batch_id: &str,
        service_id: Option<&str>,
        current_commit_num: i64,
    ) -> Result<(), MfgBatchStoreError> {
        self.conn.transaction::<_, MfgBatchStoreError, _>(|| {
            sqlite::update_mfg_batch_property_values(
                &*self.conn,
                mfg_batch_id,
                service_id,
                current_commit_num,
            )?;

            Ok(())
        })
    }
}

#[cfg(feature = "postgres")]
mod pg {
    use super::*;

    pub fn update_mfg_batch_property_values(
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

    pub fn update_mfg_batch_property_values(
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
