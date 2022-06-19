use std::collections::BTreeMap;
use std::str::FromStr;

use bdk::KeychainKind;
use bdk::bitcoin::{Network};
use bdk::bitcoin::{secp256k1, PrivateKey};

fn generate_private_key() -> PrivateKey {
    let key = secp256k1::SecretKey::new(&mut secp256k1::rand::thread_rng());
    let private_key = PrivateKey {
        compressed: true,
        network: Network::Testnet,
        inner: key
    };
    private_key
}


// ========== ==========
// =====================
use bdk::{Wallet, database::MemoryDatabase, FeeRate, wallet::tx_builder::TxOrdering};
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
    // println!("{}", generate_private_key());
    // println!("{:?}", create_taproot_wallet());
    let fee_rate = FeeRate::from_sat_per_vb(1.0);
    let wallet = create_taproot_wallet().unwrap();

    let secp = Secp256k1::new();
    let private_key= PrivateKey::from_str("cW5eyTngvsoUn4zWdAbN4hnLoeDvgQZgUnLJGazwjSxPAvvZexwp").unwrap();
    let key_pair = bitcoin::KeyPair::from_seckey_slice(&secp, &private_key.to_bytes()).unwrap();
    let to_internal_key = key_pair.public_key(); 

    println!("To public key: {}", to_internal_key);

    let builder = taproot::TaprootBuilder::new();
    let tap_tree = builder.finalize(&secp, to_internal_key).unwrap();

    let output_details = Address::p2tr(&secp, to_internal_key, tap_tree.merkle_root(), bitcoin::Network::Regtest);
    let address = output_details.to_string();
    let output_script = output_details.script_pubkey().to_hex();
    println!("{}", address);
    // println!("{}", output_script);

    let mut path = BTreeMap::new();
    path.insert("mm06tvmp".to_string(), vec![0, 1]);

    // // A full list of APIs offered by `TxBuilder` can be found at
    // // https://docs.rs/bdk/latest/bdk/wallet/tx_builder/struct.TxBuilder.html
    let (mut send_back_psbt, details) = {
        let mut builder = wallet.build_tx();
        builder
            .policy_path(path, KeychainKind::External)
            .add_data("Pleb.Fi LA is awesome~".as_bytes())
            .ordering(TxOrdering::Untouched)
            .drain_wallet()
            .enable_rbf()
            .fee_rate(fee_rate);
        builder.finish()?
    };

    // For a regular transaction, just set the recipient and amount
    // tx_builder.set_recipients(vec![(core_address.script_pubkey(), 500000000)]);
    Ok(())
}


// use bitcoin::hashes::hex::{FromHex, ToHex};
// use bitcoin::{Address, OutPoint, SchnorrSighashType, Transaction, Txid, TxIn, TxOut, Witness};
// use bitcoin::psbt::serialize::Serialize;
// use bitcoin::schnorr::{TapTweak};
// use bitcoin::secp256k1::Secp256k1;
// use bitcoin::util::sighash::Prevouts;
// use bitcoin::util::taproot;
// use secp256k1::{Message};

// fn main() {
//     // DANGER
//     let secp = Secp256k1::new();
//     let private_key_slice: Vec<u8> = FromHex::from_hex("abbaabbaabbaabbaabbaabbaabbaabbaabbaabbaabbaabbaabbaabbaabbaabba").unwrap();
//     let private_key = bitcoin::KeyPair::from_seckey_slice(&secp, &private_key_slice).unwrap();
//     let internal_key = private_key.public_key();
//     println!("Internal key: {}", internal_key);

//     let builder = taproot::TaprootBuilder::new();
//     let tap_tree = builder.finalize(&secp, internal_key).unwrap();

//     let output_details = Address::p2tr(&secp, internal_key, tap_tree.merkle_root(), bitcoin::Network::Regtest);
//     let address = output_details.to_string();
//     let output_script = output_details.script_pubkey().to_hex();
//     println!("{}", address);
//     println!("{}", output_script);

//     let previous_output = OutPoint::new(Txid::from_hex("991ab2b13f6bc6c13002d79d5e9775626a5e7328e14cd16837d50d1cc637dc6a").unwrap(), 0);
//     let tx_input = TxIn {
//         previous_output,
//         script_sig: Default::default(),
//         sequence: 0xffffffff,
//         witness: Default::default()
//     };

//     let tx_output = TxOut {
//         value: 49_9999_5000,
//         script_pubkey: output_details.script_pubkey()
//     };

//     let mut transaction = Transaction {
//         version: 2,
//         lock_time: 0,
//         input: vec![tx_input],
//         output: vec![tx_output]
//     };

//     let previous_output_as_tx_out = TxOut {
//         value: 50_0000_0000,
//         script_pubkey: output_details.script_pubkey()
//     };
//     let prevouts = vec![previous_output_as_tx_out];
//     let mut sighash_cache = bitcoin::util::sighash::SighashCache::new(&transaction);
//     let signature_hash = sighash_cache.taproot_key_spend_signature_hash(0, &Prevouts::All(&prevouts), SchnorrSighashType::Default).unwrap();
//     let message = Message::from_slice(&signature_hash.to_vec()).unwrap();
//     println!("sighash: {}", message.to_hex());

//     let tweaked_private_key = private_key.tap_tweak(&secp, tap_tree.merkle_root()).into_inner();
//     let signature = tweaked_private_key.sign_schnorr(message);
//     let signature_vec = signature.as_ref().to_vec();

//     transaction.input[0].witness = Witness::from_vec(vec![signature_vec]);

//     let transaction_hex = transaction.serialize().to_hex();
//     println!("transaction hex: {}", transaction_hex);
// }