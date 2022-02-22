// Copyright 2018-2021 Cargill Incorporated
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

pub(in crate::mfg_batch) mod models;
mod operations;
pub(in crate) mod schema;

use crate::error::ResourceTemporarilyUnavailableError;

use operations::{
    add_mfg_batch::AddMfgBatchOperation, delete_mfg_batch::DeleteMfgBatchOperation,
    get_mfg_batch::GetMfgBatchOperation, list_mfg_batches::ListMfgBatchsOperation,
    update_mfg_batch::UpdateMfgBatchOperation, MfgBatchStoreOperations,
};

use diesel::connection::AnsiTransactionManager;
use diesel::r2d2::{ConnectionManager, Pool};

use super::{MfgBatch, MfgBatchList, MfgBatchStore, MfgBatchStoreError};

#[derive(Clone)]
pub struct DieselMfgBatchStore<C: diesel::Connection + 'static> {
    connection_pool: Pool<ConnectionManager<C>>,
}

impl<C: diesel::Connection> DieselMfgBatchStore<C> {
    pub fn new(connection_pool: Pool<ConnectionManager<C>>) -> Self {
        DieselMfgBatchStore { connection_pool }
    }
}

#[cfg(feature = "postgres")]
impl MfgBatchStore for DieselMfgBatchStore<diesel::pg::PgConnection> {
    fn add_mfg_batch(&self, mfg_batch: MfgBatch) -> Result<(), MfgBatchStoreError> {
        MfgBatchStoreOperations::new(&*self.connection_pool.get().map_err(|err| {
            MfgBatchStoreError::ResourceTemporarilyUnavailableError(
                ResourceTemporarilyUnavailableError::from_source(Box::new(err)),
            )
        })?)
        .add_mfg_batch(mfg_batch)
    }

    fn get_mfg_batch(
        &self,
        mfg_batch_id: &str,
        service_id: Option<&str>,
    ) -> Result<Option<MfgBatch>, MfgBatchStoreError> {
        MfgBatchStoreOperations::new(&*self.connection_pool.get().map_err(|err| {
            MfgBatchStoreError::ResourceTemporarilyUnavailableError(
                ResourceTemporarilyUnavailableError::from_source(Box::new(err)),
            )
        })?)
        .get_mfg_batch(mfg_batch_id, service_id)
    }

    fn list_mfg_batches(
        &self,
        service_id: Option<&str>,
        offset: i64,
        limit: i64,
    ) -> Result<MfgBatchList, MfgBatchStoreError> {
        MfgBatchStoreOperations::new(&*self.connection_pool.get().map_err(|err| {
            MfgBatchStoreError::ResourceTemporarilyUnavailableError(
                ResourceTemporarilyUnavailableError::from_source(Box::new(err)),
            )
        })?)
        .list_mfg_batches(service_id, offset, limit)
    }

    fn update_mfg_batch(
        &self,
        mfg_batch_id: &str,
        service_id: Option<&str>,
        current_commit_num: i64,
    ) -> Result<(), MfgBatchStoreError> {
        MfgBatchStoreOperations::new(&*self.connection_pool.get().map_err(|err| {
            MfgBatchStoreError::ResourceTemporarilyUnavailableError(
                ResourceTemporarilyUnavailableError::from_source(Box::new(err)),
            )
        })?)
        .update_mfg_batch(mfg_batch_id, service_id, current_commit_num)
    }

    fn delete_mfg_batch(
        &self,
        address: &str,
        current_commit_num: i64,
    ) -> Result<(), MfgBatchStoreError> {
        MfgBatchStoreOperations::new(&*self.connection_pool.get().map_err(|err| {
            MfgBatchStoreError::ResourceTemporarilyUnavailableError(
                ResourceTemporarilyUnavailableError::from_source(Box::new(err)),
            )
        })?)
        .delete_mfg_batch(address, current_commit_num)
    }
}

#[cfg(feature = "sqlite")]
impl MfgBatchStore for DieselMfgBatchStore<diesel::sqlite::SqliteConnection> {
    fn add_mfg_batch(&self, mfg_batch: MfgBatch) -> Result<(), MfgBatchStoreError> {
        MfgBatchStoreOperations::new(&*self.connection_pool.get().map_err(|err| {
            MfgBatchStoreError::ResourceTemporarilyUnavailableError(
                ResourceTemporarilyUnavailableError::from_source(Box::new(err)),
            )
        })?)
        .add_mfg_batch(mfg_batch)
    }

    fn get_mfg_batch(
        &self,
        mfg_batch_id: &str,
        service_id: Option<&str>,
    ) -> Result<Option<MfgBatch>, MfgBatchStoreError> {
        MfgBatchStoreOperations::new(&*self.connection_pool.get().map_err(|err| {
            MfgBatchStoreError::ResourceTemporarilyUnavailableError(
                ResourceTemporarilyUnavailableError::from_source(Box::new(err)),
            )
        })?)
        .get_mfg_batch(mfg_batch_id, service_id)
    }

    fn list_mfg_batches(
        &self,
        service_id: Option<&str>,
        offset: i64,
        limit: i64,
    ) -> Result<MfgBatchList, MfgBatchStoreError> {
        MfgBatchStoreOperations::new(&*self.connection_pool.get().map_err(|err| {
            MfgBatchStoreError::ResourceTemporarilyUnavailableError(
                ResourceTemporarilyUnavailableError::from_source(Box::new(err)),
            )
        })?)
        .list_mfg_batches(service_id, offset, limit)
    }

    fn update_mfg_batch(
        &self,
        mfg_batch_id: &str,
        service_id: Option<&str>,
        current_commit_num: i64,
    ) -> Result<(), MfgBatchStoreError> {
        MfgBatchStoreOperations::new(&*self.connection_pool.get().map_err(|err| {
            MfgBatchStoreError::ResourceTemporarilyUnavailableError(
                ResourceTemporarilyUnavailableError::from_source(Box::new(err)),
            )
        })?)
        .update_mfg_batch(mfg_batch_id, service_id, current_commit_num)
    }

    fn delete_mfg_batch(
        &self,
        address: &str,
        current_commit_num: i64,
    ) -> Result<(), MfgBatchStoreError> {
        MfgBatchStoreOperations::new(&*self.connection_pool.get().map_err(|err| {
            MfgBatchStoreError::ResourceTemporarilyUnavailableError(
                ResourceTemporarilyUnavailableError::from_source(Box::new(err)),
            )
        })?)
        .delete_mfg_batch(address, current_commit_num)
    }
}

pub struct DieselConnectionMfgBatchStore<'a, C>
where
    C: diesel::Connection<TransactionManager = AnsiTransactionManager> + 'static,
    C::Backend: diesel::backend::UsesAnsiSavepointSyntax,
{
    connection: &'a C,
}

impl<'a, C> DieselConnectionMfgBatchStore<'a, C>
where
    C: diesel::Connection<TransactionManager = AnsiTransactionManager> + 'static,
    C::Backend: diesel::backend::UsesAnsiSavepointSyntax,
{
    pub fn new(connection: &'a C) -> Self {
        DieselConnectionMfgBatchStore { connection }
    }
}

#[cfg(feature = "postgres")]
impl<'a> MfgBatchStore for DieselConnectionMfgBatchStore<'a, diesel::pg::PgConnection> {
    fn add_mfg_batch(&self, mfg_batch: MfgBatch) -> Result<(), MfgBatchStoreError> {
        MfgBatchStoreOperations::new(self.connection).add_mfg_batch(mfg_batch)
    }

    fn get_mfg_batch(
        &self,
        mfg_batch_id: &str,
        service_id: Option<&str>,
    ) -> Result<Option<MfgBatch>, MfgBatchStoreError> {
        MfgBatchStoreOperations::new(self.connection).get_mfg_batch(mfg_batch_id, service_id)
    }

    fn list_mfg_batches(
        &self,
        service_id: Option<&str>,
        offset: i64,
        limit: i64,
    ) -> Result<MfgBatchList, MfgBatchStoreError> {
        MfgBatchStoreOperations::new(self.connection).list_mfg_batches(service_id, offset, limit)
    }

    fn update_mfg_batch(
        &self,
        mfg_batch_id: &str,
        service_id: Option<&str>,
        current_commit_num: i64,
    ) -> Result<(), MfgBatchStoreError> {
        MfgBatchStoreOperations::new(self.connection).update_mfg_batch(
            mfg_batch_id,
            service_id,
            current_commit_num,
        )
    }

    fn delete_mfg_batch(
        &self,
        address: &str,
        current_commit_num: i64,
    ) -> Result<(), MfgBatchStoreError> {
        MfgBatchStoreOperations::new(self.connection).delete_mfg_batch(address, current_commit_num)
    }
}

#[cfg(feature = "sqlite")]
impl<'a> MfgBatchStore for DieselConnectionMfgBatchStore<'a, diesel::sqlite::SqliteConnection> {
    fn add_mfg_batch(&self, mfg_batch: MfgBatch) -> Result<(), MfgBatchStoreError> {
        MfgBatchStoreOperations::new(self.connection).add_mfg_batch(mfg_batch)
    }

    fn get_mfg_batch(
        &self,
        mfg_batch_id: &str,
        service_id: Option<&str>,
    ) -> Result<Option<MfgBatch>, MfgBatchStoreError> {
        MfgBatchStoreOperations::new(self.connection).get_mfg_batch(mfg_batch_id, service_id)
    }

    fn list_mfg_batches(
        &self,
        service_id: Option<&str>,
        offset: i64,
        limit: i64,
    ) -> Result<MfgBatchList, MfgBatchStoreError> {
        MfgBatchStoreOperations::new(self.connection).list_mfg_batches(service_id, offset, limit)
    }

    fn update_mfg_batch(
        &self,
        mfg_batch_id: &str,
        service_id: Option<&str>,
        current_commit_num: i64,
    ) -> Result<(), MfgBatchStoreError> {
        MfgBatchStoreOperations::new(self.connection).update_mfg_batch(
            mfg_batch_id,
            service_id,
            current_commit_num,
        )
    }

    fn delete_mfg_batch(
        &self,
        address: &str,
        current_commit_num: i64,
    ) -> Result<(), MfgBatchStoreError> {
        MfgBatchStoreOperations::new(self.connection).delete_mfg_batch(address, current_commit_num)
    }
}
