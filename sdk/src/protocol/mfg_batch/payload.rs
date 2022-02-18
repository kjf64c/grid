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

//! Protocol structs for Product transaction payloads

use protobuf::Message;
use protobuf::RepeatedField;

use std::error::Error as StdError;

use super::errors::BuilderError;

use crate::protocol::{mfg_batch::state::MfgBatchNamespace, schema::state::PropertyValue};
use crate::protos;
use crate::protos::{mfg_batch_payload, mfg_batch_payload::MfgBatchPayload_Action};
use crate::protos::{
    FromBytes, FromNative, FromProto, IntoBytes, IntoNative, IntoProto, ProtoConversionError,
};

/// The Product payload's action envelope
#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    MfgBatchCreate(MfgBatchCreateAction),
    MfgBatchUpdate(MfgBatchUpdateAction),
    MfgBatchDelete(MfgBatchDeleteAction),
}

/// Native representation of a Product transaction payload
#[derive(Debug, Clone, PartialEq)]
pub struct MfgBatchPayload {
    action: Action,
    timestamp: u64,
}

impl MfgBatchPayload {
    pub fn action(&self) -> &Action {
        &self.action
    }
    pub fn timestamp(&self) -> &u64 {
        &self.timestamp
    }
}

impl FromProto<protos::mfg_batch_payload::MfgBatchPayload> for MfgBatchPayload {
    fn from_proto(
        payload: protos::mfg_batch_payload::MfgBatchPayload,
    ) -> Result<Self, ProtoConversionError> {
        let action = match payload.get_action() {
            MfgBatchPayload_Action::MFG_BATCH_CREATE => Action::MfgBatchCreate(
                MfgBatchCreateAction::from_proto(payload.get_mfg_batch_create().clone())?,
            ),
            MfgBatchPayload_Action::MFG_BATCH_UPDATE => Action::MfgBatchUpdate(
                MfgBatchUpdateAction::from_proto(payload.get_mfg_batch_update().clone())?,
            ),
            MfgBatchPayload_Action::MFG_BATCH_DELETE => Action::MfgBatchDelete(
                MfgBatchDeleteAction::from_proto(payload.get_mfg_batch_delete().clone())?,
            ),
            MfgBatchPayload_Action::UNSET_ACTION => {
                return Err(ProtoConversionError::InvalidTypeError(
                    "Cannot convert MfgBatchPayload_Action with type unset".to_string(),
                ));
            }
        };
        Ok(MfgBatchPayload {
            action,
            timestamp: payload.get_timestamp(),
        })
    }
}

impl FromNative<MfgBatchPayload> for protos::mfg_batch_payload::MfgBatchPayload {
    fn from_native(native: MfgBatchPayload) -> Result<Self, ProtoConversionError> {
        let mut proto = mfg_batch_payload::MfgBatchPayload::new();

        proto.set_timestamp(*native.timestamp());

        match native.action() {
            Action::MfgBatchCreate(payload) => {
                proto.set_action(MfgBatchPayload_Action::MFG_BATCH_CREATE);
                proto.set_mfg_batch_create(payload.clone().into_proto()?);
            }
            Action::MfgBatchUpdate(payload) => {
                proto.set_action(MfgBatchPayload_Action::MFG_BATCH_UPDATE);
                proto.set_mfg_batch_update(payload.clone().into_proto()?);
            }
            Action::MfgBatchDelete(payload) => {
                proto.set_action(MfgBatchPayload_Action::MFG_BATCH_DELETE);
                proto.set_mfg_batch_delete(payload.clone().into_proto()?);
            }
        }

        Ok(proto)
    }
}

impl FromBytes<MfgBatchPayload> for MfgBatchPayload {
    fn from_bytes(bytes: &[u8]) -> Result<MfgBatchPayload, ProtoConversionError> {
        let proto: mfg_batch_payload::MfgBatchPayload =
            Message::parse_from_bytes(bytes).map_err(|_| {
                ProtoConversionError::SerializationError(
                    "Unable to get MfgBatchPayload from bytes".into(),
                )
            })?;
        proto.into_native()
    }
}

impl IntoBytes for MfgBatchPayload {
    fn into_bytes(self) -> Result<Vec<u8>, ProtoConversionError> {
        let proto = self.into_proto()?;
        let bytes = proto.write_to_bytes().map_err(|_| {
            ProtoConversionError::SerializationError(
                "Unable to get MfgBatchPayload from bytes".into(),
            )
        })?;
        Ok(bytes)
    }
}

impl IntoProto<protos::mfg_batch_payload::MfgBatchPayload> for MfgBatchPayload {}
impl IntoNative<MfgBatchPayload> for protos::mfg_batch_payload::MfgBatchPayload {}

/// Returned if any required fields in a `MfgBatchPayload` are not present when being
/// converted from the corresponding builder
#[derive(Debug)]
pub enum MfgBatchPayloadBuildError {
    MissingField(String),
}

impl StdError for MfgBatchPayloadBuildError {
    fn description(&self) -> &str {
        match *self {
            MfgBatchPayloadBuildError::MissingField(ref msg) => msg,
        }
    }

    fn cause(&self) -> Option<&dyn StdError> {
        match *self {
            MfgBatchPayloadBuildError::MissingField(_) => None,
        }
    }
}

impl std::fmt::Display for MfgBatchPayloadBuildError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            MfgBatchPayloadBuildError::MissingField(ref s) => write!(f, "missing field \"{}\"", s),
        }
    }
}

/// Builder used to create a `MfgBatchPayload`
#[derive(Default, Clone)]
pub struct MfgBatchPayloadBuilder {
    action: Option<Action>,
    timestamp: Option<u64>,
}

impl MfgBatchPayloadBuilder {
    pub fn new() -> Self {
        MfgBatchPayloadBuilder::default()
    }
    pub fn with_action(mut self, action: Action) -> Self {
        self.action = Some(action);
        self
    }
    pub fn with_timestamp(mut self, value: u64) -> Self {
        self.timestamp = Some(value);
        self
    }
    pub fn build(self) -> Result<MfgBatchPayload, BuilderError> {
        let action = self
            .action
            .ok_or_else(|| BuilderError::MissingField("'action' field is required".into()))?;
        let timestamp = self
            .timestamp
            .ok_or_else(|| BuilderError::MissingField("'timestamp' field is required".into()))?;
        Ok(MfgBatchPayload { action, timestamp })
    }
}

/// Native representation of the "create product" action payload
#[derive(Debug, Default, Clone, PartialEq)]
pub struct MfgBatchCreateAction {
    mfg_batch_namespace: MfgBatchNamespace,
    mfg_batch_id: String,
    owner: String,
    properties: Vec<PropertyValue>,
}

impl MfgBatchCreateAction {
    pub fn mfg_batch_namespace(&self) -> &MfgBatchNamespace {
        &self.mfg_batch_namespace
    }

    pub fn mfg_batch_id(&self) -> &str {
        &self.mfg_batch_id
    }

    pub fn owner(&self) -> &str {
        &self.owner
    }

    pub fn properties(&self) -> &[PropertyValue] {
        &self.properties
    }
}

impl FromProto<mfg_batch_payload::MfgBatchCreateAction> for MfgBatchCreateAction {
    fn from_proto(
        proto: mfg_batch_payload::MfgBatchCreateAction,
    ) -> Result<Self, ProtoConversionError> {
        Ok(MfgBatchCreateAction {
            mfg_batch_namespace: MfgBatchNamespace::from_proto(proto.get_mfg_batch_namespace())?,
            mfg_batch_id: proto.get_mfg_batch_id().to_string(),
            owner: proto.get_owner().to_string(),
            properties: proto
                .get_properties()
                .to_vec()
                .into_iter()
                .map(PropertyValue::from_proto)
                .collect::<Result<Vec<PropertyValue>, ProtoConversionError>>()?,
        })
    }
}

impl FromNative<MfgBatchCreateAction> for mfg_batch_payload::MfgBatchCreateAction {
    fn from_native(native: MfgBatchCreateAction) -> Result<Self, ProtoConversionError> {
        let mut proto = protos::mfg_batch_payload::MfgBatchCreateAction::new();
        proto.set_mfg_batch_namespace(native.mfg_batch_namespace().clone().into_proto()?);
        proto.set_mfg_batch_id(native.mfg_batch_id().to_string());
        proto.set_owner(native.owner().to_string());
        proto.set_properties(RepeatedField::from_vec(
            native
                .properties()
                .to_vec()
                .into_iter()
                .map(PropertyValue::into_proto)
                .collect::<Result<Vec<protos::schema_state::PropertyValue>, ProtoConversionError>>(
                )?,
        ));
        Ok(proto)
    }
}

impl FromBytes<MfgBatchCreateAction> for MfgBatchCreateAction {
    fn from_bytes(bytes: &[u8]) -> Result<MfgBatchCreateAction, ProtoConversionError> {
        let proto: protos::mfg_batch_payload::MfgBatchCreateAction = Message::parse_from_bytes(bytes)
            .map_err(|_| {
                ProtoConversionError::SerializationError(
                    "Unable to get MfgBatchCreateAction from bytes".to_string(),
                )
            })?;
        proto.into_native()
    }
}

impl IntoBytes for MfgBatchCreateAction {
    fn into_bytes(self) -> Result<Vec<u8>, ProtoConversionError> {
        let proto = self.into_proto()?;
        let bytes = proto.write_to_bytes().map_err(|_| {
            ProtoConversionError::SerializationError(
                "Unable to get bytes from MfgBatchCreateAction".to_string(),
            )
        })?;
        Ok(bytes)
    }
}

impl IntoProto<protos::mfg_batch_payload::MfgBatchCreateAction> for MfgBatchCreateAction {}
impl IntoNative<MfgBatchCreateAction> for protos::mfg_batch_payload::MfgBatchCreateAction {}

/// Builder used to create a "create product" action payload
#[derive(Default, Debug)]
pub struct MfgBatchCreateActionBuilder {
    mfg_batch_namespace: Option<MfgBatchNamespace>,
    mfg_batch_id: Option<String>,
    owner: Option<String>,
    properties: Option<Vec<PropertyValue>>,
}

impl MfgBatchCreateActionBuilder {
    pub fn new() -> Self {
        MfgBatchCreateActionBuilder::default()
    }
    pub fn with_mfg_batch_namespace(mut self, value: MfgBatchNamespace) -> Self {
        self.mfg_batch_namespace = Some(value);
        self
    }
    pub fn with_mfg_batch_id(mut self, value: String) -> Self {
        self.mfg_batch_id = Some(value);
        self
    }
    pub fn with_owner(mut self, value: String) -> Self {
        self.owner = Some(value);
        self
    }
    pub fn with_properties(mut self, value: Vec<PropertyValue>) -> Self {
        self.properties = Some(value);
        self
    }
    pub fn build(self) -> Result<MfgBatchCreateAction, BuilderError> {
        let mfg_batch_namespace = self.mfg_batch_namespace.ok_or_else(|| {
            BuilderError::MissingField("'mfg_batch_namespace' field is required".to_string())
        })?;
        let mfg_batch_id = self
            .mfg_batch_id
            .ok_or_else(|| BuilderError::MissingField("'mfg_batch_id' field is required".into()))?;
        let owner = self
            .owner
            .ok_or_else(|| BuilderError::MissingField("'owner' field is required".into()))?;
        let properties = self
            .properties
            .ok_or_else(|| BuilderError::MissingField("'properties' field is required".into()))?;
        Ok(MfgBatchCreateAction {
            mfg_batch_namespace,
            mfg_batch_id,
            owner,
            properties,
        })
    }
}

/// Native representation of an "update product" action payload
#[derive(Debug, Default, Clone, PartialEq)]
pub struct MfgBatchUpdateAction {
    mfg_batch_namespace: MfgBatchNamespace,
    mfg_batch_id: String,
    properties: Vec<PropertyValue>,
}

impl MfgBatchUpdateAction {
    pub fn mfg_batch_namespace(&self) -> &MfgBatchNamespace {
        &self.mfg_batch_namespace
    }

    pub fn mfg_batch_id(&self) -> &str {
        &self.mfg_batch_id
    }

    pub fn properties(&self) -> &[PropertyValue] {
        &self.properties
    }
}

impl FromProto<protos::mfg_batch_payload::MfgBatchUpdateAction> for MfgBatchUpdateAction {
    fn from_proto(
        proto: protos::mfg_batch_payload::MfgBatchUpdateAction,
    ) -> Result<Self, ProtoConversionError> {
        Ok(MfgBatchUpdateAction {
            mfg_batch_namespace: MfgBatchNamespace::from_proto(proto.get_mfg_batch_namespace())?,
            mfg_batch_id: proto.get_mfg_batch_id().to_string(),
            properties: proto
                .get_properties()
                .to_vec()
                .into_iter()
                .map(PropertyValue::from_proto)
                .collect::<Result<Vec<PropertyValue>, ProtoConversionError>>()?,
        })
    }
}

impl FromNative<MfgBatchUpdateAction> for protos::mfg_batch_payload::MfgBatchUpdateAction {
    fn from_native(native: MfgBatchUpdateAction) -> Result<Self, ProtoConversionError> {
        let mut proto = protos::mfg_batch_payload::MfgBatchUpdateAction::new();
        proto.set_mfg_batch_namespace(native.mfg_batch_namespace().clone().into_proto()?);
        proto.set_mfg_batch_id(native.mfg_batch_id().to_string());
        proto.set_properties(RepeatedField::from_vec(
            native
                .properties()
                .to_vec()
                .into_iter()
                .map(PropertyValue::into_proto)
                .collect::<Result<Vec<protos::schema_state::PropertyValue>, ProtoConversionError>>(
                )?,
        ));

        Ok(proto)
    }
}

impl FromBytes<MfgBatchUpdateAction> for MfgBatchUpdateAction {
    fn from_bytes(bytes: &[u8]) -> Result<MfgBatchUpdateAction, ProtoConversionError> {
        let proto: protos::mfg_batch_payload::MfgBatchUpdateAction = Message::parse_from_bytes(bytes)
            .map_err(|_| {
                ProtoConversionError::SerializationError(
                    "Unable to get MfgBatchUpdateAction from bytes".to_string(),
                )
            })?;
        proto.into_native()
    }
}

impl IntoBytes for MfgBatchUpdateAction {
    fn into_bytes(self) -> Result<Vec<u8>, ProtoConversionError> {
        let proto = self.into_proto()?;
        let bytes = proto.write_to_bytes().map_err(|_| {
            ProtoConversionError::SerializationError(
                "Unable to get bytes from MfgBatchUpdateAction".to_string(),
            )
        })?;
        Ok(bytes)
    }
}

impl IntoProto<protos::mfg_batch_payload::MfgBatchUpdateAction> for MfgBatchUpdateAction {}
impl IntoNative<MfgBatchUpdateAction> for protos::mfg_batch_payload::MfgBatchUpdateAction {}

/// Builder used to create an "update product" action
#[derive(Default, Clone)]
pub struct MfgBatchUpdateActionBuilder {
    mfg_batch_namespace: Option<MfgBatchNamespace>,
    mfg_batch_id: Option<String>,
    properties: Vec<PropertyValue>,
}

impl MfgBatchUpdateActionBuilder {
    pub fn new() -> Self {
        MfgBatchUpdateActionBuilder::default()
    }

    pub fn with_mfg_batch_namespace(mut self, mfg_batch_namespace: MfgBatchNamespace) -> Self {
        self.mfg_batch_namespace = Some(mfg_batch_namespace);
        self
    }

    pub fn with_mfg_batch_id(mut self, mfg_batch_id: String) -> Self {
        self.mfg_batch_id = Some(mfg_batch_id);
        self
    }

    pub fn with_properties(mut self, properties: Vec<PropertyValue>) -> Self {
        self.properties = properties;
        self
    }

    pub fn build(self) -> Result<MfgBatchUpdateAction, BuilderError> {
        let mfg_batch_namespace = self.mfg_batch_namespace.ok_or_else(|| {
            BuilderError::MissingField("'mfg_batch_namespace' field is required".to_string())
        })?;

        let mfg_batch_id = self.mfg_batch_id.ok_or_else(|| {
            BuilderError::MissingField("'mfg_batch_id' field is required".to_string())
        })?;

        let properties = {
            if !self.properties.is_empty() {
                self.properties
            } else {
                return Err(BuilderError::MissingField(
                    "'properties' field is required".to_string(),
                ));
            }
        };

        Ok(MfgBatchUpdateAction {
            mfg_batch_namespace,
            mfg_batch_id,
            properties,
        })
    }
}

/// Native representation of the "delete product" action payload
#[derive(Debug, Default, Clone, PartialEq)]
pub struct MfgBatchDeleteAction {
    mfg_batch_namespace: MfgBatchNamespace,
    mfg_batch_id: String,
}

impl MfgBatchDeleteAction {
    pub fn mfg_batch_namespace(&self) -> &MfgBatchNamespace {
        &self.mfg_batch_namespace
    }

    pub fn mfg_batch_id(&self) -> &str {
        &self.mfg_batch_id
    }
}

impl FromProto<protos::mfg_batch_payload::MfgBatchDeleteAction> for MfgBatchDeleteAction {
    fn from_proto(
        proto: protos::mfg_batch_payload::MfgBatchDeleteAction,
    ) -> Result<Self, ProtoConversionError> {
        Ok(MfgBatchDeleteAction {
            mfg_batch_namespace: MfgBatchNamespace::from_proto(proto.get_mfg_batch_namespace())?,
            mfg_batch_id: proto.get_mfg_batch_id().to_string(),
        })
    }
}

impl FromNative<MfgBatchDeleteAction> for protos::mfg_batch_payload::MfgBatchDeleteAction {
    fn from_native(native: MfgBatchDeleteAction) -> Result<Self, ProtoConversionError> {
        let mut proto = protos::mfg_batch_payload::MfgBatchDeleteAction::new();
        proto.set_mfg_batch_namespace(native.mfg_batch_namespace().clone().into_proto()?);
        proto.set_mfg_batch_id(native.mfg_batch_id().to_string());
        Ok(proto)
    }
}

impl FromBytes<MfgBatchDeleteAction> for MfgBatchDeleteAction {
    fn from_bytes(bytes: &[u8]) -> Result<MfgBatchDeleteAction, ProtoConversionError> {
        let proto: protos::mfg_batch_payload::MfgBatchDeleteAction = Message::parse_from_bytes(bytes)
            .map_err(|_| {
                ProtoConversionError::SerializationError(
                    "Unable to get MfgBatchDeleteAction from bytes".to_string(),
                )
            })?;
        proto.into_native()
    }
}

impl IntoBytes for MfgBatchDeleteAction {
    fn into_bytes(self) -> Result<Vec<u8>, ProtoConversionError> {
        let proto = self.into_proto()?;
        let bytes = proto.write_to_bytes().map_err(|_| {
            ProtoConversionError::SerializationError(
                "Unable to get bytes from MfgBatchDeleteAction".to_string(),
            )
        })?;
        Ok(bytes)
    }
}

impl IntoProto<protos::mfg_batch_payload::MfgBatchDeleteAction> for MfgBatchDeleteAction {}
impl IntoNative<MfgBatchDeleteAction> for protos::mfg_batch_payload::MfgBatchDeleteAction {}

/// Builder used to create a "delete product" action
#[derive(Default, Clone)]
pub struct MfgBatchDeleteActionBuilder {
    mfg_batch_namespace: Option<MfgBatchNamespace>,
    mfg_batch_id: Option<String>,
}

impl MfgBatchDeleteActionBuilder {
    pub fn new() -> Self {
        MfgBatchDeleteActionBuilder::default()
    }

    pub fn with_mfg_batch_namespace(mut self, mfg_batch_namespace: MfgBatchNamespace) -> Self {
        self.mfg_batch_namespace = Some(mfg_batch_namespace);
        self
    }

    pub fn with_mfg_batch_id(mut self, mfg_batch_id: String) -> Self {
        self.mfg_batch_id = Some(mfg_batch_id);
        self
    }

    pub fn build(self) -> Result<MfgBatchDeleteAction, BuilderError> {
        let mfg_batch_namespace = self.mfg_batch_namespace.ok_or_else(|| {
            BuilderError::MissingField("'mfg_batch_namespace' field is required".to_string())
        })?;

        let mfg_batch_id = self.mfg_batch_id.ok_or_else(|| {
            BuilderError::MissingField("'mfg_batch_id' field is required".to_string())
        })?;

        Ok(MfgBatchDeleteAction {
            mfg_batch_namespace,
            mfg_batch_id,
        })
    }
}
/*
#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::schema::state::{DataType, PropertyValueBuilder};
    use std::fmt::Debug;

    #[test]
    /// Validate that a `MfgBatchCreateAction` is built correctly
    fn test_product_create_builder() {
        let action = MfgBatchCreateActionBuilder::new()
            .with_mfg_batch_id("688955434684".into()) // GTIN-12
            .with_mfg_batch_namespace(MfgBatchNamespace::Gs1)
            .with_owner("Target".into())
            .with_properties(make_properties())
            .build()
            .unwrap();

        assert_eq!(action.mfg_batch_id(), "688955434684");
        assert_eq!(action.owner(), "Target");
        assert_eq!(*action.mfg_batch_namespace(), MfgBatchNamespace::Gs1);
        assert_eq!(action.properties()[0].name(), "description");
        assert_eq!(*action.properties()[0].data_type(), DataType::String);
        assert_eq!(
            action.properties()[0].string_value(),
            "This is a product description"
        );
        assert_eq!(action.properties()[1].name(), "price");
        assert_eq!(*action.properties()[1].data_type(), DataType::Number);
        assert_eq!(*action.properties()[1].number_value(), 3);
    }

    #[test]
    /// Validate that a `MfgBatchCreateAction` may be correctly converted into bytes and back
    /// to its native representation
    fn test_product_create_into_bytes() {
        let action = MfgBatchCreateActionBuilder::new()
            .with_mfg_batch_id("688955434684".into()) // GTIN-12
            .with_mfg_batch_namespace(MfgBatchNamespace::Gs1)
            .with_owner("Target".into())
            .with_properties(make_properties())
            .build()
            .unwrap();

        test_from_bytes(action, MfgBatchCreateAction::from_bytes);
    }

    #[test]
    /// Validate that a `MfgBatchUpdateAction` is built correctly
    fn test_product_update_builder() {
        let action = MfgBatchUpdateActionBuilder::new()
            .with_mfg_batch_id("688955434684".into()) // GTIN-12
            .with_mfg_batch_namespace(MfgBatchNamespace::Gs1)
            .with_properties(make_properties())
            .build()
            .unwrap();

        assert_eq!(action.mfg_batch_id(), "688955434684");
        assert_eq!(*action.mfg_batch_namespace(), MfgBatchNamespace::Gs1);
        assert_eq!(action.properties()[0].name(), "description");
        assert_eq!(*action.properties()[0].data_type(), DataType::String);
        assert_eq!(
            action.properties()[0].string_value(),
            "This is a product description"
        );
        assert_eq!(action.properties()[1].name(), "price");
        assert_eq!(*action.properties()[1].data_type(), DataType::Number);
        assert_eq!(*action.properties()[1].number_value(), 3);
    }

    #[test]
    /// Validate that an `MfgBatchUpdateAction` may be correctly converted into bytes and back
    /// to its native representation
    fn test_product_update_into_bytes() {
        let action = MfgBatchUpdateActionBuilder::new()
            .with_mfg_batch_id("688955434684".into()) // GTIN-12
            .with_mfg_batch_namespace(MfgBatchNamespace::Gs1)
            .with_properties(make_properties())
            .build()
            .unwrap();

        test_from_bytes(action, MfgBatchUpdateAction::from_bytes);
    }

    #[test]
    /// Validate that an `MfgBatchDeleteAction` may be built correctly
    fn test_product_delete_builder() {
        let action = MfgBatchDeleteActionBuilder::new()
            .with_mfg_batch_id("688955434684".into()) // GTIN-12
            .with_mfg_batch_namespace(MfgBatchNamespace::Gs1)
            .build()
            .unwrap();

        assert_eq!(action.mfg_batch_id(), "688955434684");
        assert_eq!(*action.mfg_batch_namespace(), MfgBatchNamespace::Gs1);
    }

    #[test]
    /// Validate that a `MfgBatchDeleteAction` may be correctly converted into bytes and back
    /// to its native representation
    fn test_product_delete_into_bytes() {
        let action = MfgBatchDeleteActionBuilder::new()
            .with_mfg_batch_id("688955434684".into()) // GTIN-12
            .with_mfg_batch_namespace(MfgBatchNamespace::Gs1)
            .build()
            .unwrap();

        test_from_bytes(action, MfgBatchDeleteAction::from_bytes);
    }

    #[test]
    /// Validate that a `MfgBatchPayload` is built correctly with a `MfgBatchCreateAction`
    fn test_mfg_batch_payload_builder() {
        let action = MfgBatchCreateActionBuilder::new()
            .with_mfg_batch_id("688955434684".into()) // GTIN-12
            .with_mfg_batch_namespace(MfgBatchNamespace::Gs1)
            .with_owner("Target".into())
            .with_properties(make_properties())
            .build()
            .unwrap();

        let payload = MfgBatchPayloadBuilder::new()
            .with_action(Action::MfgBatchCreate(action.clone()))
            .with_timestamp(0)
            .build()
            .unwrap();

        assert_eq!(*payload.action(), Action::MfgBatchCreate(action));
        assert_eq!(*payload.timestamp(), 0);
    }

    #[test]
    /// Validate that a `MfgBatchPayload` with a `MfgBatchCreateAction` may be correctly converted
    /// into bytes and back to its native representation
    fn test_mfg_batch_payload_bytes() {
        let action = MfgBatchCreateActionBuilder::new()
            .with_mfg_batch_id("688955434684".into()) // GTIN-12
            .with_mfg_batch_namespace(MfgBatchNamespace::Gs1)
            .with_owner("Target".into())
            .with_properties(make_properties())
            .build()
            .unwrap();

        let payload = MfgBatchPayloadBuilder::new()
            .with_action(Action::MfgBatchCreate(action.clone()))
            .with_timestamp(0)
            .build()
            .unwrap();

        test_from_bytes(payload, MfgBatchPayload::from_bytes);
    }

    fn make_properties() -> Vec<PropertyValue> {
        let property_value_description = PropertyValueBuilder::new()
            .with_name("description".into())
            .with_data_type(DataType::String)
            .with_string_value("This is a product description".into())
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

    fn test_from_bytes<T: FromBytes<T> + Clone + PartialEq + IntoBytes + Debug, F>(
        under_test: T,
        from_bytes: F,
    ) where
        F: Fn(&[u8]) -> Result<T, ProtoConversionError>,
    {
        let bytes = under_test.clone().into_bytes().unwrap();
        let created_from_bytes = from_bytes(&bytes).unwrap();
        assert_eq!(under_test, created_from_bytes);
    }
}
*/
