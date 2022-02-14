use std::str::FromStr;

use clickr::{Reader, ClientOptions};
use serde_json::Value;

fn main(){
    let ch_host = "http://localhost:8124/?";
    let client_options = ClientOptions::new(ch_host, "bytebeam");
    let reader = Reader::new(client_options);
    let query = String::from_str("select toUnixTimestamp(timestamp) from Motor_Status_1").unwrap();
    let query = query.as_bytes();
    let response = reader.query(query).unwrap();
    let response: Value = response.into_json().unwrap();
    println!("{:?}", response["data"]);
}