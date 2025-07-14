use crate::models::sale::{Sale, SaleDTO, SaleItem, SaleResponse};
use actix_web::{
    Error as ActixError, HttpRequest, HttpResponse, Result,
    web::{Data, Json, Path},
};

use crate::errors::ApiError;
use crate::models::product::{ProductDTO, ProductResponse, UpdateProductDTO};
use crate::services::sale_service::{create_sale_service, get_sales_service};
use crate::utils::extract_user_id_from_cookie;
use mongodb::Database;
use validator::Validate;

pub async fn get_sales_handler(
    req: HttpRequest,
    db: Data<Database>,
) -> Result<HttpResponse, ApiError> {
    let user_id_str = extract_user_id_from_cookie(&req)?;
    let sales = get_sales_service(&db, &user_id_str).await?;

    let sales_response: Vec<SaleResponse> = sales.into_iter().map(SaleResponse::from).collect();

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "status": "success",
        "data": sales_response,
        "code": 200
    })))
}

pub async fn post_sale_handler(
    req: HttpRequest,
    payload: Result<Json<SaleDTO>, ActixError>,
    db: Data<Database>,
) -> Result<HttpResponse, ApiError> {
    let user_id_str = extract_user_id_from_cookie(&req)?;
    let data = payload?.into_inner();
    data.validate()?;

    let sale = create_sale_service(data, &db, &user_id_str).await?;

    Ok(HttpResponse::Created().json({
        serde_json::json!({
            "status" : "success",
            "data" : SaleResponse::from(sale),
            "code" : 201
        })
    }))
}
