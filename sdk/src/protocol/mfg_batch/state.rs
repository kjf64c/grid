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

//! Protocol structs for MfgBatch state

use protobuf::Message;
use protobuf::RepeatedField;

use std::error::Error as StdError;

use crate::protos;
use crate::protos::schema_state;
use crate::protos::{
    FromBytes, FromNative, FromProto, IntoBytes, IntoNative, IntoProto, ProtoConversionError,
};

use crate::protocol::schema::state::PropertyValue;

/// Possible MfgBatch namespaces
///
/// The namespace determines the schema used to define a `MfgBatch`'s properties
#[derive(Debug, Clone, PartialEq)]
pub enum MfgBatchNamespace {
    Gs1,
}

impl Default for MfgBatchNamespace {
    fn default() -> Self {
        MfgBatchNamespace::Gs1
    }
}

impl FromProto<protos::mfg_batch_state::MfgBatch_MfgBatchNamespace> for MfgBatchNamespace {
    fn from_proto(
        product_namespace: protos::mfg_batch_state::MfgBatch_MfgBatchNamespace,
    ) -> Result<Self, ProtoConversionError> {
        match product_namespace {
            protos::mfg_batch_state::MfgBatch_MfgBatchNamespace::GS1 => Ok(MfgBatchNamespace::Gs1),
            protos::mfg_batch_state::MfgBatch_MfgBatchNamespace::UNSET_TYPE => {
                Err(ProtoConversionError::InvalidTypeError(
                    "Cannot convert MfgBatch_MfgBatchNamespace with type UNSET_TYPE".to_string(),
                ))
            }
        }
    }
}

impl FromNative<MfgBatchNamespace> for protos::mfg_batch_state::MfgBatch_MfgBatchNamespace {
    fn from_native(product_namespace: MfgBatchNamespace) -> Result<Self, ProtoConversionError> {
        match product_namespace {
            MfgBatchNamespace::Gs1 => Ok(protos::mfg_batch_state::MfgBatch_MfgBatchNamespace::GS1),
        }
    }
}

impl IntoProto<protos::mfg_batch_state::MfgBatch_MfgBatchNamespace> for MfgBatchNamespace {}
impl IntoNative<MfgBatchNamespace> for protos::mfg_batch_state::MfgBatch_MfgBatchNamespace {}

/// Native representation of `MfgBatch`
///
/// A `MfgBatch` contains a list of properties determined by the `product_namespace`.
#[derive(Debug, Clone, PartialEq)]
pub struct MfgBatch {
    product_id: String,
    product_namespace: MfgBatchNamespace,
    owner: String,
    properties: Vec<PropertyValue>,
}

impl MfgBatch {
    pub fn product_id(&self) -> &str {
        &self.product_id
    }

    pub fn product_namespace(&self) -> &MfgBatchNamespace {
        &self.product_namespace
    }

    pub fn owner(&self) -> &str {
        &self.owner
    }

    pub fn properties(&self) -> &[PropertyValue] {
        &self.properties
    }

    pub fn into_builder(self) -> MfgBatchBuilder {
        MfgBatchBuilder::new()
            .with_product_id(self.product_id)
            .with_product_namespace(self.product_namespace)
            .with_owner(self.owner)
            .with_properties(self.properties)
    }
}

impl FromProto<protos::mfg_batch_state::MfgBatch> for MfgBatch {
    fn from_proto(product: protos::mfg_batch_state::MfgBatch) -> Result<Self, ProtoConversionError> {
        Ok(MfgBatch {
            product_id: product.get_product_id().to_string(),
            product_namespace: MfgBatchNamespace::from_proto(product.get_product_namespace())?,
            owner: product.get_owner().to_string(),
            properties: product
                .get_properties()
                .to_vec()
                .into_iter()
                .map(PropertyValue::from_proto)
                .collect::<Result<Vec<PropertyValue>, ProtoConversionError>>()?,
        })
    }
}

impl FromNative<MfgBatch> for protos::mfg_batch_state::MfgBatch {
    fn from_native(product: MfgBatch) -> Result<Self, ProtoConversionError> {
        let mut proto = protos::mfg_batch_state::MfgBatch::new();
        proto.set_product_id(product.product_id().to_string());
        proto.set_product_namespace(product.product_namespace().clone().into_proto()?);
        proto.set_owner(product.owner().to_string());
        proto.set_properties(RepeatedField::from_vec(
            product
                .properties()
                .to_vec()
                .into_iter()
                .map(PropertyValue::into_proto)
                .collect::<Result<Vec<schema_state::PropertyValue>, ProtoConversionError>>()?,
        ));
        Ok(proto)
    }
}

impl FromBytes<MfgBatch> for MfgBatch {
    fn from_bytes(bytes: &[u8]) -> Result<MfgBatch, ProtoConversionError> {
        let proto: protos::mfg_batch_state::MfgBatch =
            Message::parse_from_bytes(bytes).map_err(|_| {
                ProtoConversionError::SerializationError(
                    "Unable to get MfgBatch from bytes".to_string(),
                )
            })?;
        proto.into_native()
    }
}

impl IntoBytes for MfgBatch {
    fn into_bytes(self) -> Result<Vec<u8>, ProtoConversionError> {
        let proto = self.into_proto()?;
        let bytes = proto.write_to_bytes().map_err(|_| {
            ProtoConversionError::SerializationError("Unable to get bytes from MfgBatch".to_string())
        })?;
        Ok(bytes)
    }
}

impl IntoProto<protos::mfg_batch_state::MfgBatch> for MfgBatch {}
impl IntoNative<MfgBatch> for protos::mfg_batch_state::MfgBatch {}

/// Returned if any required fields in a `MfgBatch` are not present when being
/// converted from the corresponding builder
#[derive(Debug)]
pub enum MfgBatchBuildError {
    MissingField(String),
    EmptyVec(String),
}

impl StdError for MfgBatchBuildError {
    fn description(&self) -> &str {
        match *self {
            MfgBatchBuildError::MissingField(ref msg) => msg,
            MfgBatchBuildError::EmptyVec(ref msg) => msg,
        }
    }

    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match *self {
            MfgBatchBuildError::MissingField(_) => None,
            MfgBatchBuildError::EmptyVec(_) => None,
        }
    }
}

impl std::fmt::Display for MfgBatchBuildError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            MfgBatchBuildError::MissingField(ref s) => write!(f, "missing field \"{}\"", s),
            MfgBatchBuildError::EmptyVec(ref s) => write!(f, "\"{}\" must not be empty", s),
        }
    }
}

/// Builder used to create a `MfgBatch`
#[derive(Default, Clone, PartialEq)]
pub struct MfgBatchBuilder {
    pub product_id: Option<String>,
    pub product_namespace: Option<MfgBatchNamespace>,
    pub owner: Option<String>,
    pub properties: Option<Vec<PropertyValue>>,
}

impl MfgBatchBuilder {
    pub fn new() -> Self {
        MfgBatchBuilder::default()
    }

    pub fn with_product_id(mut self, product_id: String) -> Self {
        self.product_id = Some(product_id);
        self
    }

    pub fn with_product_namespace(mut self, product_namespace: MfgBatchNamespace) -> Self {
        self.product_namespace = Some(product_namespace);
        self
    }

    pub fn with_owner(mut self, owner: String) -> Self {
        self.owner = Some(owner);
        self
    }

    pub fn with_properties(mut self, properties: Vec<PropertyValue>) -> Self {
        self.properties = Some(properties);
        self
    }

    pub fn build(self) -> Result<MfgBatch, MfgBatchBuildError> {
        let product_id = self.product_id.ok_or_else(|| {
            MfgBatchBuildError::MissingField("'product_id' field is required".to_string())
        })?;

        let product_namespace = self.product_namespace.ok_or_else(|| {
            MfgBatchBuildError::MissingField("'product_namespace' field is required".to_string())
        })?;

        let owner = self.owner.ok_or_else(|| {
            MfgBatchBuildError::MissingField("'owner' field is required".to_string())
        })?;

        // MfgBatch values are not required
        let properties = self.properties.ok_or_else(|| {
            MfgBatchBuildError::MissingField("'properties' field is required".to_string())
        })?;

        Ok(MfgBatch {
            product_id,
            product_namespace,
            owner,
            properties,
        })
    }
}

/// Native representation of a list of `MfgBatch`s
#[derive(Debug, Clone, PartialEq)]
pub struct MfgBatchList {
    products: Vec<MfgBatch>,
}

impl MfgBatchList {
    pub fn products(&self) -> &[MfgBatch] {
        &self.products
    }

    pub fn into_builder(self) -> MfgBatchListBuilder {
        MfgBatchListBuilder::new().with_products(self.products)
    }
}

impl FromProto<protos::mfg_batch_state::MfgBatchList> for MfgBatchList {
    fn from_proto(
        product_list: protos::mfg_batch_state::MfgBatchList,
    ) -> Result<Self, ProtoConversionError> {
        Ok(MfgBatchList {
            products: product_list
                .get_entries()
                .to_vec()
                .into_iter()
                .map(MfgBatch::from_proto)
                .collect::<Result<Vec<MfgBatch>, ProtoConversionError>>()?,
        })
    }
}

impl FromNative<MfgBatchList> for protos::mfg_batch_state::MfgBatchList {
    fn from_native(product_list: MfgBatchList) -> Result<Self, ProtoConversionError> {
        let mut product_list_proto = protos::mfg_batch_state::MfgBatchList::new();

        product_list_proto.set_entries(RepeatedField::from_vec(
            product_list
                .products()
                .to_vec()
                .into_iter()
                .map(MfgBatch::into_proto)
                .collect::<Result<Vec<protos::mfg_batch_state::MfgBatch>, ProtoConversionError>>()?,
        ));

        Ok(product_list_proto)
    }
}

impl FromBytes<MfgBatchList> for MfgBatchList {
    fn from_bytes(bytes: &[u8]) -> Result<MfgBatchList, ProtoConversionError> {
        let proto: protos::mfg_batch_state::MfgBatchList =
            Message::parse_from_bytes(bytes).map_err(|_| {
                ProtoConversionError::SerializationError(
                    "Unable to get MfgBatchList from bytes".to_string(),
                )
            })?;
        proto.into_native()
    }
}

impl IntoBytes for MfgBatchList {
    fn into_bytes(self) -> Result<Vec<u8>, ProtoConversionError> {
        let proto = self.into_proto()?;
        let bytes = proto.write_to_bytes().map_err(|_| {
            ProtoConversionError::SerializationError(
                "Unable to get bytes from MfgBatchList".to_string(),
            )
        })?;
        Ok(bytes)
    }
}

impl IntoProto<protos::mfg_batch_state::MfgBatchList> for MfgBatchList {}
impl IntoNative<MfgBatchList> for protos::mfg_batch_state::MfgBatchList {}

/// Returned if any required fields in a `MfgBatchList` are not present when being
/// converted from the corresponding builder
#[derive(Debug)]
pub enum MfgBatchListBuildError {
    MissingField(String),
}

impl StdError for MfgBatchListBuildError {
    fn description(&self) -> &str {
        match *self {
            MfgBatchListBuildError::MissingField(ref msg) => msg,
        }
    }

    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match *self {
            MfgBatchListBuildError::MissingField(_) => None,
        }
    }
}

impl std::fmt::Display for MfgBatchListBuildError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            MfgBatchListBuildError::MissingField(ref s) => write!(f, "missing field \"{}\"", s),
        }
    }
}

/// Builder used to create a `MfgBatchList`
#[derive(Default, Clone)]
pub struct MfgBatchListBuilder {
    pub products: Option<Vec<MfgBatch>>,
}

impl MfgBatchListBuilder {
    pub fn new() -> Self {
        MfgBatchListBuilder::default()
    }

    pub fn with_products(mut self, products: Vec<MfgBatch>) -> MfgBatchListBuilder {
        self.products = Some(products);
        self
    }

    pub fn build(self) -> Result<MfgBatchList, MfgBatchListBuildError> {
        // MfgBatch values are not required
        let products = self.products.ok_or_else(|| {
            MfgBatchListBuildError::MissingField("'products' field is required".to_string())
        })?;

        let products = {
            if products.is_empty() {
                return Err(MfgBatchListBuildError::MissingField(
                    "'products' cannot be empty".to_string(),
                ));
            } else {
                products
            }
        };

        Ok(MfgBatchList { products })
    }
}
/*
#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::schema::state::{DataType, PropertyValueBuilder};
    use std::fmt::Debug;

    #[test]
    /// Validate that a `MfgBatch` may be built correctly
    fn test_product_builder() {
        let product = build_product();

        assert_eq!(product.product_id(), "688955434684");
        assert_eq!(*product.product_namespace(), MfgBatchNamespace::Gs1);
        assert_eq!(product.owner(), "Target");
        assert_eq!(product.properties()[0].name(), "description");
        assert_eq!(*product.properties()[0].data_type(), DataType::String);
        assert_eq!(
            product.properties()[0].string_value(),
            "This is a product description"
        );
        assert_eq!(product.properties()[1].name(), "price");
        assert_eq!(*product.properties()[1].data_type(), DataType::Number);
        assert_eq!(*product.properties()[1].number_value(), 3);
    }

    #[test]
    /// Validate that a `MfgBatch` may be correctly converted back to its respective builder
    fn test_product_into_builder() {
        let product = build_product();

        let builder = product.into_builder();

        assert_eq!(builder.product_id, Some("688955434684".to_string()));
        assert_eq!(builder.product_namespace, Some(MfgBatchNamespace::Gs1));
        assert_eq!(builder.owner, Some("Target".to_string()));
        assert_eq!(builder.properties, Some(make_properties()));
    }

    #[test]
    /// Validate that a `MfgBatch` may be correctly converted into bytes and then back to its native
    /// representation
    fn test_product_into_bytes() {
        let builder = MfgBatchBuilder::new();
        let original = builder
            .with_product_id("688955434684".into())
            .with_product_namespace(MfgBatchNamespace::Gs1)
            .with_owner("Target".into())
            .with_properties(make_properties())
            .build()
            .unwrap();

        test_from_bytes(original, MfgBatch::from_bytes);
    }

    #[test]
    /// Validate that a list of products, `MfgBatchList`, can be built correctly
    fn test_product_list_builder() {
        let product_list = build_product_list();

        assert_eq!(product_list.products.len(), 2);

        // Test product 1
        assert_eq!(product_list.products[0].product_id(), "688955434684");
        assert_eq!(
            *product_list.products[0].product_namespace(),
            MfgBatchNamespace::Gs1
        );
        assert_eq!(product_list.products[0].owner(), "Target");
        assert_eq!(
            product_list.products[0].properties()[0].name(),
            "description"
        );
        assert_eq!(
            *product_list.products[0].properties()[0].data_type(),
            DataType::String
        );
        assert_eq!(
            product_list.products[0].properties()[0].string_value(),
            "This is a product description"
        );
        assert_eq!(product_list.products[0].properties()[1].name(), "price");
        assert_eq!(
            *product_list.products[0].properties()[1].data_type(),
            DataType::Number
        );
        assert_eq!(*product_list.products[0].properties()[1].number_value(), 3);

        // Test product 2
        assert_eq!(product_list.products[1].product_id(), "688955434685");
        assert_eq!(
            *product_list.products[1].product_namespace(),
            MfgBatchNamespace::Gs1
        );
        assert_eq!(product_list.products[1].owner(), "Cargill");
        assert_eq!(
            product_list.products[1].properties()[0].name(),
            "description"
        );
        assert_eq!(
            *product_list.products[1].properties()[0].data_type(),
            DataType::String
        );
        assert_eq!(
            product_list.products[1].properties()[0].string_value(),
            "This is a product description"
        );
        assert_eq!(product_list.products[1].properties()[1].name(), "price");
        assert_eq!(
            *product_list.products[1].properties()[1].data_type(),
            DataType::Number
        );
        assert_eq!(*product_list.products[1].properties()[1].number_value(), 3);
    }

    #[test]
    /// Validate that a `MfgBatchList` can be correctly converted back to a builder
    fn test_product_list_into_builder() {
        let product_list = build_product_list();

        let builder = product_list.into_builder();

        assert_eq!(builder.products, Some(make_products()));
    }

    #[test]
    /// Validate that a `MfgBatchList` can be converted into bytes and back to its native
    /// representation successfully
    fn test_product_list_into_bytes() {
        let builder = MfgBatchListBuilder::new();
        let original = builder.with_products(make_products()).build().unwrap();

        test_from_bytes(original, MfgBatchList::from_bytes);
    }

    fn build_product() -> MfgBatch {
        MfgBatchBuilder::new()
            .with_product_id("688955434684".into()) // GTIN-12
            .with_product_namespace(MfgBatchNamespace::Gs1)
            .with_owner("Target".into())
            .with_properties(make_properties())
            .build()
            .expect("Failed to build test product")
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

    fn build_product_list() -> MfgBatchList {
        MfgBatchListBuilder::new()
            .with_products(make_products())
            .build()
            .expect("Failed to build test product list")
    }

    fn make_products() -> Vec<MfgBatch> {
        vec![
            MfgBatchBuilder::new()
                .with_product_id("688955434684".into()) // GTIN-12
                .with_product_namespace(MfgBatchNamespace::Gs1)
                .with_owner("Target".into())
                .with_properties(make_properties())
                .build()
                .expect("Failed to build test product"),
            MfgBatchBuilder::new()
                .with_product_id("688955434685".into()) // GTIN-12
                .with_product_namespace(MfgBatchNamespace::Gs1)
                .with_owner("Cargill".into())
                .with_properties(make_properties())
                .build()
                .expect("Failed to build test product"),
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

