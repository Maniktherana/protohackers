use tokio::net::{TcpListener, TcpStream};
use tokio::io::{self, AsyncReadExt, AsyncWriteExt};
use std::net::SocketAddr;
use serde::{Deserialize, Serialize};
use serde_json::from_str;

#[tokio::main]
async fn main() -> io::Result<()> {
    // Bind the listener to the address
    let listener = TcpListener::bind("0.0.0.0:8080").await?;
    
    println!("Server listening on: {}", &listener.local_addr()?);

    loop {
        match listener.accept().await {
            Ok((mut socket, addr)) => {
                tokio::spawn(async move { parse_request(&mut socket, addr).await });
            }
            Err(e) => println!("Error accepting connection: {}", e),
        }
    }
}

async fn parse_request(socket: &mut TcpStream, address: SocketAddr) -> io::Result<()> {
    println!("New client: {}", address);

    let (mut read, mut write) = socket.split();

    let mut buf = vec![0u8; 1024];
    loop {
        let bytes = read.read(&mut buf).await?;
        if bytes == 0 {
            // End of stream, client closed connection
            break;
        }
        
        let bytes_to_str = String::from_utf8(buf[..bytes].to_vec()).unwrap();
        
        // Attempt to parse the incoming string as a Request
        match from_str::<Request>(&bytes_to_str) {
            Ok(request) => {
                println!("Request: {:#?}", request);
                
                let response = if request.method == "isPrime" {
                    Response {
                        method: request.method,
                        prime: Request::is_prime(request.number),
                    }
                } else {
                    Response {
                        method: String::from("malformed"),
                        prime: false,
                    }
                };
                
                println!("Response: {:#?}", response);
                
                let res_to_str = serde_json::to_string(&response).unwrap();
                let res_to_bytes = res_to_str.as_bytes();
                
                write.write_all(res_to_bytes).await?;
                write.flush().await?;
            }
            Err(_) => {
                // If parsing as Request fails, send a default Response
                let response = Response {
                    method: String::from("malformed"),
                    prime: false,
                };
                
                println!("Malformed Request. Sending default Response: {:#?}", response);
                
                let res_to_str = serde_json::to_string(&response).unwrap();
                let res_to_bytes = res_to_str.as_bytes();
                
                write.write_all(res_to_bytes).await?;
                write.flush().await?;
            }
        }
    }
        
        println!("Closed connection: {}", address);
        Ok(())
    }

#[derive(Serialize, Deserialize, Debug)]
struct Response {
    method: String,
    prime: bool
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
struct Request {
    method: String,
    number: f64
}


impl Request {
    fn is_prime(n: f64) -> bool {
        if n <= 1.0 || n.floor() != n {
            return false;
        }
    
        for i in 2..=((n as f32).sqrt() as i32) {
            if n % i as f64 == 0.0 {
                return false;
            }
        }
    
        return true;
    }
}
