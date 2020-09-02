use std::net::{SocketAddr, UdpSocket};

mod peer_config;

// Bind a new socket with a local address
fn bind(
    endpoint_cfg: quinn::ServerConfig,
    local_addr: SocketAddr,
) -> Result<(quinn::Endpoint, quinn::Incoming), String> {
    let mut endpoint_builder = quinn::Endpoint::builder();
    
    let _ = endpoint_builder.listen(endpoint_cfg);

    match UdpSocket::bind(&local_addr) {
        Ok(udp) => endpoint_builder.with_socket(udp).map_err(|err| format!("{}", err)),
        Err(err) => Err(
            format!(
                "Could not bind to the user supplied port: {}! Error: {}",
                local_addr.port(),
                err
            )
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::{peer_config};
    use std::net::{SocketAddr, IpAddr};
    use futures::StreamExt;

    #[tokio::test]
    async fn simple_test() {

        let cert = rcgen::generate_simple_self_signed(vec![
            "MaidSAFE.net".to_string()
        ]).unwrap();

        let key = quinn::PrivateKey::from_der(&cert.serialize_private_key_der()).unwrap();
        let cert =  quinn::Certificate::from_der(&cert.serialize_der().unwrap()).unwrap();


        let endpoint_cfg =
            peer_config::new_our_cfg(30_000, 10_000, cert, key).unwrap();

        let client_cfg = peer_config::new_client_cfg(30_000, 10_000);
        let socket_a = SocketAddr::new("127.0.0.1".parse().unwrap(), "8000".parse().unwrap());
        let (endpoint_a, incoming_a) = super::bind(endpoint_cfg.clone(), socket_a).unwrap();
        
        let socket_b = SocketAddr::new("127.0.0.1".parse().unwrap(), "8001".parse().unwrap());
        let (endpoint_b, mut incoming_b) = super::bind(endpoint_cfg, socket_b).unwrap();

        let quinn::NewConnection {
            connection: quinn_conn,
            ..
        } = endpoint_a.connect_with(client_cfg.clone(), &socket_b, "QuinnDemo").unwrap().await.unwrap();

        let (mut send_a, recv_a) = quinn_conn.open_bi().await.unwrap();
        send_a.write(&vec![1, 2, 3, 4]).await.unwrap();
        send_a.finish().await.unwrap();
        let quinn::NewConnection {
            connection,
            mut uni_streams,
            mut bi_streams,
            ..
        } = incoming_b.next().await.unwrap().await.unwrap();
        
        let (send_b, mut recv_b) = bi_streams.next().await.unwrap().unwrap();
        
        // Data is always empty and len is None
        
        let mut data = Vec::new();
        let len = recv_b.read(&mut data).await.unwrap();
        println!("Read {:?}, Len {:?}", data, len);
        
        // Data is always empty
        
        let mut data = Vec::new();
        recv_b.read_exact(&mut data).await.unwrap();
        println!("Read: {:?}", data);
        
        // This works
        
        let data = recv_b.read_to_end(100).await.unwrap();
        println!("Read: {:?}", data);


    }
}
