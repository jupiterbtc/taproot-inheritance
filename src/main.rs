use std::collections::BTreeMap;
use std::str::FromStr;

use bdk::{KeychainKind, SignOptions};
use bdk::bitcoin::{Network};
use bdk::bitcoin::{secp256k1, PrivateKey};

// fn generate_private_key() -> PrivateKey {
//     let key = secp256k1::SecretKey::new(&mut secp256k1::rand::thread_rng());
//     let private_key = PrivateKey {
//         compressed: true,
//         network: Network::Testnet,
//         inner: key
//     };
//     private_key
// }

use bdk::{Wallet, database::MemoryDatabase, FeeRate, wallet::tx_builder::TxOrdering};
use bitcoin::consensus::encode::serialize_hex;
use bitcoin::{util::taproot, Address, hashes::hex::{ToHex}};
use secp256k1::{Secp256k1};

fn create_taproot_wallet() -> Result<Wallet<MemoryDatabase>, Box<dyn std::error::Error>> {
    use std::{sync::Arc};

    use bdk::{blockchain::{ElectrumBlockchain, ConfigurableBlockchain, ElectrumBlockchainConfig}, SyncOptions};

    let config = ElectrumBlockchainConfig {
        url: "ssl://electrum.blockstream.info:60002".to_string(),
        socks5: None,
        retry: 10,
        timeout: None,
        stop_gap: 100,
    };
    let blockchain = Arc::new(ElectrumBlockchain::from_config(&config)?);
    let unspendable_key = bitcoin::PublicKey::from_str("020000000000000000000000000000000000000000000000000000000000000001").unwrap();
    let taproot_key = bitcoin::PrivateKey::from_str("cMsqXifnUoZT3gvMWroHjKcVsJjg4zQ6fpSoRjc8mqNTFwDkth7Q").unwrap();
    let taproot_key_2 = bitcoin::PrivateKey::from_str("cQvVaHWhnDvkrqMFrcdK5CrB28CNiDeEDMordRBkLLfjmGMvsR8F").unwrap();
    let taproot_wallet = Wallet::new(
        bdk::descriptor!(tr(unspendable_key,multi_a(1,taproot_key,taproot_key_2)))?,
        None,
        Network::Testnet,
        MemoryDatabase::new(),
    )?;

    taproot_wallet.sync(&blockchain, SyncOptions::default())?;

    Ok(taproot_wallet)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let fee_rate = FeeRate::from_sat_per_vb(0.0);
    let wallet = create_taproot_wallet().unwrap();

    let secp = Secp256k1::new();
    let private_key= PrivateKey::from_str("cW5eyTngvsoUn4zWdAbN4hnLoeDvgQZgUnLJGazwjSxPAvvZexwp").unwrap();
    let key_pair = bitcoin::KeyPair::from_seckey_slice(&secp, &private_key.to_bytes()).unwrap();
    let to_internal_key = key_pair.public_key(); 
    println!("To public key: {}", to_internal_key);

    // Source: Arik
    let builder = taproot::TaprootBuilder::new();
    let tap_tree = builder.finalize(&secp, to_internal_key).unwrap();
    let output_details = Address::p2tr(&secp, to_internal_key, tap_tree.merkle_root(), bitcoin::Network::Regtest);
    let address = output_details.to_string();
    let output_script = output_details.script_pubkey();
    println!("{}", address); //bcrt1pkss5ldy4p5gfjlu9v7w075tp42xvsdnvn4zcvzrjlrqllu8ne5jsf3ccv3
    println!("{}", output_script.to_hex()); //5120b4214fb4950d10997f85679cff5161aa8cc8366c9d45860872f8c1fff0f3cd25

    let mut path = BTreeMap::new();
    path.insert("mm06tvmp".to_string(), vec![0, 1, 2]);

    // // A full list of APIs offered by `TxBuilder` can be found at
    // // https://docs.rs/bdk/latest/bdk/wallet/tx_builder/struct.TxBuilder.html
    let (mut send_back_psbt, _details) = {
        let mut builder = wallet.build_tx();
        builder
            .add_recipient(output_script, 100_000)
            .policy_path(path, KeychainKind::External)
            .add_data("Pleb.Fi LA is awesome~".as_bytes())
            .ordering(TxOrdering::Untouched)
            .enable_rbf()
            .fee_rate(fee_rate);
        builder.finish()?
    };

    let finalized = wallet.sign(&mut send_back_psbt, SignOptions::default())?;
    dbg!(finalized);

    let send_back_tx = send_back_psbt.extract_tx();
    println!("send_back_tx: {}", serialize_hex(&send_back_tx));

    // wallet.broadcast(&send_back_tx)?;

    Ok(())
}