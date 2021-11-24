use serde::{Deserialize, Serialize};
use std::{net::SocketAddr, path::PathBuf};

#[derive(Deserialize, Debug)]
pub struct EnvConfig {
    pub db_host: String,
    pub db_port: u16,
    pub db_name: String,
    pub db_user: String,
    pub db_password: String,
    pub db_cert: String,
    pub listen_addr: SocketAddr,
    pub listen_password: String,
    pub setup_csv: Option<PathBuf>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Data {
    pub tip_b: f64,
    pub root_b: f64,
    pub span_b: f64,
    pub sweep_b: f64,
    pub body_length_b: f64,
    pub tip_s: f64,
    pub root_s: f64,
    pub span_s: f64,
    pub sweep_s: f64,
    pub body_length_s: f64,
    pub body_diameter_bs: f64,
    pub mach_number: f64,
    pub power_on_bs: Option<f64>,
    pub power_off_bs: Option<f64>,
    pub power_on_s: Option<f64>,
    pub power_off_s: Option<f64>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OptionalData {
    pub power_on_bs: f64,
    pub power_off_bs: f64,
    pub power_on_s: f64,
    pub power_off_s: f64,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub(crate) struct RowId(pub i64);

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct GetResponse {
    pub id: RowId,
    pub data: Data,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct PostRequest {
    pub id: RowId,
    pub data: OptionalData,
}
