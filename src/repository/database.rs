use std::{sync::{Arc, Mutex}, str::FromStr};
use bigdecimal::{ToPrimitive, BigDecimal};
use chrono::format::Numeric;
use num_traits::FromPrimitive;
use pg_bigdecimal::PgNumeric;
use postgres::{ NoTls, Error};
use tokio_postgres::types::ToSql;

use crate::models::instrument::{Instrument, Instrument_Detail, Instrument_Update};

pub struct Database {
    pub instruments: Arc<Mutex<Vec<Instrument>>>,
}

impl Database {
    pub fn new() -> Self {
        let instruments = Arc::new(Mutex::new(vec![]));
        Database { instruments }
    }

    pub async fn push_to_instruments(&self, new_list: &[Instrument]) {
        let mut current_instrumnets = self.instruments.lock().unwrap();
        current_instrumnets.extend_from_slice(new_list);
    }

    pub async fn load(&self) -> Result<Vec<Instrument>, Error> {
        let instruments = self.instruments.lock().unwrap();
        if !instruments.is_empty() {
            return Ok(instruments.clone());
        }

        drop(instruments);

        let connection_string = "host=localhost user=postgres password=Clubmix081416 dbname=solgram";

        let (client, connection) = tokio_postgres::connect(connection_string, NoTls)
            .await?;
        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("connection error: {}", e);
            }
        });

        let rows = client.query("SELECT instrument_id, code, symbol, last_price, prev_price, change FROM market_data LIMIT 10", &[]).await?;
        let mut new_instruments = Vec::new();
        for row in rows {
            let last_price: PgNumeric = row.get(3);
            let prev_price: PgNumeric = row.get(4);
            let change: PgNumeric = row.get(5);
            let instrument = Instrument {
                instrument_id: row.get("instrument_id"),
                code: row.get("code"),
                symbol: row.get("symbol"),
                last_price: last_price.n.unwrap().clone().to_f64().unwrap(),
                prev_price: prev_price.n.unwrap().clone().to_f64().unwrap(),
                change: change.n.unwrap().clone().to_f64().unwrap()
            };

            new_instruments.push(instrument);
        }

        self.push_to_instruments(&new_instruments);
        Ok(new_instruments)
    }

    
    pub async fn update_instrument_by_id(&self, instrument_id: i32, instrument: Instrument_Update) -> Result<(), Error> {
        let connection_string = "host=localhost user=postgres password= dbname=solgram";

        let (client, connection) = tokio_postgres::connect(connection_string, NoTls)
        .await?;
        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("connection error: {}", e);
            }
        });
    
        let update_query = r#"
            UPDATE market_data
            SET last_price = $1, prev_price = $2
            WHERE instrument_id = $3
        "#;

        let last_price_big_decimal = BigDecimal::from_f64(instrument.last_price).expect("Failed to convert f64 to BigDecimal");
        let last_price = PgNumeric {
            n: Some(last_price_big_decimal),
        };

        let prev_price_big_decimal = BigDecimal::from_f64(instrument.prev_price).expect("Failed to convert f64 to BigDecimal");
        let prev_price = PgNumeric {
            n: Some(prev_price_big_decimal),
        };

        let params: &[&(dyn ToSql + Sync)] = &[
            &last_price,
            &prev_price,
            &instrument_id,
        ];
        
        client.execute(update_query, params).await?;
    
        Ok(())
    
    }
    

    pub async fn get_instrument_by_id(&self, instrument_id: i32) -> Result<Vec<Instrument_Detail>, Error> {
        let connection_string = "host=localhost user=postgres password=Clubmix081416 dbname=solgram";

        let (client, connection) = tokio_postgres::connect(connection_string, NoTls)
            .await?;
        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("connection error: {}", e);
            }
        });

        let rows = client.query("SELECT * FROM market_data WHERE instrument_id = $1", &[&instrument_id]).await?;
        let mut new_instruments = Vec::new();
        for row in rows {
            let last_price:     PgNumeric = row.get(3);
            let prev_price:     PgNumeric = row.get(4);
            let change:         PgNumeric = row.get(5);
            let today_change:   PgNumeric = row.get(6);
            let day_7_change:   PgNumeric = row.get(7);
            let day_30_change:  PgNumeric = row.get(8);
            let day_90_change:  PgNumeric = row.get(9);
            let day_180_change: PgNumeric = row.get(10);
            let year_1_change:  PgNumeric = row.get(11);
            let market_cap:     PgNumeric = row.get(12);

            let vol_24:         PgNumeric = row.get(13);
            let high_24:        PgNumeric = row.get(14);
            let low_24:         PgNumeric = row.get(15);
            let total_vol:      PgNumeric = row.get(16);

            let instrument = Instrument_Detail {
                instrument_id: row.get("instrument_id"),
                code: row.get("code"),
                symbol: row.get("symbol"),
                last_price:     last_price.n.unwrap().clone().to_f64().unwrap(),
                prev_price:     prev_price.n.unwrap().clone().to_f64().unwrap(),
                change:         change.n.unwrap().clone().to_f64().unwrap(),
                today_change:   today_change.n.unwrap().clone().to_f64().unwrap(),
                day_7_change:   day_7_change.n.unwrap().clone().to_f64().unwrap(),
                day_30_change:  day_30_change.n.unwrap().clone().to_f64().unwrap(),
                day_90_change:  day_90_change.n.unwrap().clone().to_f64().unwrap(),
                day_180_change: day_180_change.n.unwrap().clone().to_f64().unwrap(),
                year_1_change:  year_1_change.n.unwrap().clone().to_f64().unwrap(),
                market_cap:     market_cap.n.unwrap().clone().to_f64().unwrap(),
                vol_24:         vol_24.n.unwrap().clone().to_f64().unwrap(),
                high_24:        high_24.n.unwrap().clone().to_f64().unwrap(),
                low_24:         low_24.n.unwrap().clone().to_f64().unwrap(),
                total_vol:      total_vol.n.unwrap().clone().to_f64().unwrap(),
            };

            new_instruments.push(instrument);
        }
        Ok(new_instruments)
    }
    
}
