use std::io::Write;

use rand::Rng;
use ring::error::Unspecified;

fn generate_random_nonce() -> [u8; 12] {
    let mut rng = rand::thread_rng();
    rng.gen::<[u8; 12]>()
}

// TODO KEY NOT PASSWORD?

fn generate_key(password: String)-> Result<aes_gcm_siv::Key<aes_gcm_siv::aead::consts::U32>, String> {
    let mut pass_padded: Vec<u8> = vec![0; 32];
    pass_padded.as_mut_slice().write(password.as_bytes()).map_err(|e| format!("write error {}", e))?;

    Ok(*aes_gcm_siv::Key::from_slice(&pass_padded))
}

/** password in plain text, returns decrypted message */
pub fn decrypt_bytes(
    bytes: Vec<u8>,
    password: String,
    nonce_bytes: [u8; 12],
) -> Result<Vec<u8>, String> {
    use aes_gcm_siv::aead::{Aead, NewAead};
    use aes_gcm_siv::{Aes256GcmSiv, Nonce}; // Or `Aes128GcmSiv`

    let key = generate_key(password)?;
    let cipher = Aes256GcmSiv::new(&key);

    let nonce = &Nonce::from(nonce_bytes);

    Ok(cipher
        .decrypt(nonce, bytes.as_ref())
        .map_err(|_| String::from("decryption failure!"))?) // NOTE: handle this error to avoid panics!
}

/** password in plain text, returns nonce_bytes and encrypted message */
pub fn encrypt_bytes(bytes: Vec<u8>, password: String) -> Result<([u8; 12], Vec<u8>), String> {
    use aes_gcm_siv::aead::{Aead, NewAead};
    use aes_gcm_siv::{Aes256GcmSiv, Nonce}; // Or `Aes128GcmSiv`

    let key = generate_key(password)?;
    let cipher = Aes256GcmSiv::new(&key);

    let nonce_bytes = generate_random_nonce();

    let nonce = &Nonce::from(nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, bytes.as_ref())
        .map_err(|_| String::from("encryption failure!"))?; // NOTE: handle this error to avoid panics!

    Ok((nonce_bytes, ciphertext))
}

pub fn hash_password(salt: String, password: String) -> String {
    use argon2::Config;

    let password = password.as_bytes();
    let salt = salt.as_bytes();
    let config = Config::default();
    let hash = argon2::hash_encoded(password, salt, &config).unwrap();
    let matches = argon2::verify_encoded(&hash, password).unwrap();
    assert!(matches);

    hash
}

pub fn get_key_pair(path: &std::path::Path) -> Result<ring::signature::RsaKeyPair, CryptoError> {
    let key_data = read_file(path)?;
    Ok(ring::signature::RsaKeyPair::from_pkcs8(&key_data)
        .map_err(|_| CryptoError::BadPrivateKey)?)
}

pub fn sign_file(
    file_data: &Vec<u8>,
    file_name: &[u8],
    key_pair: &ring::signature::RsaKeyPair,
) -> Result<Vec<u8>, CryptoError> {
    let mut full_file = file_data.to_vec();
    full_file.append(&mut file_name.to_vec());

    let rng = ring::rand::SystemRandom::new();
    let mut signature = vec![0; key_pair.public_modulus_len()];
    key_pair
        .sign(
            &ring::signature::RSA_PKCS1_SHA256,
            &rng,
            &full_file,
            &mut signature,
        )
        .map_err(|_| CryptoError::OOM)?;

    Ok(signature)
}

#[derive(Debug)]
pub enum CryptoError {
    IO(std::io::Error),
    BadPrivateKey,
    OOM,
    BadSignature,
}

fn read_file(path: &std::path::Path) -> Result<Vec<u8>, CryptoError> {
    use std::io::Read;

    let mut file = std::fs::File::open(path).map_err(|e| CryptoError::IO(e))?;
    let mut contents: Vec<u8> = Vec::new();
    file.read_to_end(&mut contents)
        .map_err(|e| CryptoError::IO(e))?;
    Ok(contents)
}
