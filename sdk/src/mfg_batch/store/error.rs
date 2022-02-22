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

use std::error::Error;
use std::fmt;

#[cfg(feature = "diesel")]
use crate::error::ConstraintViolationType;
use crate::error::{ConstraintViolationError, InternalError, ResourceTemporarilyUnavailableError};

/// Represents Store errors
#[derive(Debug)]
pub enum MfgBatchStoreError {
    InternalError(InternalError),
    ConstraintViolationError(ConstraintViolationError),
    ResourceTemporarilyUnavailableError(ResourceTemporarilyUnavailableError),
    NotFoundError(String),
}

impl Error for MfgBatchStoreError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            MfgBatchStoreError::InternalError(err) => Some(err),
            MfgBatchStoreError::ConstraintViolationError(err) => Some(err),
            MfgBatchStoreError::ResourceTemporarilyUnavailableError(err) => Some(err),
            MfgBatchStoreError::NotFoundError(_) => None,
        }
    }
}

impl fmt::Display for MfgBatchStoreError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            MfgBatchStoreError::InternalError(err) => err.fmt(f),
            MfgBatchStoreError::ConstraintViolationError(err) => err.fmt(f),
            MfgBatchStoreError::ResourceTemporarilyUnavailableError(err) => err.fmt(f),
            MfgBatchStoreError::NotFoundError(ref s) => write!(f, "Element not found: {}", s),
        }
    }
}

#[cfg(feature = "diesel")]
impl From<diesel::result::Error> for MfgBatchStoreError {
    fn from(err: diesel::result::Error) -> Self {
        match err {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UniqueViolation,
                _,
            ) => MfgBatchStoreError::ConstraintViolationError(
                ConstraintViolationError::from_source_with_violation_type(
                    ConstraintViolationType::Unique,
                    Box::new(err),
                ),
            ),
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::ForeignKeyViolation,
                _,
            ) => MfgBatchStoreError::ConstraintViolationError(
                ConstraintViolationError::from_source_with_violation_type(
                    ConstraintViolationType::ForeignKey,
                    Box::new(err),
                ),
            ),
            _ => MfgBatchStoreError::InternalError(InternalError::from_source(Box::new(err))),
        }
    }
}

#[cfg(feature = "diesel")]
impl From<diesel::r2d2::PoolError> for MfgBatchStoreError {
    fn from(err: diesel::r2d2::PoolError) -> MfgBatchStoreError {
        MfgBatchStoreError::ResourceTemporarilyUnavailableError(
            ResourceTemporarilyUnavailableError::from_source(Box::new(err)),
        )
    }
}

/// Represents ProductBuilder errors
#[derive(Debug)]
pub enum MfgBatchBuilderError {
    /// Returned when a required field was not set
    MissingRequiredField(String),
    /// Returned when an error occurs building the mfg_batch
    BuildError(Box<dyn Error>),
}

impl Error for MfgBatchBuilderError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            MfgBatchBuilderError::MissingRequiredField(_) => None,
            MfgBatchBuilderError::BuildError(err) => Some(&**err),
        }
    }
}

impl fmt::Display for MfgBatchBuilderError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            MfgBatchBuilderError::MissingRequiredField(ref s) => {
                write!(f, "failed to build mfg_batch: {}", s)
            }
            MfgBatchBuilderError::BuildError(ref s) => {
                write!(f, "failed to build mfg_batch: {}", s)
            }
        }
    }
}
