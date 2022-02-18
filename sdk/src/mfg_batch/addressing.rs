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


/*
# Adding namespace based on contract seed logic
#   hashlib.sha512('grid_mfg_batch'.encode("utf-8")).hexdigest()[0:6]
# '11bb0e'
*/
pub const GRID_NAMESPACE: &str = "11bb0e";
pub const MFG_BATCH_PREFIX: &str = "01";
pub const GRID_MFG_BATCH_NAMESPACE: &str = "11bb0e01";

/// Computes the address of a GS1 product based on its 
pub fn compute_gs1_mfg_batch_address(gtin: &str) -> String {
    // 621ddee (grid namespace) + 02 (product namespace) + 01 (gs1 namespace)
    String::from(GRID_NAMESPACE)
        + MFG_BATCH_PREFIX
        + "01"
        + "00000000000000000000000000000000000000000000"
        + &format!("{:0>14}", gtin)
        + "00"
}
 