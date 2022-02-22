// Copyright (c) 2019 Target Brands, Inc.
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

cfg_if! {
    if #[cfg(target_arch = "wasm32")] {
        use sabre_sdk::ApplyError;
        use sabre_sdk::TransactionContext;
    } else {
        use sawtooth_sdk::processor::handler::ApplyError;
        use sawtooth_sdk::processor::handler::TransactionContext;
    }
}

use grid_sdk::{
    pike::addressing::compute_organization_address,
    // Calls the grid SDK GS1 validation routine in sdk/src/mfg_batch
    mfg_batch::addressing::compute_gs1_mfg_batch_address,
    protocol::{
        pike::state::{Organization, OrganizationList},
        mfg_batch::state::{MfgBatch, MfgBatchList, MfgBatchListBuilder},
        schema::state::{Schema, SchemaList},
    },
    protos::{FromBytes, IntoBytes},
    schema::addressing::compute_schema_address,
};

pub struct MfgBatchState<'a> {
    context: &'a dyn TransactionContext,
}

impl<'a> MfgBatchState<'a> {
    pub fn new(context: &'a dyn TransactionContext) -> MfgBatchState {
        MfgBatchState { context }
    }

    pub fn get_mfg_batch(&self, mfg_batch_id: &str) -> Result<Option<MfgBatch>, ApplyError> {
        // Calls the grid SDK GS1 validation routine in sdk/src/mfg_batch
        let address = compute_gs1_mfg_batch_address(mfg_batch_id); //mfg_batch id = gtin
        let d = self.context.get_state_entry(&address)?;
        match d {
            Some(packed) => {
                let mfg_batches = match MfgBatchList::from_bytes(packed.as_slice()) {
                    Ok(mfg_batches) => mfg_batches,
                    Err(_) => {
                        return Err(ApplyError::InternalError(String::from(
                            "Cannot deserialize mfg_batch list",
                        )));
                    }
                };

                // find the mfg_batch with the correct id
                Ok(mfg_batches
                    .mfg_batches()
                    .iter()
                    .find(|p| p.mfg_batch_id() == mfg_batch_id)
                    .cloned())
            }
            None => Ok(None),
        }
    }

    pub fn set_mfg_batch(&self, mfg_batch_id: &str, mfg_batch: MfgBatch) -> Result<(), ApplyError> {
        // Calls the grid SDK GS1 validation routine in sdk/src/mfg_batch
        let address = compute_gs1_mfg_batch_address(mfg_batch_id);
        let d = self.context.get_state_entry(&address)?;
        let mut mfg_batches = match d {
            Some(packed) => match MfgBatchList::from_bytes(packed.as_slice()) {
                Ok(mfg_batch_list) => mfg_batch_list.mfg_batches().to_vec(),
                Err(err) => {
                    return Err(ApplyError::InternalError(format!(
                        "Cannot deserialize mfg_batch list: {:?}",
                        err
                    )));
                }
            },
            None => vec![],
        };

        let mut index = None;
        for (i, mfg_batch) in mfg_batches.iter().enumerate() {
            if mfg_batch.mfg_batch_id() == mfg_batch_id {
                index = Some(i);
                break;
            }
        }

        if let Some(i) = index {
            mfg_batches.remove(i);
        }
        mfg_batches.push(mfg_batch);
        mfg_batches.sort_by_key(|r| r.mfg_batch_id().to_string());
        let mfg_batch_list = MfgBatchListBuilder::new()
            .with_mfg_batches(mfg_batches)
            .build()
            .map_err(|err| {
                ApplyError::InvalidTransaction(format!("Cannot build mfg_batch list: {:?}", err))
            })?;

        let serialized = match mfg_batch_list.into_bytes() {
            Ok(serialized) => serialized,
            Err(err) => {
                return Err(ApplyError::InvalidTransaction(format!(
                    "Cannot serialize mfg_batch list: {:?}",
                    err
                )));
            }
        };
        self.context
            .set_state_entry(address, serialized)
            .map_err(|err| ApplyError::InternalError(format!("{}", err)))?;
        Ok(())
    }

    // Currently mfg_batch_id = gtin
    pub fn remove_mfg_batch(&self, mfg_batch_id: &str) -> Result<(), ApplyError> {
        // Calls the grid SDK GS1 validation routine in sdk/src/mfg_batch
        let address = compute_gs1_mfg_batch_address(mfg_batch_id);
        let d = self.context.get_state_entry(&address)?;
        let mfg_batches = match d {
            Some(packed) => match MfgBatchList::from_bytes(packed.as_slice()) {
                Ok(mfg_batch_list) => mfg_batch_list.mfg_batches().to_vec(),
                Err(err) => {
                    return Err(ApplyError::InternalError(format!(
                        "Cannot deserialize mfg_batch list: {:?}",
                        err
                    )));
                }
            },
            None => vec![],
        };

        // Filter out the mfg_batch we are deleting
        let filtered_mfg_batches = mfg_batches
            .into_iter()
            .filter(|p| p.mfg_batch_id() != mfg_batch_id)
            .collect::<Vec<_>>();

        // If the only mfg_batch at the address was the one we are removing, we can delete the entire state entry
        // Else, we can set the the filtered mfg_batch list at the address
        if filtered_mfg_batches.is_empty() {
            self.context
                .delete_state_entries(&[address])
                .map_err(|err| ApplyError::InternalError(format!("{}", err)))?;
        } else {
            let mfg_batch_list = MfgBatchListBuilder::new()
                .with_mfg_batches(filtered_mfg_batches)
                .build()
                .map_err(|err| {
                    ApplyError::InvalidTransaction(format!("Cannot build mfg_batch list: {:?}", err))
                })?;

            let serialized = match mfg_batch_list.into_bytes() {
                Ok(serialized) => serialized,
                Err(_) => {
                    return Err(ApplyError::InternalError(String::from(
                        "Cannot serialize mfg_batch list",
                    )));
                }
            };
            self.context
                .set_state_entry(address, serialized)
                .map_err(|err| ApplyError::InternalError(format!("{}", err)))?;
        }

        Ok(())
    }

    pub fn get_organization(&self, id: &str) -> Result<Option<Organization>, ApplyError> {
        let address = compute_organization_address(id);
        let d = self.context.get_state_entry(&address)?;
        match d {
            Some(packed) => {
                let orgs: OrganizationList = match OrganizationList::from_bytes(packed.as_slice()) {
                    Ok(orgs) => orgs,
                    Err(err) => {
                        return Err(ApplyError::InternalError(format!(
                            "Cannot deserialize organization list: {:?}",
                            err,
                        )))
                    }
                };

                for org in orgs.organizations() {
                    if org.org_id() == id {
                        return Ok(Some(org.clone()));
                    }
                }
                Ok(None)
            }
            None => Ok(None),
        }
    }

    pub fn get_schema(&self, name: &str) -> Result<Option<Schema>, ApplyError> {
        let address = compute_schema_address(name);
        let d = self.context.get_state_entry(&address)?;
        match d {
            Some(packed) => {
                let schemas = match SchemaList::from_bytes(packed.as_slice()) {
                    Ok(schemas) => schemas,
                    Err(err) => {
                        return Err(ApplyError::InvalidTransaction(format!(
                            "Cannot deserialize schema list: {:?}",
                            err,
                        )));
                    }
                };

                // find the schema with the correct name
                for schema in schemas.schemas() {
                    if schema.name() == name {
                        return Ok(Some(schema.clone()));
                    }
                }
                Ok(None)
            }
            None => Ok(None),
        }
    }
}

/*

#[cfg(test)]
mod tests {
    use super::*;

    use std::cell::RefCell;
    use std::collections::HashMap;

    use grid_sdk::protocol::mfg_batch::state::{MfgBatchBuilder, MfgBatchNamespace};
    use grid_sdk::protocol::schema::state::{DataType, PropertyValue, PropertyValueBuilder};

    use sawtooth_sdk::processor::handler::{ContextError, TransactionContext};

    const mfg_batch_ID: &str = "688955434684";

    #[derive(Default, Debug)]
    /// A MockTransactionContext that can be used to test TrackAndTraceState
    struct MockTransactionContext {
        state: RefCell<HashMap<String, Vec<u8>>>,
    }

    impl TransactionContext for MockTransactionContext {
        fn get_state_entries(
            &self,
            addresses: &[String],
        ) -> Result<Vec<(String, Vec<u8>)>, ContextError> {
            let mut results = Vec::new();
            for addr in addresses {
                let data = match self.state.borrow().get(addr) {
                    Some(data) => data.clone(),
                    None => Vec::new(),
                };
                results.push((addr.to_string(), data));
            }
            Ok(results)
        }

        fn set_state_entries(&self, entries: Vec<(String, Vec<u8>)>) -> Result<(), ContextError> {
            for (addr, data) in entries {
                self.state.borrow_mut().insert(addr, data);
            }
            Ok(())
        }

        /// this is not needed for these tests
        fn delete_state_entries(&self, _addresses: &[String]) -> Result<Vec<String>, ContextError> {
            unimplemented!()
        }

        /// this is not needed for these tests
        fn add_receipt_data(&self, _data: &[u8]) -> Result<(), ContextError> {
            unimplemented!()
        }

        /// this is not needed for these tests
        fn add_event(
            &self,
            _event_type: String,
            _attributes: Vec<(String, String)>,
            _data: &[u8],
        ) -> Result<(), ContextError> {
            unimplemented!()
        }
    }

    #[test]
    // Test that if a mfg_batch does not exist in state, None is returned
    fn test_get_mfg_batch_none() {
        let mut transaction_context = MockTransactionContext::default();
        let state = MfgBatchState::new(&mut transaction_context);

        let result = state.get_mfg_batch("not_a_mfg_batch").unwrap();
        assert!(result.is_none())
    }

    #[test]
    // Test that a mfg_batch can be added to state
    fn test_set_mfg_batch() {
        let mut transaction_context = MockTransactionContext::default();
        let state = MfgBatchState::new(&mut transaction_context);

        assert!(state.set_mfg_batch(mfg_batch_ID, make_mfg_batch()).is_ok());
        let result = state.get_mfg_batch(mfg_batch_ID).unwrap();
        assert_eq!(result, Some(make_mfg_batch()));
    }

    fn make_mfg_batch() -> mfg_batch {
        mfg_batchBuilder::new()
            .with_mfg_batch_id(mfg_batch_ID.to_string())
            .with_owner("some_owner".to_string())
            .with_mfg_batch_namespace(MfgBatchNamespace::Gs1)
            .with_properties(make_properties())
            .build()
            .expect("Failed to build new_mfg_batch")
    }

    fn make_properties() -> Vec<PropertyValue> {
        let property_value_description = PropertyValueBuilder::new()
            .with_name("description".into())
            .with_data_type(DataType::String)
            .with_string_value("This is a mfg_batch description".into())
            .build()
            .unwrap();
        let property_value_price = PropertyValueBuilder::new()
            .with_name("price".into())
            .with_data_type(DataType::Number)
            .with_number_value(3)
            .build()
            .unwrap();

        vec![
            property_value_description.clone(),
            property_value_price.clone(),
        ]
    }
}
*/
