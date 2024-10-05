use std::{env, sync::{Arc, Mutex}};
use bigdecimal::ToPrimitive;
// use chrono::NaiveDateTime;
use pg_bigdecimal::PgNumeric;
use tokio_postgres::{ Error, NoTls};
use dotenv::dotenv;
use tokio_postgres::Client;

use crate::models::instrument::{ChartData, Instrument, InstrumentDetail, SparkPoint};


// fn pg_numeric_to_f64(numeric: PgNumeric) -> f64 {
//     numeric.n.unwrap().to_f64().unwrap()
// }

pub struct Database {
    pub instruments: Arc<Mutex<Vec<Instrument>>>,
}

impl Database {
    pub fn new() -> Self {
        let instruments = Arc::new(Mutex::new(vec![]));
        Database { instruments }
    }

    async fn get_db_client() -> Result<Client, Error> {
        dotenv().ok();
        let connection_string = env::var("DATABASE_URL")
            .expect("DATABASE_URL must be set");
    
        let (client, connection) = tokio_postgres::connect(&connection_string, NoTls)
            .await?;
    
        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("connection error: {}", e);
            }
        });
    
        Ok(client)
    }

    pub async fn load(&self) -> Result<Vec<Instrument>, Error> {
        let client = Database::get_db_client().await?;

        let rows = client.query("SELECT instrument_id, code, symbol, last_price, prev_price, change, volume FROM market_data ORDER BY instrument_id ASC LIMIT 5", &[]).await?;
        let mut new_instruments = Vec::new();
        for row in rows {
            let last_price: PgNumeric = row.get(3);
            let prev_price: PgNumeric = row.get(4);
            let change: PgNumeric = row.get(5);
            let instrument_id: i64 = row.get("instrument_id");
            let spark_rows = client
                .query(
                    "SELECT price FROM market_price_history WHERE instrument_id = $1 AND recorded_at >= NOW() - INTERVAL '24 hours' ORDER BY recorded_at ASC LIMIT 20",
                    &[&instrument_id]
                )
                .await?;
            let spark: Vec<SparkPoint> = spark_rows
                .iter()
                .enumerate()
                .map(|(i, spark_row)| {
                    let price: PgNumeric = spark_row.get("price"); 
                    let x_value = i as f64;
                    let y_value = price.n.unwrap().clone().to_f64().unwrap();
                    SparkPoint {
                        x: x_value,
                        y: y_value,
                    }
                })
                .collect();
            
            let instrument = Instrument {
                instrument_id: row.get("instrument_id"),
                code: row.get("code"),
                symbol: row.get("symbol"),
                last_price: last_price.n.unwrap().clone().to_f64().unwrap(),
                prev_price: prev_price.n.unwrap().clone().to_f64().unwrap(),
                change: change.n.unwrap().clone().to_f64().unwrap(),
                volume: row.get("volume"),
                spark
            };

            new_instruments.push(instrument);
        }

        Ok(new_instruments)
    }

    pub async fn get_chart_data_by_id(&self, instrument_id: i64) -> Result<Vec<ChartData>, Error> {
        let client = Database::get_db_client().await?;

        // let query: String = "".to_owned();

        let rows = client.query(
            "WITH grouped_data AS (
                SELECT
                    date_trunc('hour', timestamp) AS hour_timestamp,
                    instrument_id,
                    MIN(timestamp) AS first_timestamp,
                    MAX(timestamp) AS last_timestamp,
                    MIN(low_price) AS low_price,
                    MAX(high_price) AS high_price,
                    SUM(volume) AS volume
                FROM
                    market_data_chart
                WHERE
                    timestamp >= '2024-10-03 08:02:00'
                    AND timestamp <= '2024-10-03 17:02:59'
                    AND instrument_id = $1
                GROUP BY
                    date_trunc('hour', timestamp), instrument_id
            )
            SELECT
                (SELECT open_price FROM market_data_chart WHERE instrument_id = gd.instrument_id AND timestamp = gd.first_timestamp) AS open_price,
                (SELECT close_price FROM market_data_chart WHERE instrument_id = gd.instrument_id AND timestamp = gd.last_timestamp) AS close_price,
                (SELECT chart_data_id FROM market_data_chart WHERE instrument_id = gd.instrument_id AND timestamp = gd.last_timestamp) AS chart_data_id,
                gd.instrument_id,
                gd.high_price,
                gd.low_price,
                gd.volume,
                gd.hour_timestamp
            FROM
                grouped_data gd
            ORDER BY
                gd.hour_timestamp ASC;
            
            
            ", 
             &[&instrument_id]
        ).await?;

        let mut chart_datas = Vec::new();
        for row in rows {
            let instrument_id: i64 = row.get("instrument_id");
            let open_price: PgNumeric = row.get("open_price");
            let close_price: PgNumeric = row.get("close_price");
            let high_price: PgNumeric = row.get("high_price");
            let low_price: PgNumeric = row.get("low_price");
            let volume: PgNumeric = row.get("volume");

            let chart_data = ChartData {
                instrument_id: instrument_id,
                open_price: open_price.n.unwrap().clone().to_f64().unwrap(),
                close_price: close_price.n.unwrap().clone().to_f64().unwrap(),
                high_price: high_price.n.unwrap().clone().to_f64().unwrap(),
                low_price: low_price.n.unwrap().clone().to_f64().unwrap(),
                volume: volume.n.unwrap().clone().to_f64().unwrap(),
                timestamp: row.get("hour_timestamp"),
                chart_data_id: row.get("chart_data_id"),
            };
            chart_datas.push(chart_data);       
        }

        Ok(chart_datas)
    }
    
    pub async fn top_losers(&self) -> Result<Vec<Instrument>, Error> {
        let client = Database::get_db_client().await?;

        let rows = client.query(
            "SELECT instrument_id, code, symbol, last_price, prev_price, change, volume
            FROM market_data 
            WHERE change < 0.0
            ORDER BY change ASC 
            LIMIT 15", 
             &[]
        ).await?;

        
        let mut new_instruments = Vec::new();
        for row in rows {
            let last_price: PgNumeric = row.get(3);
            let prev_price: PgNumeric = row.get(4);
            let change: PgNumeric = row.get(5);
            let instrument_id: i64 = row.get("instrument_id");
            let spark_rows = client
                .query(
                    "SELECT price FROM market_price_history WHERE instrument_id = $1 AND recorded_at >= NOW() - INTERVAL '24 hours' ORDER BY recorded_at ASC LIMIT 20",
                    &[&instrument_id]
                )
                .await?;
            let spark: Vec<SparkPoint> = spark_rows
                .iter()
                .enumerate()
                .map(|(i, spark_row)| {
                    let price: PgNumeric = spark_row.get("price"); 
                    let x_value = i as f64;
                    let y_value = price.n.unwrap().clone().to_f64().unwrap();
                    SparkPoint {
                        x: x_value,
                        y: y_value,
                    }
                })
                .collect();
            let instrument = Instrument {
                instrument_id: row.get("instrument_id"),
                code: row.get("code"),
                symbol: row.get("symbol"),
                last_price: last_price.n.unwrap().clone().to_f64().unwrap(),
                prev_price: prev_price.n.unwrap().clone().to_f64().unwrap(),
                change: change.n.unwrap().clone().to_f64().unwrap(),
                volume: row.get("volume"),
                spark
            };

            new_instruments.push(instrument);
        }

        Ok(new_instruments)
    }


    pub async fn top_gainers(&self) -> Result<Vec<Instrument>, Error> {
        let client = Database::get_db_client().await?;

        let rows = client.query(
            "SELECT instrument_id, code, symbol, last_price, prev_price, change, volume
            FROM market_data 
            WHERE change > 0.0 
            ORDER BY change DESC 
            LIMIT 15", 
             &[]
        ).await?;

        
        let mut new_instruments = Vec::new();
        for row in rows {
            let last_price: PgNumeric = row.get(3);
            let prev_price: PgNumeric = row.get(4);
            let change: PgNumeric = row.get(5);
            let instrument_id: i64 = row.get("instrument_id");
            let spark_rows = client
                .query(
                    "SELECT price FROM market_price_history WHERE instrument_id = $1 AND recorded_at >= NOW() - INTERVAL '24 hours' ORDER BY recorded_at ASC LIMIT 20",
                    &[&instrument_id]
                )
                .await?;
            let spark: Vec<SparkPoint> = spark_rows
                .iter()
                .enumerate()
                .map(|(i, spark_row)| {
                    let price: PgNumeric = spark_row.get("price"); 
                    let x_value = i as f64;
                    let y_value = price.n.unwrap().clone().to_f64().unwrap();
                    SparkPoint {
                        x: x_value,
                        y: y_value,
                    }
                })
                .collect();
            let instrument = Instrument {
                instrument_id: row.get("instrument_id"),
                code: row.get("code"),
                symbol: row.get("symbol"),
                last_price: last_price.n.unwrap().clone().to_f64().unwrap(),
                prev_price: prev_price.n.unwrap().clone().to_f64().unwrap(),
                change: change.n.unwrap().clone().to_f64().unwrap(),
                volume: row.get("volume"),
                spark
            };

            new_instruments.push(instrument);
        }

        Ok(new_instruments)
    }

    pub async fn search_instruments(&self, search_term: String) -> Result<Vec<Instrument>, Error> {
        let client = Database::get_db_client().await?;

        let search_term = format!("%{}%", search_term);
        print!("search_instruments {:?}",search_term );
        let rows = client.query(
            "SELECT instrument_id, code, symbol, last_price, prev_price, change, volume
             FROM market_data 
             WHERE code LIKE $1 OR symbol LIKE $1
             LIMIT 15", 
             &[&search_term]
        ).await?;

        
        let mut new_instruments = Vec::new();
        for row in rows {
            let last_price: PgNumeric = row.get(3);
            let prev_price: PgNumeric = row.get(4);
            let change: PgNumeric = row.get(5);
            let instrument_id: i64 = row.get("instrument_id");
            let spark_rows = client
                .query(
                    "SELECT price FROM market_price_history WHERE instrument_id = $1 AND recorded_at >= NOW() - INTERVAL '24 hours' ORDER BY recorded_at ASC LIMIT 20",
                    &[&instrument_id]
                )
                .await?;
            let spark: Vec<SparkPoint> = spark_rows
                .iter()
                .enumerate()
                .map(|(i, spark_row)| {
                    let price: PgNumeric = spark_row.get("price"); 
                    let x_value = i as f64;
                    let y_value = price.n.unwrap().clone().to_f64().unwrap();
                    SparkPoint {
                        x: x_value,
                        y: y_value,
                    }
                })
                .collect();
            let instrument = Instrument {
                instrument_id: row.get("instrument_id"),
                code: row.get("code"),
                symbol: row.get("symbol"),
                last_price: last_price.n.unwrap().clone().to_f64().unwrap(),
                prev_price: prev_price.n.unwrap().clone().to_f64().unwrap(),
                change: change.n.unwrap().clone().to_f64().unwrap(),
                volume: row.get("volume"),
                spark
            };

            new_instruments.push(instrument);
        }

        Ok(new_instruments)
    }

    pub async fn get_instrument_by_id(&self, instrument_id: i64) -> Result<InstrumentDetail, Error> {
        let client = Database::get_db_client().await?;

        let row = client.query_one("
            SELECT 
                instrument_id,
                MAX(high_price) AS \"24High\",
                MIN(low_price) AS \"24Low\",
                SUM(volume) AS \"24Volume\"
            FROM 
                market_data_chart
            WHERE 
                instrument_id = $1 
                AND timestamp >= NOW() - INTERVAL '24 HOURS'
            GROUP BY 
                instrument_id;
            ", &[&instrument_id]).await?;

            let high_24: PgNumeric = row.get(1);
            let low_24:  PgNumeric = row.get(2);
            let vol_24:  PgNumeric = row.get(3);

            let instrument_detail = InstrumentDetail {
                instrument_id: row.get("instrument_id"),
                vol_24:         vol_24.n.unwrap().clone().to_f64().unwrap(),
                high_24:        high_24.n.unwrap().clone().to_f64().unwrap(),
                low_24:         low_24.n.unwrap().clone().to_f64().unwrap(),
            };
        Ok(instrument_detail)
    }
    
}
