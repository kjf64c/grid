// MFG_BATCH::handlers.rs
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


// MFG_BTCH
cfg_if! {
    if #[cfg(target_arch = "wasm32")] {
        use sabre_sdk::ApplyError;
        use sabre_sdk::TransactionContext;
        use sabre_sdk::TransactionHandler;
        use sabre_sdk::TpProcessRequest;
        use sabre_sdk::{WasmPtr, execute_entrypoint};
    } else {
        use sawtooth_sdk::processor::handler::ApplyError;
        use sawtooth_sdk::processor::handler::TransactionContext;
        use sawtooth_sdk::processor::handler::TransactionHandler;
        use sawtooth_sdk::messages::processor::TpProcessRequest;
    }
}

use grid_sdk::{
    pike::permissions::PermissionChecker,
    mfg_batch::addressing::GRID_NAMESPACE,
    protocol::mfg_batch::{
        payload::{
            Action, MfgBatchCreateAction, MfgBatchDeleteAction, MfgBatchPayload, MfgBatchUpdateAction,
        },
        state::{MfgBatchBuilder, MfgBatchNamespace},
    },
    protos::FromBytes,
};

use crate::payload::validate_payload;
use crate::permissions::{permission_to_perm_string, Permission};
use crate::state::MfgBatchState;
use crate::validation::validate_gtin;

#[cfg(target_arch = "wasm32")]
// Sabre apply must return a bool
fn apply(
    request: &TpProcessRequest,
    context: &mut dyn TransactionContext,
) -> Result<bool, ApplyError> {
    let handler = MfgBatchTransactionHandler::new();
    match handler.apply(request, context) {
        Ok(_) => Ok(true),
        Err(err) => {
            info!("{} received {}", handler.family_name(), err);
            Err(err)
        }
    }
}

#[cfg(target_arch = "wasm32")]
#[no_mangle]
pub unsafe fn entrypoint(payload: WasmPtr, signer: WasmPtr, signature: WasmPtr) -> i32 {
    execute_entrypoint(payload, signer, signature, apply)
}

#[derive(Default)]
pub struct MfgBatchTransactionHandler {
    family_name: String,
    family_versions: Vec<String>,
    namespaces: Vec<String>,
}

impl MfgBatchTransactionHandler {
    pub fn new() -> MfgBatchTransactionHandler {
        MfgBatchTransactionHandler {
            family_name: "grid_mfg_batch".to_string(),
            family_versions: vec!["1".to_string()],
            namespaces: vec![GRID_NAMESPACE.to_string()],
        }
    }

    fn create_mfg_batch(
        &self,
        payload: &MfgBatchCreateAction,
        state: &mut MfgBatchState,
        signer: &str,
        perm_checker: &PermissionChecker,
    ) -> Result<(), ApplyError> {
        let mfg_batch_id = payload.mfg_batch_id();
        let owner = payload.owner();
        let mfg_batch_namespace = payload.mfg_batch_namespace();
        let properties = payload.properties();

        // Check signing agent's permission
        check_permission(
            perm_checker,
            signer,
            &permission_to_perm_string(Permission::CanCreateMfgBatch),
            owner,
        )?;

        // Check if mfg_batch exists in state
        if state.get_mfg_batch(mfg_batch_id)?.is_some() {
            return Err(ApplyError::InvalidTransaction(format!(
                "Product already exists: {}",
                mfg_batch_id,
            )));
        }

        // Check if the mfg_batch namespace is a GS1 mfg_batch
        if mfg_batch_namespace != &MfgBatchNamespace::Gs1 {
            return Err(ApplyError::InvalidTransaction(
                "Invalid mfg_batch namespace enum for mfg_batch".to_string(),
            ));
        }

        // Check if mfg_batch mfg_batch_id is a valid gtin
        if let Err(e) = validate_gtin(mfg_batch_id) {
            return Err(ApplyError::InvalidTransaction(e.to_string()));
        }

        // Check that the organization ID exists in state
        let org = match state.get_organization(payload.owner())? {
            Some(org) => org,
            None => {
                return Err(ApplyError::InvalidTransaction(format!(
                    "The Agent's organization does not exist: {}",
                    signer,
                )));
            }
        };

        /* Check if the agents organization contain GS1 Company Prefix key in its alternate IDs
        (gs1_company_prefix), and the prefix must match the company prefix in the mfg_batch_id */
        if payload.mfg_batch_namespace() == &MfgBatchNamespace::Gs1 {
            let metadata = org.alternate_ids().to_vec();
            let gs1_company_prefix = match metadata
                .iter()
                .find(|p| p.id_type() == "gs1_company_prefix")
            {
                Some(gs1_company_prefix) => gs1_company_prefix,
                None => {
                    return Err(ApplyError::InvalidTransaction(format!(
                        "The agents organization does not have the gs1_company_prefix prefix: {:?}",
                        org.alternate_ids()
                    )));
                }
            };
            // If the gtin identifer does not contain the organizations gs1 prefix
            if !mfg_batch_id.contains(gs1_company_prefix.id()) {
                return Err(ApplyError::InvalidTransaction(format!(
                    "The agents organization does not own the GS1 company prefix in the GTIN mfg_batch_id: {:?}",
                    org.alternate_ids()
                )));
            }
        }

        if payload.mfg_batch_namespace() == &MfgBatchNamespace::Gs1 {
            // Check if gs1 schema exists
            let schema = if let Some(schema) = state.get_schema("gs1_mfg_batch")? {
                schema
            } else {
                return Err(ApplyError::InvalidTransaction(
                    "gs1_mfg_batch schema has not been defined".into(),
                ));
            };

            // Check if properties in mfg_batch are all a part of the gs1 schema
            for property in payload.properties() {
                if schema
                    .properties()
                    .iter()
                    .all(|p| p.name() != property.name())
                {
                    return Err(ApplyError::InvalidTransaction(format!(
                        "{} is not a property that is defined by the gs1 schema",
                        property.name()
                    )));
                }
            }

            // Check if property has all required fields
            for property in schema.properties().iter().filter(|p| *p.required()) {
                if !payload
                    .properties()
                    .iter()
                    .any(|p| p.name() == property.name() && p.data_type() == property.data_type())
                {
                    return Err(ApplyError::InvalidTransaction(format!(
                        "Missing required field '{}' of type '{:?}'",
                        property.name(),
                        property.data_type()
                    )));
                }
            }
        }

        let new_mfg_batch = MfgBatchBuilder::new()
            .with_mfg_batch_id(mfg_batch_id.to_string())
            .with_owner(owner.to_string())
            .with_mfg_batch_namespace(mfg_batch_namespace.clone())
            .with_properties(properties.to_vec())
            .build()
            .map_err(|err| {
                ApplyError::InvalidTransaction(format!("Cannot build mfg_batch: {}", err))
            })?;

        state.set_mfg_batch(mfg_batch_id, new_mfg_batch)?;

        Ok(())
    }

    fn update_mfg_batch(
        &self,
        payload: &MfgBatchUpdateAction,
        state: &mut MfgBatchState,
        signer: &str,
        perm_checker: &PermissionChecker,
    ) -> Result<(), ApplyError> {
        let mfg_batch_id = payload.mfg_batch_id();
        let mfg_batch_namespace = payload.mfg_batch_namespace();
        let properties = payload.properties();

        // Check if the mfg_batch namespace is a GS1 mfg_batch
        if mfg_batch_namespace != &MfgBatchNamespace::Gs1 {
            return Err(ApplyError::InvalidTransaction(
                "Invalid mfg_batch namespace enum for mfg_batch".to_string(),
            ));
        }

        // Check if mfg_batch exists
        let mfg_batch = match state.get_mfg_batch(mfg_batch_id) {
            Ok(Some(mfg_batch)) => Ok(mfg_batch),
            Ok(None) => Err(ApplyError::InvalidTransaction(format!(
                "No mfg_batch exists: {}",
                mfg_batch_id
            ))),
            Err(err) => Err(err),
        }?;

        // Check signing agent's permission
        check_permission(
            perm_checker,
            signer,
            &permission_to_perm_string(Permission::CanUpdateMfgBatch),
            mfg_batch.owner(),
        )?;

        // Check if mfg_batch mfg_batch_id is a valid gtin
        if let Err(e) = validate_gtin(mfg_batch_id) {
            return Err(ApplyError::InvalidTransaction(e.to_string()));
        }

        if payload.mfg_batch_namespace() == &MfgBatchNamespace::Gs1 {
            // Check if gs1 schema exists
            let schema = if let Some(schema) = state.get_schema("gs1_mfg_batch")? {
                schema
            } else {
                return Err(ApplyError::InvalidTransaction(
                    "gs1_mfg_batch schema has not been defined".into(),
                ));
            };

            // Check if properties in mfg_batch are all a part of the gs1 schema
            for property in payload.properties() {
                if schema
                    .properties()
                    .iter()
                    .all(|p| p.name() != property.name())
                {
                    return Err(ApplyError::InvalidTransaction(format!(
                        "{} is not a property that is defined by the gs1 schema",
                        property.name()
                    )));
                }
            }

            // Check if property has all required fields
            for property in schema.properties().iter().filter(|p| *p.required()) {
                if !payload
                    .properties()
                    .iter()
                    .any(|p| p.name() == property.name() && p.data_type() == property.data_type())
                {
                    return Err(ApplyError::InvalidTransaction(format!(
                        "Missing required field '{}' of type '{:?}'",
                        property.name(),
                        property.data_type()
                    )));
                }
            }
        }

        // Handle updating the mfg_batch
        let updated_mfg_batch = MfgBatchBuilder::new()
            .with_mfg_batch_id(mfg_batch_id.to_string())
            .with_owner(mfg_batch.owner().to_string())
            .with_mfg_batch_namespace(mfg_batch_namespace.clone())
            .with_properties(properties.to_vec())
            .build()
            .map_err(|err| {
                ApplyError::InvalidTransaction(format!("Cannot build mfg_batch: {}", err))
            })?;

        state.set_mfg_batch(mfg_batch_id, updated_mfg_batch)?;

        Ok(())
    }

    fn delete_mfg_batch(
        &self,
        payload: &MfgBatchDeleteAction,
        state: &mut MfgBatchState,
        signer: &str,
        perm_checker: &PermissionChecker,
    ) -> Result<(), ApplyError> {
        let mfg_batch_id = payload.mfg_batch_id();
        let mfg_batch_namespace = payload.mfg_batch_namespace();

        // Check if the mfg_batch namespace is a GS1 mfg_batch
        if mfg_batch_namespace != &MfgBatchNamespace::Gs1 {
            return Err(ApplyError::InvalidTransaction(
                "Invalid mfg_batch namespace enum for mfg_batch".to_string(),
            ));
        }

        // Check if mfg_batch exists in state
        let mfg_batch = match state.get_mfg_batch(mfg_batch_id) {
            Ok(Some(mfg_batch)) => Ok(mfg_batch),
            Ok(None) => Err(ApplyError::InvalidTransaction(format!(
                "No mfg_batch exists: {}",
                mfg_batch_id
            ))),
            Err(err) => Err(err),
        }?;

        // Check signing agent's permission
        check_permission(
            perm_checker,
            signer,
            &permission_to_perm_string(Permission::CanDeleteMfgBatch),
            mfg_batch.owner(),
        )?;

        // Check if mfg_batch mfg_batch_id is a valid gtin
        if let Err(e) = validate_gtin(mfg_batch_id) {
            return Err(ApplyError::InvalidTransaction(e.to_string()));
        }

        // Delete the mfg_batch
        state.remove_mfg_batch(mfg_batch_id)?;
        Ok(())
    }
}

impl TransactionHandler for MfgBatchTransactionHandler {
    fn family_name(&self) -> String {
        self.family_name.clone()
    }

    fn family_versions(&self) -> Vec<String> {
        self.family_versions.clone()
    }

    fn namespaces(&self) -> Vec<String> {
        self.namespaces.clone()
    }

    fn apply(
        &self,
        request: &TpProcessRequest,
        context: &mut dyn TransactionContext,
    ) -> Result<(), ApplyError> {
        let payload = MfgBatchPayload::from_bytes(request.get_payload()).map_err(|err| {
            ApplyError::InvalidTransaction(format!("Cannot build manufacturig batch payload: {}", err))
        })?;

        validate_payload(&payload)?;

        info!(
            "Grid Manufactured Batch Payload {:?} {}",
            payload.action(),
            payload.timestamp(),
        );

        let signer = request.get_header().get_signer_public_key();
        let mut state = MfgBatchState::new(context);
        let perm_checker = PermissionChecker::new(context);

        match payload.action() {
            Action::MfgBatchCreate(create_mfg_batch_payload) => {
                self.create_mfg_batch(create_mfg_batch_payload, &mut state, signer, &perm_checker)?
            }
            Action::MfgBatchUpdate(update_mfg_batch_payload) => {
                self.update_mfg_batch(update_mfg_batch_payload, &mut state, signer, &perm_checker)?
            }
            Action::MfgBatchDelete(delete_mfg_batch_payload) => {
                self.delete_mfg_batch(delete_mfg_batch_payload, &mut state, signer, &perm_checker)?
            }
        }
        Ok(())
    }
}

fn check_permission(
    perm_checker: &PermissionChecker,
    signer: &str,
    permission: &str,
    record_owner: &str,
) -> Result<(), ApplyError> {
    match perm_checker.has_permission(signer, permission, record_owner) {
        Ok(true) => Ok(()),
        Ok(false) => Err(ApplyError::InvalidTransaction(format!(
            "The signer \"{}\" does not have the \"{}\" permission for org \"{}\"",
            signer, permission, record_owner
        ))),
        Err(e) => Err(ApplyError::InvalidTransaction(format!(
            "Permission check failed: {}",
            e
        ))),
    }
}

/*

#[cfg(test)]
mod tests {
    use super::*;

    use std::cell::RefCell;
    use std::collections::HashMap;

    use grid_sdk::{
        pike::addressing::{
            compute_agent_address, compute_organization_address, compute_role_address,
        },
        mfg_batch::addressing::compute_gs1_mfg_batch_address,
        protocol::{
            pike::state::{
                AgentBuilder, AgentListBuilder, AlternateIdBuilder, OrganizationBuilder,
                OrganizationListBuilder, RoleBuilder, RoleListBuilder,
            },
            mfg_batch::{
                payload::{
                    MfgBatchCreateAction, MfgBatchCreateActionBuilder, MfgBatchDeleteAction,
                    MfgBatchDeleteActionBuilder, MfgBatchUpdateAction, MfgBatchUpdateActionBuilder,
                },
                state::{Product, MfgBatchBuilder, ProductListBuilder, MfgBatchNamespace},
            },
            schema::state::{
                DataType, PropertyDefinitionBuilder, PropertyValue, PropertyValueBuilder,
                SchemaBuilder, SchemaListBuilder,
            },
        },
        protos::IntoBytes,
        schema::addressing::compute_schema_address,
    };

    use sawtooth_sdk::processor::handler::{ContextError, TransactionContext};

    const AGENT_ORG_ID: &str = "test_org";
    const PUBLIC_KEY: &str = "test_public_key";
    const ROLE_NAME: &str = "mfg_batch_roles";
    const MFG_BATCH_ID: &str = "688955434684";
    const PRODUCT_2_ID: &str = "9781981855728";

    #[derive(Default, Debug)]
    /// A MockTransactionContext that can be used to test MfgBatchState
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

    impl MockTransactionContext {
        fn add_agent(&self, public_key: &str) {
            let builder = AgentBuilder::new();
            let agent = builder
                .with_org_id(AGENT_ORG_ID.to_string())
                .with_public_key(public_key.to_string())
                .with_active(true)
                .with_roles(vec![ROLE_NAME.to_string()])
                .build()
                .unwrap();

            let builder = AgentListBuilder::new();
            let agent_list = builder.with_agents(vec![agent.clone()]).build().unwrap();
            let agent_bytes = agent_list.into_bytes().unwrap();
            let agent_address = compute_agent_address(public_key);
            self.set_state_entry(agent_address, agent_bytes).unwrap();
        }

        fn add_agent_without_roles(&self, public_key: &str) {
            let builder = AgentBuilder::new();
            let agent = builder
                .with_org_id(AGENT_ORG_ID.to_string())
                .with_public_key(public_key.to_string())
                .with_active(true)
                .with_roles([].to_vec())
                .build()
                .unwrap();

            let builder = AgentListBuilder::new();
            let agent_list = builder.with_agents(vec![agent.clone()]).build().unwrap();
            let agent_bytes = agent_list.into_bytes().unwrap();
            let agent_address = compute_agent_address(public_key);
            self.set_state_entry(agent_address, agent_bytes).unwrap();
        }

        fn add_role(&self) {
            let builder = RoleBuilder::new();
            let role = builder
                .with_org_id(AGENT_ORG_ID.to_string())
                .with_name(ROLE_NAME.to_string())
                .with_description("role description".to_string())
                .with_permissions(vec![
                    "mfg_batch::can-create-mfg_batch".to_string(),
                    "mfg_batch::can-update-mfg_batch".to_string(),
                    "mfg_batch::can-delete-mfg_batch".to_string(),
                ])
                .build()
                .unwrap();

            let builder = RoleListBuilder::new();
            let role_list = builder.with_roles(vec![role.clone()]).build().unwrap();
            let role_bytes = role_list.into_bytes().unwrap();
            let role_address = compute_role_address(ROLE_NAME, AGENT_ORG_ID);
            self.set_state_entry(role_address, role_bytes).unwrap();
        }

        fn add_org(&self, org_id: &str) {
            // Products can only be created when there is a gs1 prefix
            // within the mfg_batch organization's metadata
            let alternate_id = AlternateIdBuilder::new()
                .with_id_type("gs1_company_prefix".to_string())
                .with_id("6889".to_string())
                .build()
                .unwrap();
            let builder = OrganizationBuilder::new();
            let org = builder
                .with_org_id(org_id.to_string())
                .with_name("test_org_name".to_string())
                .with_locations(vec!["test".to_string()])
                .with_alternate_ids(vec![alternate_id.clone()])
                .build()
                .unwrap();

            let builder = OrganizationListBuilder::new();
            let org_list = builder
                .with_organizations(vec![org.clone()])
                .build()
                .unwrap();
            let org_bytes = org_list.into_bytes().unwrap();
            let org_address = compute_organization_address(org_id);
            self.set_state_entry(org_address, org_bytes).unwrap();
        }

        fn add_org_without_gs1_prefix(&self, org_id: &str) {
            let builder = OrganizationBuilder::new();
            let org = builder
                .with_org_id(org_id.to_string())
                .with_name("test_org_name".to_string())
                .with_locations(vec!["test".to_string()])
                .build()
                .unwrap();

            let builder = OrganizationListBuilder::new();
            let org_list = builder
                .with_organizations(vec![org.clone()])
                .build()
                .unwrap();
            let org_bytes = org_list.into_bytes().unwrap();
            let org_address = compute_organization_address(org_id);
            self.set_state_entry(org_address, org_bytes).unwrap();
        }

        fn add_mfg_batch(&self, prod_id: &str) {
            let mfg_batch_list = ProductListBuilder::new()
                .with_mfg_batchs(vec![make_mfg_batch()])
                .build()
                .unwrap();
            let mfg_batch_bytes = mfg_batch_list.into_bytes().unwrap();
            let mfg_batch_address = compute_gs1_mfg_batch_address(prod_id);
            self.set_state_entry(mfg_batch_address, mfg_batch_bytes)
                .unwrap();
        }

        fn add_mfg_batchs(&self, mfg_batch_ids: &[&str]) {
            let mfg_batch_list = ProductListBuilder::new()
                .with_mfg_batchs(make_mfg_batchs(mfg_batch_ids))
                .build()
                .unwrap();
            let mfg_batch_list_bytes = mfg_batch_list.into_bytes().unwrap();
            let mfg_batch_list_bytes_copy = mfg_batch_list_bytes.clone();
            let mfg_batch_1_address = compute_gs1_mfg_batch_address(PRODUCT_ID);
            let mfg_batch_2_address = compute_gs1_mfg_batch_address(PRODUCT_2_ID);
            self.set_state_entries(vec![
                (mfg_batch_1_address, mfg_batch_list_bytes),
                (mfg_batch_2_address, mfg_batch_list_bytes_copy),
            ])
            .unwrap();
        }

        fn add_gs1_schema(&self) {
            let properties = vec![
                PropertyDefinitionBuilder::new()
                    .with_name("counter".into())
                    .with_data_type(DataType::Number)
                    .with_number_exponent(1)
                    .with_required(true)
                    .build()
                    .unwrap(),
                PropertyDefinitionBuilder::new()
                    .with_name("description".into())
                    .with_data_type(DataType::String)
                    .with_required(true)
                    .build()
                    .unwrap(),
            ];

            let schema = SchemaBuilder::new()
                .with_name("gs1_mfg_batch".into())
                .with_description("GS1 mfg_batch".into())
                .with_owner(AGENT_ORG_ID.to_string())
                .with_properties(properties)
                .build()
                .unwrap();

            let schema_list = SchemaListBuilder::new()
                .with_schemas(vec![schema])
                .build()
                .unwrap();

            self.set_state_entries(vec![(
                compute_schema_address("gs1_mfg_batch"),
                schema_list.into_bytes().unwrap(),
            )])
            .unwrap();
        }
    }

    #[test]
    /// Test that if ProductCreationAction is valid an OK is returned and a new Product is added to state
    fn test_create_mfg_batch_handler_valid() {
        let transaction_context = MockTransactionContext::default();
        transaction_context.add_agent(PUBLIC_KEY);
        transaction_context.add_org(AGENT_ORG_ID);
        transaction_context.add_role();
        transaction_context.add_gs1_schema();
        let perm_checker = PermissionChecker::new(&transaction_context);
        let mut state = MfgBatchState::new(&transaction_context);

        let transaction_handler = MfgBatchTransactionHandler::new();
        let mfg_batch_create_action = make_mfg_batch_create_action();

        match transaction_handler.create_mfg_batch(
            &mfg_batch_create_action,
            &mut state,
            PUBLIC_KEY,
            &perm_checker,
        ) {
            Ok(()) => {}
            Err(ApplyError::InternalError(err)) => {
                assert_eq!("Failed to check permissions: InvalidPublicKey: The signer is not an Agent: test_public_key", err);
            }
            Err(err) => panic!("Should have gotten internal error but got {}", err),
        }

        let mfg_batch = state
            .get_mfg_batch(PRODUCT_ID)
            .expect("Failed to fetch mfg_batch")
            .expect("No mfg_batch found");

        assert_eq!(mfg_batch, make_mfg_batch());
    }

    #[test]
    /// Test that ProductCreationAction is invalid if the agent's org does not exist.
    fn test_create_mfg_batch_org_does_not_exist() {
        let transaction_context = MockTransactionContext::default();
        transaction_context.add_agent(PUBLIC_KEY);
        let perm_checker = PermissionChecker::new(&transaction_context);
        let mut state = MfgBatchState::new(&transaction_context);

        let transaction_handler = MfgBatchTransactionHandler::new();
        let mfg_batch_create_action = make_mfg_batch_create_action();

        match transaction_handler.create_mfg_batch(
            &mfg_batch_create_action,
            &mut state,
            PUBLIC_KEY,
            &perm_checker,
        ) {
            Ok(()) => panic!(
                "Agent's organization should not exist, InvalidTransaction should be returned"
            ),
            Err(ApplyError::InvalidTransaction(err)) => {
                assert_eq!("The signer \"test_public_key\" does not have the \"mfg_batch::can-create-mfg_batch\" permission for org \"test_org\"", err);
            }
            Err(err) => panic!("Should have gotten invalid error but go {}", err),
        }
    }

    #[test]
    /// Test that ProductCreationAction is invalid if the agent's org does not contain the gs1 prefix.
    fn test_create_mfg_batch_org_without_gs1_prefix() {
        let transaction_context = MockTransactionContext::default();
        transaction_context.add_agent(PUBLIC_KEY);
        transaction_context.add_role();
        transaction_context.add_gs1_schema();
        transaction_context.add_org_without_gs1_prefix(AGENT_ORG_ID);
        let perm_checker = PermissionChecker::new(&transaction_context);
        let mut state = MfgBatchState::new(&transaction_context);

        let transaction_handler = MfgBatchTransactionHandler::new();
        let mfg_batch_create_action = make_mfg_batch_create_action();

        match transaction_handler.create_mfg_batch(
            &mfg_batch_create_action,
            &mut state,
            PUBLIC_KEY,
            &perm_checker
        ) {
            Ok(()) => panic!("Agent's organization should not have a gs1 prefix key, InvalidTransaction should be returned"),
            Err(ApplyError::InvalidTransaction(err)) => {
                assert!(err.contains("The agents organization does not have the gs1_company_prefix prefix: []"));
            }
            Err(err) => panic!("Should have gotten invalid error but go {}", err),
        }
    }

    #[test]
    /// Test that ProductCreationAction is invalid if the a mfg_batch with the same id
    /// already exists.
    fn test_create_mfg_batch_already_exist() {
        let transaction_context = MockTransactionContext::default();
        transaction_context.add_agent(PUBLIC_KEY);
        transaction_context.add_org(AGENT_ORG_ID);
        transaction_context.add_role();
        transaction_context.add_mfg_batch(PRODUCT_ID);
        let perm_checker = PermissionChecker::new(&transaction_context);
        let mut state = MfgBatchState::new(&transaction_context);

        let transaction_handler = MfgBatchTransactionHandler::new();
        let mfg_batch_create_action = make_mfg_batch_create_action();

        match transaction_handler.create_mfg_batch(
            &mfg_batch_create_action,
            &mut state,
            PUBLIC_KEY,
            &perm_checker,
        ) {
            Ok(()) => panic!("Product should not exist, InvalidTransaction should be returned"),
            Err(ApplyError::InvalidTransaction(err)) => {
                assert!(err.contains(&format!("Product already exists: {}", PRODUCT_ID)));
            }
            Err(err) => panic!("Should have gotten invalid error but go {}", err),
        }
    }

    #[test]
    /// Test that if MfgBatchUpdateAction is valid an OK is returned and a Product is updated in state
    fn test_update_mfg_batch_handler_valid() {
        let transaction_context = MockTransactionContext::default();
        transaction_context.add_agent(PUBLIC_KEY);
        transaction_context.add_org(AGENT_ORG_ID);
        transaction_context.add_role();
        transaction_context.add_gs1_schema();
        transaction_context.add_mfg_batch(PRODUCT_ID);
        let perm_checker = PermissionChecker::new(&transaction_context);
        let mut state = MfgBatchState::new(&transaction_context);

        let transaction_handler = MfgBatchTransactionHandler::new();
        let mfg_batch_update_action = make_mfg_batch_update_action();

        match transaction_handler.update_mfg_batch(
            &mfg_batch_update_action,
            &mut state,
            PUBLIC_KEY,
            &perm_checker,
        ) {
            Ok(()) => {}
            Err(ApplyError::InternalError(err)) => {
                assert_eq!("Failed to check permissions: InvalidPublicKey: The signer is not an Agent: test_public_key", err);
            }
            Err(err) => panic!("Should have gotten internal error but got {}", err),
        }

        let mfg_batch = state
            .get_mfg_batch(PRODUCT_ID)
            .expect("Failed to fetch mfg_batch")
            .expect("No mfg_batch found");

        assert_eq!(mfg_batch, make_updated_mfg_batch());
    }

    #[test]
    /// Test that MfgBatchUpdateAction is invalid if there is no mfg_batch to update
    fn test_update_mfg_batch_that_does_not_exist() {
        let transaction_context = MockTransactionContext::default();
        transaction_context.add_agent(PUBLIC_KEY);
        let perm_checker = PermissionChecker::new(&transaction_context);
        let mut state = MfgBatchState::new(&transaction_context);

        let transaction_handler = MfgBatchTransactionHandler::new();
        let mfg_batch_update_action = make_mfg_batch_update_action();

        match transaction_handler.update_mfg_batch(
            &mfg_batch_update_action,
            &mut state,
            PUBLIC_KEY,
            &perm_checker,
        ) {
            Ok(()) => panic!("Product should not exist, InvalidTransaction should be returned"),
            Err(ApplyError::InvalidTransaction(err)) => {
                assert!(err.contains(&format!("No mfg_batch exists: {}", PRODUCT_ID)));
            }
            Err(err) => panic!("Should have gotten invalid error but go {}", err),
        }
    }

    #[test]
    /// Test that if MfgBatchDeleteAction is valid an OK is returned and a Product is deleted from state
    fn test_delete_mfg_batch_handler_valid() {
        let transaction_context = MockTransactionContext::default();
        transaction_context.add_agent(PUBLIC_KEY);
        transaction_context.add_org(AGENT_ORG_ID);
        transaction_context.add_role();
        transaction_context.add_gs1_schema();
        transaction_context.add_mfg_batchs(&vec![PRODUCT_ID, PRODUCT_2_ID]);
        let perm_checker = PermissionChecker::new(&transaction_context);
        let mut state = MfgBatchState::new(&transaction_context);

        let transaction_handler = MfgBatchTransactionHandler::new();
        let mfg_batch_delete_action = make_mfg_batch_delete_action(PRODUCT_ID);

        assert!(transaction_handler
            .delete_mfg_batch(
                &mfg_batch_delete_action,
                &mut state,
                PUBLIC_KEY,
                &perm_checker
            )
            .is_ok());

        let mfg_batch = state.get_mfg_batch(PRODUCT_ID).expect("No mfg_batch found");

        assert_eq!(mfg_batch, None);
    }

    #[test]
    /// Test that if MfgBatchDeleteAction is valid an OK is returned and a
    /// second mfg_batch is deleted from state
    fn test_delete_second_mfg_batch_handler_valid() {
        let transaction_context = MockTransactionContext::default();
        transaction_context.add_agent(PUBLIC_KEY);
        transaction_context.add_org(AGENT_ORG_ID);
        transaction_context.add_role();
        transaction_context.add_gs1_schema();
        transaction_context.add_mfg_batchs(&vec![PRODUCT_ID, PRODUCT_2_ID]);
        let perm_checker = PermissionChecker::new(&transaction_context);
        let mut state = MfgBatchState::new(&transaction_context);

        let transaction_handler = MfgBatchTransactionHandler::new();
        let mfg_batch_delete_action = make_mfg_batch_delete_action(PRODUCT_2_ID);

        assert!(transaction_handler
            .delete_mfg_batch(
                &mfg_batch_delete_action,
                &mut state,
                PUBLIC_KEY,
                &perm_checker,
            )
            .is_ok());

        let mfg_batch = state.get_mfg_batch(PRODUCT_2_ID).expect("No mfg_batch found");

        assert_eq!(mfg_batch, None);
    }

    #[test]
    /// Test that MfgBatchDeleteAction is invalid if the agent does not have can_delete_mfg_batch role
    fn test_delete_mfg_batch_agent_without_roles() {
        let transaction_context = MockTransactionContext::default();
        transaction_context.add_agent_without_roles(PUBLIC_KEY);
        transaction_context.add_org(AGENT_ORG_ID);
        transaction_context.add_mfg_batchs(&vec![PRODUCT_ID, PRODUCT_2_ID]);
        let perm_checker = PermissionChecker::new(&transaction_context);
        let mut state = MfgBatchState::new(&transaction_context);

        let transaction_handler = MfgBatchTransactionHandler::new();
        let mfg_batch_delete_action = make_mfg_batch_delete_action(PRODUCT_ID);

        match transaction_handler.delete_mfg_batch(
            &mfg_batch_delete_action,
            &mut state,
            PUBLIC_KEY,
            &perm_checker,
        ) {
            Ok(()) => panic!(
                "Agent should not have can_delete_mfg_batch role, InvalidTransaction should be returned"
            ),
            Err(ApplyError::InvalidTransaction(err)) => {
                assert_eq!(
                    "The signer \"test_public_key\" does not have the \"mfg_batch::can-delete-mfg_batch\" permission for org \"test_org\"",
                    err
                );
            }
            Err(err) => panic!("Should have gotten invalid error but got {}", err),
        }
    }

    #[test]
    /// Test that MfgBatchDeleteAction is invalid when deleting a non existant mfg_batch
    fn test_delete_mfg_batch_not_exists() {
        let transaction_context = MockTransactionContext::default();
        transaction_context.add_agent(PUBLIC_KEY);
        transaction_context.add_org(AGENT_ORG_ID);
        transaction_context.add_mfg_batchs(&vec![PRODUCT_ID, PRODUCT_2_ID]);
        let perm_checker = PermissionChecker::new(&transaction_context);
        let mut state = MfgBatchState::new(&transaction_context);

        let transaction_handler = MfgBatchTransactionHandler::new();
        let mfg_batch_delete_action = make_mfg_batch_delete_action("13491387613");

        match transaction_handler.delete_mfg_batch(
            &mfg_batch_delete_action,
            &mut state,
            PUBLIC_KEY,
            &perm_checker,
        ) {
            Ok(()) => panic!("Product should not exist, InvalidTransaction should be returned"),
            Err(ApplyError::InvalidTransaction(err)) => {
                assert!(err.contains("No mfg_batch exists: 13491387613"));
            }
            Err(err) => panic!("Should have gotten invalid error but go {}", err),
        }
    }

    fn make_mfg_batch() -> Product {
        MfgBatchBuilder::new()
            .with_mfg_batch_id(PRODUCT_ID.to_string())
            .with_owner(AGENT_ORG_ID.to_string())
            .with_mfg_batch_namespace(MfgBatchNamespace::Gs1)
            .with_properties(make_properties())
            .build()
            .expect("Failed to build new_mfg_batch")
    }

    fn make_mfg_batchs(mfg_batch_ids: &[&str]) -> Vec<Product> {
        vec![
            MfgBatchBuilder::new()
                .with_mfg_batch_id(mfg_batch_ids[0].to_string())
                .with_owner(AGENT_ORG_ID.to_string())
                .with_mfg_batch_namespace(MfgBatchNamespace::Gs1)
                .with_properties(make_properties())
                .build()
                .expect("Failed to build new_mfg_batch"),
            MfgBatchBuilder::new()
                .with_mfg_batch_id(mfg_batch_ids[1].to_string())
                .with_owner(AGENT_ORG_ID.to_string())
                .with_mfg_batch_namespace(MfgBatchNamespace::Gs1)
                .with_properties(make_properties())
                .build()
                .expect("Failed to build new_mfg_batch"),
        ]
    }

    fn make_updated_mfg_batch() -> Product {
        MfgBatchBuilder::new()
            .with_mfg_batch_id(PRODUCT_ID.to_string())
            .with_owner(AGENT_ORG_ID.to_string())
            .with_mfg_batch_namespace(MfgBatchNamespace::Gs1)
            .with_properties(make_updated_properties())
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
        let property_value_counter = PropertyValueBuilder::new()
            .with_name("counter".into())
            .with_data_type(DataType::Number)
            .with_number_value(3)
            .build()
            .unwrap();

        vec![
            property_value_description.clone(),
            property_value_counter.clone(),
        ]
    }

    fn make_updated_properties() -> Vec<PropertyValue> {
        let property_value_description = PropertyValueBuilder::new()
            .with_name("description".into())
            .with_data_type(DataType::String)
            .with_string_value("This is a new mfg_batch description".into())
            .build()
            .unwrap();
        let property_value_counter = PropertyValueBuilder::new()
            .with_name("counter".into())
            .with_data_type(DataType::Number)
            .with_number_value(4)
            .build()
            .unwrap();

        vec![
            property_value_description.clone(),
            property_value_counter.clone(),
        ]
    }

    fn make_mfg_batch_create_action() -> MfgBatchCreateAction {
        MfgBatchCreateActionBuilder::new()
            .with_mfg_batch_id(PRODUCT_ID.to_string())
            .with_owner(AGENT_ORG_ID.to_string())
            .with_mfg_batch_namespace(MfgBatchNamespace::Gs1)
            .with_properties(make_properties())
            .build()
            .expect("Failed to build MfgBatchCreateAction")
    }

    fn make_mfg_batch_update_action() -> MfgBatchUpdateAction {
        MfgBatchUpdateActionBuilder::new()
            .with_mfg_batch_id(PRODUCT_ID.to_string())
            .with_mfg_batch_namespace(MfgBatchNamespace::Gs1)
            .with_properties(make_updated_properties())
            .build()
            .expect("Failed to build MfgBatchUpdateAction")
    }

    fn make_mfg_batch_delete_action(mfg_batch_id: &str) -> MfgBatchDeleteAction {
        MfgBatchDeleteActionBuilder::new()
            .with_mfg_batch_id(mfg_batch_id.to_string())
            .with_mfg_batch_namespace(MfgBatchNamespace::Gs1)
            .build()
            .expect("Failed to build MfgBatchDeleteAction")
    }
}

*/
