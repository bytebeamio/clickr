use clickr::{Reader, ClientOptions};

fn main() {
    let ch_host = "http://localhost:8124/";
    let mut client_options = ClientOptions::new(ch_host, "bytebeam");
    let mut reader = Reader::new(client_options);
    let response = reader.end().unwrap();
    println!("{:?}", response);
}