use std::str::FromStr;
use std::error::Error;

use tokio;
use tokio::runtime::Runtime;
use trust_dns_server::store::forwarder::{ForwardAuthority, ForwardConfig};
use trust_dns_server::proto::rr::Name;
use trust_dns_resolver::config::NameServerConfigGroup;
use trust_dns_server::authority::{ZoneType, Authority, LookupObject, AuthorityObject, Catalog};
use trust_dns_resolver::proto::rr::RecordType;
use trust_dns_client::rr::LowerName;
use trust_dns_server::server::ServerFuture;
use std::net::UdpSocket;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut runtime = Runtime::new().expect("failed to create Tokio Runtime");

    let config = ForwardConfig {
        name_servers: NameServerConfigGroup::cloudflare_tls(),
        options: None,
    };

    let forwarder = ForwardAuthority::try_from_config(
        Name::root().into(),
        ZoneType::Forward,
        &config,
        runtime.handle().clone()
    ).await?;

    let mut catalog = Catalog::new();
    catalog.upsert(Name::root().into(), Box::new(forwarder));

    let socket = UdpSocket::bind("127.0.0.1:53").expect("fail udp socket");
    let mut server = ServerFuture::new(catalog);
    server.register_socket_std(socket, &runtime);
    server.block_until_done().await?;

    Ok(())
}
