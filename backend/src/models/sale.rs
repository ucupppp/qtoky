use crate::utils::{object_id_as_string, opt_object_id_as_string};
use mongodb::bson::{DateTime, oid::ObjectId};
use serde::{Deserialize, Serialize};
use validator::Validate;
use super::payment_method::PaymentMethod;

#[derive(Debug, Serialize, Deserialize)]
pub struct SaleItem {
    #[serde(serialize_with = "object_id_as_string")]
    pub product_id: ObjectId,
    pub product_name: String,
    pub sku: String,
    pub quantity: i32,
    pub price: f64,
    pub subtotal: f64
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Sale {
    #[serde(
        rename = "_id",
        skip_serializing_if = "Option::is_none",
        serialize_with = "opt_object_id_as_string"
    )]
    pub id: Option<ObjectId>,

    #[serde(serialize_with = "object_id_as_string")]
    pub user_id: ObjectId,
    pub customer_id: Option<ObjectId>,
    pub items: Vec<SaleItem>,

    pub total_amount: f64,

    pub paid_amount: f64,
    pub remaining_amount: f64,
    pub status: String, // "paid", "partial", "unpaid"

    pub invoice_number: Option<String>,

    pub payment_method: Option<PaymentMethod>,
    pub sale_date: Option<DateTime>,
    pub notes: Option<String>,

    #[serde(default)]
    pub created_at: Option<DateTime>,
    #[serde(default)]
    pub updated_at: Option<DateTime>,
}

#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct SaleItemDTO {
    pub product_id: ObjectId, // tidak divalidasi karena sudah pasti BSON ID yang valid

    #[validate(range(min = 1, message = "Jumlah item minimal 1"))]
    pub quantity: i32,
}

#[derive(Debug, Deserialize, Validate)]
pub struct SaleDTO {
    pub customer_id: Option<ObjectId>,

    #[validate(length(min = 1, message = "Daftar item tidak boleh kosong"))]
    #[validate(nested)]
    pub items: Vec<SaleItemDTO>,

    pub payment_method_id: Option<ObjectId>,

    #[validate(range(min = 0.0, message = "Jumlah bayar tidak boleh negatif"))]
    pub paid_amount: f64,

    #[validate(length(min = 0, max = 255, message = "Catatan maksimal 255 karakter"))]
    pub notes: Option<String>,
}




#[derive(Debug, Serialize)]
pub struct SaleResponse {
    pub id: String,
    pub user_id: String,

    pub customer_id: Option<String>,
    pub items: Vec<SaleItem>,

    pub total_amount: f64,

    pub paid_amount: f64,
    pub remaining_amount: f64,
    pub status: String,

    pub invoice_number: Option<String>,
    pub payment_method: Option<PaymentMethod>,
    pub sale_date: Option<String>,
    pub notes: Option<String>,

    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub updated_at: Option<String>,
}


impl From<Sale> for SaleResponse {
    fn from(sale: Sale) -> Self {
        SaleResponse {
            id: sale.id.expect("Sale.id harus ada").to_hex(),
            user_id: sale.user_id.to_hex(),
            customer_id: sale.customer_id.map(|id| id.to_hex()),
            items: sale.items,

            total_amount: sale.total_amount,

            paid_amount: sale.paid_amount,
            remaining_amount: sale.remaining_amount,
            status: sale.status,

            invoice_number: sale.invoice_number,
            payment_method: sale.payment_method,
            sale_date: sale.sale_date.map(|t| t.to_chrono().to_rfc3339()),
            notes: sale.notes,

            created_at: sale.created_at.map(|t| t.to_chrono().to_rfc3339()),
            updated_at: sale.updated_at.map(|t| t.to_chrono().to_rfc3339()),
        }
    }
}

