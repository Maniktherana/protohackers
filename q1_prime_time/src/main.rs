use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use std::net::SocketAddr;
use serde::{Deserialize, Serialize};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("0.0.0.0:8080").await?;
    println!("Server listening on: {}", &listener.local_addr()?);

    loop {
        match listener.accept().await {
            Ok((mut socket, addr)) => {
                tokio::spawn(async move { handle_request(&mut socket, addr).await });
            }
            Err(e) => println!("Error accepting connection: {}", e),
        }
    }
}

async fn handle_request(socket: &mut TcpStream, address: SocketAddr) -> std::io::Result<()> {
    println!("New client: {}", address);

    let mut buf = vec![0u8; 1024];

    let mut i = 0;

    loop {
        println!("i is now {}", i);
        let bytes = match socket.read(&mut buf).await {
            Ok(n) if n == 0 => break, // End of stream, client closed connection
            Ok(n) => n,
            Err(e) => {
                eprintln!("Failed to read from socket: {}", e);
                return Err(e);
            }
        };

        let bytes_to_str = String::from_utf8_lossy(&buf[..bytes]);

        // Attempt to parse the incoming string as a Request
        let response = match parse_request(&bytes_to_str) {
            Ok(request) => {
                println!("{:?}", request);

                if request.method == "isPrime" {
                    Response {
                        method: request.method,
                        prime: Request::is_prime(request.number),
                    }
                } else {
                    Response {
                        method: String::from("malformed"),
                        prime: false,
                    }
                }
            }
            Err(_) => {
                // If parsing as Request fails, send a default Response
                Response {
                    method: String::from("malformed"),
                    prime: false,
                }
            }
        };

        println!("{:?}\n", response);

        let res_to_str = serde_json::to_string(&response).unwrap();
        let res_to_bytes = res_to_str.as_bytes();

        match socket.write_all(res_to_bytes).await {
            Ok(_) => {} // Data successfully written
            Err(e) => {
                eprintln!("Failed to write to socket: {}", e);
                return Err(e);
            }
        }
        i+=1;
    }

    println!("Closed connection: {}", address);
    Ok(())
}

#[derive(Serialize, Deserialize, Debug)]
struct Response {
    method: String,
    prime: bool,
}

#[derive(Serialize, Deserialize, Debug)]
struct Request {
    method: String,
    number: f64,
}

impl Request {
    fn is_prime(n: f64) -> bool {
        if n <= 1.0 || n.floor() != n {
            return false;
        }

        for i in 2..=((n as f32).sqrt() as i32) {
            println!("n is {}", i);
            if n % i as f64 == 0.0 {
                return false;
            }
        }

        return true;
    }
}

fn parse_request(body: &str) -> Result<Request, serde_json::Error> {
    serde_json::from_str::<Request>(body)
}
