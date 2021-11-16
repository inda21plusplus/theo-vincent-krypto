use rand::Rng;

fn generate_random_nonce() -> [u8; 12] {
    let mut rng = rand::thread_rng();
    rng.gen::<[u8; 12]>()
}

/** password in plain text, returns decrypted message */
pub fn decrypt_bytes(bytes: Vec<u8>, password: String, nonce_bytes: [u8; 12])  -> Result<Vec<u8>, String>{
    use aes_gcm_siv::aead::{Aead, NewAead};
    use aes_gcm_siv::{Aes256GcmSiv, Key, Nonce}; // Or `Aes128GcmSiv`

    let key = Key::from_slice(password.as_bytes());
    let cipher = Aes256GcmSiv::new(key);

    let nonce = &Nonce::from(nonce_bytes);

    Ok(cipher
        .decrypt(nonce, bytes.as_ref())
        .map_err(|_| String::from("decryption failure!"))?) // NOTE: handle this error to avoid panics!
}

/** password in plain text, returns nonce_bytes and encrypted message */
pub fn encrypt_bytes(bytes: Vec<u8>, password: String) -> Result<([u8; 12], Vec<u8>), String> {
    use aes_gcm_siv::aead::{Aead, NewAead};
    use aes_gcm_siv::{Aes256GcmSiv, Key, Nonce}; // Or `Aes128GcmSiv`

    let key = Key::from_slice(password.as_bytes());
    let cipher = Aes256GcmSiv::new(key);

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
