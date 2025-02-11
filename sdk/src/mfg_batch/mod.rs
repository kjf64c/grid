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

pub mod addressing;
pub mod store;

pub const MAX_COMMIT_NUM: i64 = i64::MAX;
//TODO decide what to do with internal validations 
/*
- Store non-gs1 internal blocks
-- Link to supporting information
-- Consider if information could extend selectively
*/
#[cfg(feature = "mfg-batch-gdsn")] // Not a real thing...
pub mod gdsn;
