use std::collections::HashMap;
use actix_web::{web, get};
use actix_web::HttpResponse;
use repository::database::Database;
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

#[get("/instruments/search")]
pub async fn search_instruments(db: web::Data<Database>, query: web::Query<HashMap<String, String>>) -> HttpResponse {
    let search_term = query.get("q").unwrap_or(&"".to_string()).clone();
    match db.search_instruments(search_term).await {
        Ok(instrument_list) => {
            HttpResponse::Ok().json(instrument_list)
        },
        Err(error) => {
            eprintln!("error: {:?}", error);
            HttpResponse::NotFound().body("error".to_string())
        }
    }
}

#[get("/instruments/top-losers")]
pub async fn top_losers(db: web::Data<Database>) -> HttpResponse {
    match db.top_losers().await {
        Ok(instrument_list) => {
            HttpResponse::Ok().json(instrument_list)
        },
        Err(error) => {
            eprintln!("error: {:?}", error);
            HttpResponse::NotFound().body("error".to_string())
        }
    }
}


#[get("/instruments/top-gainers")]
pub async fn top_gainers(db: web::Data<Database>) -> HttpResponse {
    match db.top_gainers().await {
        Ok(instrument_list) => {
            HttpResponse::Ok().json(instrument_list)
        },
        Err(error) => {
            eprintln!("error: {:?}", error);
            HttpResponse::NotFound().body("error".to_string())
        }
    }
}


#[get("/instrument/{id}")]
pub async fn get_instrument_by_id(db: web::Data<Database>, id: web::Path<i64>) -> HttpResponse {
    let instrument_id = id.into_inner();
    let instrument = match db.get_instrument_by_id(instrument_id).await {
        Ok(instrument_detail) => {
            HttpResponse::Ok().json(instrument_detail)
        },
        Err(error) => {
            eprintln!("error: {:?}", error);
            HttpResponse::NotFound().body("error".to_string())
        }
    };

    instrument
}

// #[put("/instrument")]
// pub async fn update_instrument_by_id(db: web::Data<Database>, payload: web::Json<UpdatePayload>) -> HttpResponse {
//     let payload = payload.into_inner().clone();
    
//     if payload.client_id == "OMS_SERVER" {
//         match db.update_instrument_by_id(payload.instrument.instrument_id, payload.instrument.clone()).await {
//             Ok(_) => {
//                 HttpResponse::Ok().json("data base update success ")
//             },
//             Err(e) => {
//                 HttpResponse::NotFound().body(e.to_string())
//             }
//         }
//     } else {
//         HttpResponse::NotFound().body("error".to_string())
//     }

// }

#[get("/instrument/charts/{id}")]
pub async fn get_chart_data_by_id(db: web::Data<Database>, id: web::Path<i64>) -> HttpResponse {
    let instrument_id = id.into_inner();
    let instrument = match db.get_chart_data_by_id(instrument_id).await {
        Ok(chart_data) => {
            println!("chart_data {:?}", chart_data);
            HttpResponse::Ok().json(chart_data)
        },
        Err(error) => {
            eprintln!("error: {:?}", error);
            HttpResponse::NotFound().body("error".to_string())
        }
    };

    instrument
}


pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api")
            .service(get_instruments)
            // .service(update_instrument_by_id)
            .service(get_instrument_by_id)
            .service(search_instruments)
            .service(top_losers)
            .service(top_gainers)
            .service(get_chart_data_by_id)
    );
}
