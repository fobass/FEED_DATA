use actix_web::web::Payload;
use actix_web::{web, get, post, put, ResponseError, Error, Responder};
use actix_web::HttpResponse;
use repository::database::Database;
use crate::models::instrument::{UpdatePayload, self};
use crate::repository;

#[get("/instruments")]
pub async fn get_instruments(db: web::Data<Database>) -> HttpResponse {
    let instruments = match db.load().await {
        Ok(instrument_list) => {
            HttpResponse::Ok().json(instrument_list)
        },
        Err(error) => {
            eprintln!("error: {:?}", error);
            HttpResponse::NotFound().body("error".to_string())
        }
    };

    instruments
}

#[get("/instrument/{id}")]
pub async fn get_instrument_by_id(db: web::Data<Database>, id: web::Path<i32>) -> HttpResponse {
    let instrument_id = id.into_inner();
    let instrument = match db.get_instrument_by_id(instrument_id).await {
        Ok(instrument_list) => {
            HttpResponse::Ok().json(instrument_list)
        },
        Err(error) => {
            eprintln!("error: {:?}", error);
            HttpResponse::NotFound().body("error".to_string())
        }
    };

    instrument
}

#[put("/instrument")]
pub async fn update_instrument_by_id(db: web::Data<Database>, payload: web::Json<UpdatePayload>) -> HttpResponse {
    let payload = payload.into_inner().clone();
    
    if payload.client_id == "OMS_SERVER" {
        match db.update_instrument_by_id(payload.instrument.instrument_id, payload.instrument.clone()).await {
            Ok(_) => {
                HttpResponse::Ok().json("data base update success ")
            },
            Err(e) => {
                HttpResponse::NotFound().body(e.to_string())
            }
        }
    } else {
        HttpResponse::NotFound().body("error".to_string())
    }

}


pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api")
            .service(get_instruments)
            .service(update_instrument_by_id)
            .service(get_instrument_by_id)
    );
}
