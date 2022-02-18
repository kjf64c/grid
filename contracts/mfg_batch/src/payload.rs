// MFG_BATCH::Payload.rs
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
    } else {
        use sawtooth_sdk::processor::handler::ApplyError;
    }
}

use grid_sdk::protocol::product::payload::{Action, MfgBatchCreateAction, MfgBatchPayload};

pub fn validate_payload(payload: &MfgBatchPayload) -> Result<(), ApplyError> {
    validate_timestamp(*payload.timestamp())?;
    match payload.action() {
        Action::ProductCreate(action_payload) => validate_mfg_batch_create_action(action_payload),
        _ => Ok(()),
    }
}

fn validate_mfg_batch_create_action(
    mfg_batch_create_action: &MfgBatchCreateAction,
) -> Result<(), ApplyError> {
    if mfg_batch_create_action.mfg_batch_id() == "" {
        return Err(ApplyError::InvalidTransaction(String::from(
            "mfg_batch_id cannot be empty string",
        )));
    }
    if mfg_batch_create_action.owner() == "" {
        return Err(ApplyError::InvalidTransaction(String::from(
            "Owner cannot be empty string",
        )));
    }
    Ok(())
}

fn validate_timestamp(timestamp: u64) -> Result<(), ApplyError> {
    match timestamp {
        0 => Err(ApplyError::InvalidTransaction(String::from(
            "Timestamp is not set",
        ))),
        _ => Ok(()),
    }
}



/*

#[cfg(test)]
mod tests {
    use super::*;

    use grid_sdk::protos::product_payload::{
        MfgBatchCreateAction as MfgBatchCreateActionProto, MfgBatchPayload as MfgBatchPayloadProto,
        MfgBatchPayload_Action as ActionProto,
    };
    use grid_sdk::protos::product_state::Product_ProductNamespace;
    use grid_sdk::protos::IntoNative;

    #[test]
    /// Test that an ok is returned if the payload with MfgBatchCreateAction is valid. This test
    /// needs to use the proto directly originally to be able to mimic the scenarios possbile
    /// from creating a MfgBatchCreateAction from bytes. This is because the
    /// MfgBatchCreateActionBuilder protects from building certain invalid payloads.
    fn test_validate_payload_valid() {
        let mut payload_proto = MfgBatchPayloadProto::new();
        payload_proto.set_action(ActionProto::PRODUCT_CREATE);
        payload_proto.set_timestamp(2);
        let mut action = MfgBatchCreateActionProto::new();
        action.set_mfg_batch_id("688955434684".to_string());
        action.set_owner("my_owner".to_string());
        action.set_product_namespace(Product_ProductNamespace::GS1);
        payload_proto.set_product_create(action);
        let payload = payload_proto.clone().into_native().unwrap();
        assert!(
            validate_payload(&payload).is_ok(),
            "Payload should be valid"
        );
    }

    #[test]
    /// Test that an error is returned if the payload with MfgBatchCreateAction is missing the
    /// mfg_batch_id. This test needs to use the proto directly originally to be able to mimic the
    /// scenarios possbile from creating a MfgBatchCreateAction from bytes. This is because the
    /// MfgBatchCreateActionBuilder protects from building certain invalid payloads.
    fn test_validate_payload_mfg_batch_id_missing() {
        let mut payload_proto = MfgBatchPayloadProto::new();

        payload_proto.set_action(ActionProto::PRODUCT_CREATE);
        payload_proto.set_timestamp(2);
        let mut action = MfgBatchCreateActionProto::new();
        action.set_product_namespace(Product_ProductNamespace::GS1);
        payload_proto.set_product_create(action.clone());
        let payload = payload_proto.clone().into_native().unwrap();
        match validate_payload(&payload) {
            Ok(_) => panic!("Payload missing mfg_batch_id, should return error"),
            Err(err) => {
                assert!(err
                    .to_string()
                    .contains("mfg_batch_id cannot be empty string"));
            }
        }
    }

    #[test]
    /// Test that an error is returned if the payload with MfgBatchCreateAction is missing the
    /// owner. This test needs to use the proto directly originally to be able to mimic the
    /// scenarios possbile from creating a MfgBatchCreateAction from bytes. This is because the
    /// MfgBatchCreateActionBuilder protects from building certain invalid payloads.
    fn test_validate_payload_owner_missing() {
        let mut payload_proto = MfgBatchPayloadProto::new();

        payload_proto.set_action(ActionProto::PRODUCT_CREATE);
        payload_proto.set_timestamp(2);
        let mut action = MfgBatchCreateActionProto::new();
        action.set_product_namespace(Product_ProductNamespace::GS1);
        action.set_mfg_batch_id("688955434684".to_string());
        payload_proto.set_product_create(action.clone());
        let payload = payload_proto.clone().into_native().unwrap();
        match validate_payload(&payload) {
            Ok(_) => panic!("Payload missing owner, should return error"),
            Err(err) => {
                assert!(err.to_string().contains("Owner cannot be empty string"));
            }
        }
    }
}
*/