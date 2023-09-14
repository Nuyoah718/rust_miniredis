#![feature(impl_trait_in_assoc_type)]

use std::net::SocketAddr;
use mini_redis::LogLayer;

use mini_redis::{S, DEFAULT_ADDR};

#[volo::main]
async fn main() {
    let addr: SocketAddr = DEFAULT_ADDR.parse().unwrap();
    let addr = volo::net::Address::from(addr);

    volo_gen::volo::example::ItemServiceServer::new(S::new())
        .layer_front(LogLayer)
        .run(addr)
        .await
        .unwrap();
}
