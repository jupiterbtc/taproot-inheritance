use bdk::{database::MemoryDatabase, electrum_client::Client};
use bdk::bitcoin::{secp256k1, PrivateKey, Network};

fn generate_private_key() -> PrivateKey {
    let key = secp256k1::SecretKey::new(&mut secp256k1::rand::thread_rng());
    let private_key = PrivateKey {
        compressed: true,
        network: Network::Testnet,
        inner: key
    };
    private_key
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", generate_private_key());
    
    let cache = MemoryDatabase::new();
    let client = Client::new("ssl://electrum.blockstream.info:60002")?;

    Ok(())
}