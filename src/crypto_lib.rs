#![allow(dead_code)]

use std::fmt;

use rand::{Rng, OsRng};
use crypto::curve25519::{curve25519_base, curve25519};
use crypto::chacha20poly1305::ChaCha20Poly1305;
use crypto::aead::{AeadEncryptor, AeadDecryptor};


pub type Key = [u8; 32];

pub enum EncryptError {
    RngInitializationFailed,
}

impl fmt::Debug for EncryptError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Failed to encrypt")
    }
}

pub enum DecryptError {
    Malformed,
    Invalid,
}

impl fmt::Debug for DecryptError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Failed to decrypt")
    }
}

pub fn gen_key_pair() -> (Key, Key) {
    let mut priv_key = [0u8; 32];
    OsRng::new().unwrap().fill_bytes(&mut priv_key[..]);
    (priv_key, curve25519_base(&priv_key[..]))
}

#[derive(Clone)]
pub struct Crypto {
    priv_key: Key,
    pub pub_key: Key,
}

impl Crypto {
    pub fn new(private_key: Key, public_key: Key) -> Crypto {
        Crypto {
            priv_key: private_key,
            pub_key: public_key,
        }
    }

    pub fn encrypt(&self, public_key: &[u8; 32], message: &[u8]) -> Result<Vec<u8>, EncryptError> {
        let mut rng = try!(OsRng::new().map_err(|_| EncryptError::RngInitializationFailed));

        let mut ephemeral_secret_key = [0u8; 32];
        rng.fill_bytes(&mut ephemeral_secret_key[..]);

        let ephemeral_public_key: [u8; 32] = curve25519_base(&ephemeral_secret_key[..]);
        let symmetric_key = curve25519(&ephemeral_secret_key[..], &public_key[..]);

        let mut c = ChaCha20Poly1305::new(&symmetric_key, &[0u8; 8][..], &[]);

        let mut output = vec![0; 32 + 16 + message.len()];
        let mut tag = [0u8; 16];
        c.encrypt(message, &mut output[32+16..], &mut tag[..]);

        for (dest, src) in (&mut output[0..32]).iter_mut().zip( ephemeral_public_key.iter() ) {
            *dest = *src;
        }

        for (dest, src) in (&mut output[32..48]).iter_mut().zip( tag.iter() ) {
            *dest = *src;
        }

        Ok(output)
    }

    pub fn decrypt(&self, message: &[u8]) -> Result<Vec<u8>, DecryptError> {
        if message.len() < 48 {
            return Err(DecryptError::Malformed);
        }

        let ephemeral_public_key = &message[0..32];
        let tag = &message[32..48];
        let ciphertext = &message[48..];

        let mut plaintext = vec![0; ciphertext.len()];
        let symmetric_key = curve25519(&self.priv_key, ephemeral_public_key);

        let mut decrypter = ChaCha20Poly1305::new(&symmetric_key[..], &[0u8; 8][..], &[]);
        if !decrypter.decrypt(ciphertext, &mut plaintext[..], tag) {
            return Err(DecryptError::Invalid);
        }

        Ok(plaintext)
    }

}



































