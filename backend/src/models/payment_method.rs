use crate::utils::{opt_object_id_as_string};
use mongodb::bson::oid::ObjectId;
use serde::{Serialize, Deserialize};
use validator::Validate;


#[derive(Debug, Serialize, Deserialize)]
pub struct PaymentMethod {
    #[serde(
        rename = "_id",
        skip_serializing_if = "Option::is_none",
        serialize_with = "opt_object_id_as_string"
    )]
    pub id: Option<ObjectId>,
    pub name: String,
    pub is_active: bool
}


#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct PaymentMethodDTO {
    pub name: String,
    pub is_active: bool
}
