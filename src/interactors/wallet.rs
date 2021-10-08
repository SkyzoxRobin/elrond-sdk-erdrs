extern crate rand;

use bip39::{Language, Mnemonic};
use hmac::{Hmac, Mac, NewMac};
use pbkdf2::pbkdf2;
use sha2::Sha512;
use zeroize::Zeroize;

use crate::crypto::private_key::PrivateKey;

const EGLD_COIN_TYPE: u32 = 508;
const HARDENED: u32 = 0x80000000;

type HmacSha521 = Hmac<Sha512>;

pub struct Wallet {}

impl Wallet {
    pub fn new() -> Self {
        Self {}
    }

    // GenerateMnemonic will generate a new mnemonic value using the bip39 implementation
    pub fn generate_mnemonic() -> Mnemonic {
        let mut rng = rand::thread_rng();
        Mnemonic::generate_in_with(&mut rng, Language::English, 24).unwrap()
    }

    fn seed_from_mnemonic(mnemonic: Mnemonic, password: &str) -> [u8; 64] {
        let mut salt = String::with_capacity(8 + password.len());
        salt.push_str("mnemonic");
        salt.push_str(password);

        let mut seed = [0u8; 64];

        pbkdf2::<Hmac<Sha512>>(
            mnemonic.to_string().as_bytes(),
            salt.as_bytes(),
            2048,
            &mut seed,
        );

        salt.zeroize();

        seed
    }

    pub fn get_private_key_from_mnemonic(
        mnemonic: Mnemonic,
        account: u32,
        address_index: u32,
    ) -> PrivateKey {
        let seed = Self::seed_from_mnemonic(mnemonic, "");

        let serialized_key_len = 32;
        let hardened_child_padding: u8 = 0;

        let mut digest =
            HmacSha521::new_from_slice(b"ed25519 seed").expect("HMAC can take key of any size");
        digest.update(&seed);
        let intermediary: Vec<u8> = digest
            .finalize()
            .into_bytes()
            .into_iter()
            .map(|x| x)
            .collect();
        let mut key = intermediary[..serialized_key_len].to_vec();
        let mut chain_code = intermediary[serialized_key_len..].to_vec();

        for child_idx in [
            44 | HARDENED,
            EGLD_COIN_TYPE | HARDENED,
            account | HARDENED, // account
            HARDENED,
            address_index | HARDENED, // addressIndex
        ] {
            let mut buff = [vec![hardened_child_padding], key.clone()].concat();
            buff.push((child_idx >> 24) as u8);
            buff.push((child_idx >> 16) as u8);
            buff.push((child_idx >> 8) as u8);
            buff.push(child_idx as u8);

            digest =
                HmacSha521::new_from_slice(&chain_code).expect("HMAC can take key of any size");
            digest.update(&buff);
            let intermediary: Vec<u8> = digest
                .finalize()
                .into_bytes()
                .into_iter()
                .map(|x| x)
                .collect();
            key = intermediary[..serialized_key_len].to_vec();
            chain_code = intermediary[serialized_key_len..].to_vec();
        }

        PrivateKey::from_bytes(key.as_slice()).unwrap()
    }

    pub fn get_address_from_private_key() {}
}