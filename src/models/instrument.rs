use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Instrument_Code{
    pub symbol: String,
}
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Instrument{
    pub instrument_id: i32,
    pub code: String,
    pub symbol: String,
    pub last_price: f64,
    pub prev_price: f64,
    pub change: f64
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Instrument_Update{
    pub instrument_id: i32,
    pub last_price: f64,
    pub prev_price: f64,
    pub change: f64
}
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Instrument_Detail{
    pub instrument_id: i32,
    pub code: String,
    pub symbol: String,
    pub last_price: f64,
    pub prev_price: f64,
	pub change: f64,
	pub today_change: f64,
	pub day_7_change: f64,
	pub day_30_change: f64,
	pub day_90_change: f64,
	pub day_180_change: f64,
	pub year_1_change: f64,
	pub market_cap: f64,
	pub vol_24 : f64,
	pub high_24: f64,
	pub low_24: f64,
	pub total_vol: f64
}



#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct UpdatePayload {
    pub client_id: String,
    pub instrument: Instrument_Update,
}