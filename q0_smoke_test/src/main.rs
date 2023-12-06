use tokio::net::{TcpListener, TcpStream};
use tokio::io::copy;
use std::net::SocketAddr;
use std::io;

#[tokio::main]
async fn main() -> io::Result<()> {
    // Bind the listener to the address
    let listener = TcpListener::bind("0.0.0.0:8080").await?;
    
    println!("Server listening on: {}", &listener.local_addr()?);

    loop {
        match listener.accept().await {
            Ok((mut socket, addr)) => {
                tokio::spawn(async move { echo(&mut socket, addr).await });
            }
            Err(e) => println!("Error accepting connection: {}", e),
        }
    }
}

async fn echo(socket: &mut TcpStream, address: SocketAddr) -> io::Result<()> {
    println!("New client: {}", address);

    let (mut reader, mut writer) = socket.split();
    
    copy(&mut reader, &mut writer).await?;

    println!("Closed connection: {}", address);
    Ok(())
}
