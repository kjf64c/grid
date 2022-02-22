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

use chrono::NaiveDateTime;

use crate::mfg_batch::{
    store::{LatLongValue, MfgBatch as GridMfgBatch, PropertyValue},
    MAX_COMMIT_NUM,
};

use super::schema::{mfg_batch, mfg_batch_property_value};

#[derive(Clone, Insertable, Debug)]
#[table_name = "mfg_batch"]
pub struct NewMfgBatch {
    pub mfg_batch_id: String,
    pub mfg_batch_address: String,
    pub mfg_batch_namespace: String,
    pub owner: String,
    pub start_commit_num: i64,
    pub end_commit_num: i64,
    pub service_id: Option<String>,
}

#[derive(Queryable, Identifiable, Debug)]
#[table_name = "mfg_batch"]
pub struct MfgBatch {
    pub id: i64,
    pub mfg_batch_id: String,
    pub mfg_batch_address: String,
    pub mfg_batch_namespace: String,
    pub owner: String,
    pub start_commit_num: i64,
    pub end_commit_num: i64,
    pub service_id: Option<String>,
    pub last_updated: Option<NaiveDateTime>,
}

#[derive(AsChangeset, Clone, Insertable, Debug)]
#[table_name = "mfg_batch_property_value"]
pub struct NewMfgBatchPropertyValue {
    pub mfg_batch_id: String,
    pub mfg_batch_address: String,
    pub property_name: String,
    pub parent_property: Option<String>,
    pub data_type: String,
    pub bytes_value: Option<Vec<u8>>,
    pub boolean_value: Option<bool>,
    pub number_value: Option<i64>,
    pub string_value: Option<String>,
    pub enum_value: Option<i32>,
    pub latitude_value: Option<i64>,
    pub longitude_value: Option<i64>,
    pub start_commit_num: i64,
    pub end_commit_num: i64,
    pub service_id: Option<String>,
}

#[derive(Queryable, Identifiable, Debug)]
#[table_name = "mfg_batch_property_value"]
pub struct MfgBatchPropertyValue {
    pub id: i64,
    pub mfg_batch_id: String,
    pub mfg_batch_address: String,
    pub property_name: String,
    pub parent_property: Option<String>,
    pub data_type: String,
    pub bytes_value: Option<Vec<u8>>,
    pub boolean_value: Option<bool>,
    pub number_value: Option<i64>,
    pub string_value: Option<String>,
    pub enum_value: Option<i32>,
    pub latitude_value: Option<i64>,
    pub longitude_value: Option<i64>,
    pub start_commit_num: i64,
    pub end_commit_num: i64,
    pub service_id: Option<String>,
}

impl From<GridMfgBatch> for (NewMfgBatch, Vec<NewMfgBatchPropertyValue>) {
    fn from(mfg_batch: GridMfgBatch) -> Self {
        let new_mfg_batch = NewMfgBatch {
            mfg_batch_id: mfg_batch.mfg_batch_id.clone(),
            mfg_batch_address: mfg_batch.mfg_batch_address.clone(),
            mfg_batch_namespace: mfg_batch.mfg_batch_namespace.clone(),
            owner: mfg_batch.owner.clone(),
            start_commit_num: mfg_batch.start_commit_num,
            end_commit_num: mfg_batch.end_commit_num,
            service_id: mfg_batch.service_id.clone(),
        };

        (new_mfg_batch, make_property_values(None, &mfg_batch.properties))
    }
}

impl From<(MfgBatch, Vec<PropertyValue>)> for GridMfgBatch {
    fn from((model, properties): (MfgBatch, Vec<PropertyValue>)) -> Self {
        Self {
            mfg_batch_id: model.mfg_batch_id,
            mfg_batch_address: model.mfg_batch_address,
            mfg_batch_namespace: model.mfg_batch_namespace,
            owner: model.owner,
            start_commit_num: model.start_commit_num,
            end_commit_num: model.end_commit_num,
            service_id: model.service_id,
            last_updated: model.last_updated.map(|d| d.timestamp()),
            properties,
        }
    }
}

fn make_property_values(
    parent_property: Option<String>,
    properties: &[PropertyValue],
) -> Vec<NewMfgBatchPropertyValue> {
    let mut model_properties: Vec<NewMfgBatchPropertyValue> = Vec::new();

    for property in properties {
        model_properties.push(NewMfgBatchPropertyValue {
            mfg_batch_id: property.mfg_batch_id.clone(),
            mfg_batch_address: property.mfg_batch_address.clone(),
            property_name: property.property_name.clone(),
            parent_property: parent_property.clone(),
            data_type: property.data_type.clone(),
            bytes_value: property.bytes_value.clone(),
            boolean_value: property.boolean_value,
            number_value: property.number_value,
            string_value: property.string_value.clone(),
            enum_value: property.enum_value,
            latitude_value: property.lat_long_value.clone().map(|l| l.latitude),
            longitude_value: property.lat_long_value.clone().map(|l| l.longitude),
            start_commit_num: property.start_commit_num,
            end_commit_num: MAX_COMMIT_NUM,
            service_id: property.service_id.clone(),
        });

        if !property.struct_values.is_empty() {
            model_properties.append(&mut make_property_values(
                Some(format!(
                    "{}:{}",
                    property.mfg_batch_id, property.property_name
                )),
                &property.struct_values,
            ));
        }
    }

    model_properties
}

impl From<MfgBatchPropertyValue> for PropertyValue {
    fn from(model: MfgBatchPropertyValue) -> Self {
        Self {
            mfg_batch_id: model.mfg_batch_id,
            mfg_batch_address: model.mfg_batch_address,
            property_name: model.property_name,
            data_type: model.data_type,
            bytes_value: model.bytes_value,
            boolean_value: model.boolean_value,
            number_value: model.number_value,
            string_value: model.string_value,
            enum_value: model.enum_value,
            struct_values: vec![],
            lat_long_value: if model.latitude_value.is_some() && model.longitude_value.is_some() {
                Some(LatLongValue {
                    latitude: model.latitude_value.unwrap(),
                    longitude: model.longitude_value.unwrap(),
                })
            } else {
                None
            },
            start_commit_num: model.start_commit_num,
            end_commit_num: model.end_commit_num,
            service_id: model.service_id,
        }
    }
}

impl From<(MfgBatchPropertyValue, Vec<PropertyValue>)> for PropertyValue {
    fn from((model, children): (MfgBatchPropertyValue, Vec<PropertyValue>)) -> Self {
        Self {
            mfg_batch_id: model.mfg_batch_id,
            mfg_batch_address: model.mfg_batch_address,
            property_name: model.property_name,
            data_type: model.data_type,
            bytes_value: model.bytes_value,
            boolean_value: model.boolean_value,
            number_value: model.number_value,
            string_value: model.string_value,
            enum_value: model.enum_value,
            struct_values: children,
            lat_long_value: if model.latitude_value.is_some() && model.longitude_value.is_some() {
                Some(LatLongValue {
                    latitude: model.latitude_value.unwrap(),
                    longitude: model.longitude_value.unwrap(),
                })
            } else {
                None
            },
            start_commit_num: model.start_commit_num,
            end_commit_num: model.end_commit_num,
            service_id: model.service_id,
        }
    }
}
