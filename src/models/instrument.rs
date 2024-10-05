use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use core_graphics::geometry::CGPoint;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct InstrumentCode{
    pub symbol: String,
}
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SparkPoint {
    pub x: f64,
    pub y: f64,
}

impl From<CGPoint> for SparkPoint {
    fn from(point: CGPoint) -> Self {
        SparkPoint { x: point.x, y: point.y }
    }
}

impl From<SparkPoint> for CGPoint {
    fn from(gram_point: SparkPoint) -> Self {
        CGPoint::new(gram_point.x, gram_point.y)
    }
}
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Instrument{
    pub instrument_id: i64,
    pub code: String,
    pub symbol: String,
    pub last_price: f64,
    pub prev_price: f64,
    pub change: f64,
    pub volume: i64,
    pub spark: Vec<SparkPoint>
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct InstrumentUpdate{
    pub instrument_id: i32,
    pub last_price: f64,
    pub prev_price: f64,
    pub change: f64
}
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct InstrumentDetail{
    pub instrument_id: i64,
    // pub code: String,
    // pub symbol: String,
    // pub last_price: f64,
    // pub prev_price: f64,
	// pub change: f64,
	// pub today_change: f64,
	// pub day_7_change: f64,
	// pub day_30_change: f64,
	// pub day_90_change: f64,
	// pub day_180_change: f64,
	// pub year_1_change: f64,
	// pub market_cap: f64,
	pub vol_24 : f64,
	pub high_24: f64,
	pub low_24: f64,
	// pub total_vol: f64
}



#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct UpdatePayload {
    pub client_id: String,
    pub instrument: InstrumentUpdate,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ChartData {
    pub chart_data_id: i64,
    pub instrument_id: i64,
    pub open_price: f64,
    pub close_price: f64,
    pub high_price: f64,
    pub low_price: f64,
    pub volume: f64,
    pub timestamp: NaiveDateTime
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ChartInterval{
    Min1, 
    Min5,
    Min30, 
    Hour1,
    Day1,
    Month1
}

