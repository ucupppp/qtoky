use crate::models::product::Product;
use log::error;
use crate::errors::ServiceError;
use futures::stream::TryStreamExt;
use crate::utils::{handle_duplicate_key_error, string_id_to_obj_id};
use bson::datetime::DateTime as BsonDateTime;
use chrono::Utc;
use mongodb::{Collection, Database, bson::doc};
use crate::models::sale::{Sale, SaleItem, SaleDTO};
use crate::models::payment_method::{PaymentMethod};

pub async fn get_sales_service(db: &Database, id:&str) -> Result<Vec<Sale>, ServiceError>{
    let user_id = match string_id_to_obj_id(id) {
        Some(oid) => oid,
        None => return Err(ServiceError::InvalidId("Invalid ID".into())),
    };
    let collection: Collection<Sale> = db.collection("sales");

    let mut cursor = collection
        .find(doc! {"user_id":user_id})
        .await
        .map_err(|e| ServiceError::DatabaseError(e.to_string()))?;
    
    let mut sales: Vec<Sale> = Vec::new();

    while let Some(sale) = cursor
        .try_next()
        .await
        .map_err(|e| ServiceError::DatabaseError(e.to_string()))?
    {
        sales.push(sale);
    }

    Ok(sales)
}


pub async fn create_sale_service(
    payload: SaleDTO,
    db: &Database,
    id: &str,
) -> Result<Sale, ServiceError> {
    let user_id = match string_id_to_obj_id(id) {
        Some(oid) => oid,
        None => return Err(ServiceError::InvalidId("Invalid ID".into())),
    };

    if payload.items.is_empty() {
        return Err(ServiceError::BadRequest("Items tidak boleh kosong".into()));
    }
    
    if payload.paid_amount < 0.0 {
        return Err(ServiceError::BadRequest("Paid amount tidak boleh negatif".into()));
    }
    
    let product_collection: Collection<Product> = db.collection("products");
    let mut sale_items: Vec<SaleItem> = Vec::new();
    let mut total_amount = 0.0;
    
    for item_dto in &payload.items {
        let filter = doc! { "_id": item_dto.product_id, "user_id": &user_id };
        
        // Ambil detail produk dari DB
        let product = product_collection
            .find_one(filter)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))?
            .ok_or_else(|| ServiceError::NotFound("Produk tidak ditemukan".to_string()))?;
        
        let actual_price = product.price;
        
        let subtotal = actual_price * item_dto.quantity as f64;
        total_amount += subtotal;
        
        sale_items.push(SaleItem {
            product_id: item_dto.product_id.clone(),
            product_name: product.name,
            sku: product.sku,
            quantity: item_dto.quantity,
            price: product.price,
            subtotal
        });
    }

    let payment_method: Option<PaymentMethod>;
    let pm_collection: Collection<PaymentMethod> = db.collection("payment_methods");

    let payment_method = match &payload.payment_method_id {
        Some(payment_method_id) => {
            let found_method = pm_collection
                .find_one(doc! { "_id": payment_method_id.clone() })
                .await
                .map_err(|e| ServiceError::DatabaseError(e.to_string()))?
                .ok_or_else(|| ServiceError::NotFound(format!(
                    "Metode Pembayaran dengan ID '{}' tidak ditemukan",
                    payment_method_id
                )))?;

            if !found_method.is_active {
                return Err(ServiceError::BadRequest(
                    "Metode Pembayaran sedang tidak aktif".to_string(),
                ));
            }

            Some(found_method)
        }
        None => None,
    };

    let final_amount = total_amount;
    let remaining_amount = total_amount - payload.paid_amount;
    let now = BsonDateTime::from_chrono(Utc::now());
    
    let sale = Sale {
        id: None,
        user_id,
        customer_id: payload.customer_id.clone(),
        items: sale_items,
        total_amount,
        paid_amount: payload.paid_amount,
        remaining_amount,
        status: if remaining_amount <= 0.0 {
            "paid".to_string()
        } else if payload.paid_amount > 0.0 {
            "partial".to_string()
        } else {
            "unpaid".to_string()
        },
        invoice_number: None,
        payment_method,
        sale_date: Some(now.clone()),
        notes: payload.notes.clone(),
        created_at: Some(now.clone()),
        updated_at: Some(now),
    };
    
    let collection: Collection<Sale> = db.collection("sales");
    let result = collection.insert_one(&sale).await;
    
    match result {
        Ok(insert_result) => {
            Ok(Sale {
                id: insert_result.inserted_id.as_object_id().map(|oid| oid.to_owned()),
                ..sale
            })
        }
        Err(e) => {
            if let Some(err) = handle_duplicate_key_error(&e) {
                return Err(err);
            }
            Err(ServiceError::DatabaseError(e.to_string()))
        }
    }
}


/* pub async fn validate_stock_availability(
    db: &Database, 
    items: &[SaleItemDTO],
    user_id: &ObjectId
) -> Result<(), ServiceError> {
    let product_collection: Collection<Product> = db.collection("products");
    
    for item in items {
        let filter = doc! { "_id": item.product_id, "user_id": user_id };
        let product = product_collection
            .find_one(filter)
            .await
            .map_err(|e| ServiceError::DatabaseError(e.to_string()))?
            .ok_or_else(|| ServiceError::NotFound("Produk tidak ditemukan".to_string()))?;
        
        // Cek stock jika field stock ada di Product model
        if let Some(stock) = product.stock {
            if stock < item.quantity {
                return Err(ServiceError::BadRequest(
                    format!("Stok {} tidak mencukupi", product.name)
                ));
            }
        }
    }
    
    Ok(())
} */
