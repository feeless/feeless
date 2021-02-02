#![feature(result_contains_err)]
#![forbid(unsafe_code)]

mod header;

use tokio::io::AsyncReadExt;
use tokio::net::TcpStream;

use feeless;

#[tokio::main]
async fn main() {
    let addr = "localhost:7075";
    let mut stream = TcpStream::connect(addr).await.unwrap();
    let (r, w) = stream.split();

    // r.read()
}
