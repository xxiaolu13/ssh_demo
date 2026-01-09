
use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    Aes256Gcm,Key
};
use anyhow::{Context, Result};
const MASTER_KEY:&[u8; 32] = b"MYSSHwAKR3!EPEM*YeID*TL9t*35Ei!O";
pub fn passwd_encryption(plain_text: String)->Result<String>{

    let key = Key::<Aes256Gcm>::from_slice(MASTER_KEY);

    let cipher = Aes256Gcm::new(&key);
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
    let ciphertext = cipher
        .encrypt(&nonce, plain_text.as_bytes())
        .map_err(|e| anyhow::anyhow!("Encryption failed: {}", e))?;

    Ok(format!("{}.{}", hex::encode(nonce), hex::encode(ciphertext)))
}


pub fn passwd_decrypt(stored_str:String)->Result<String>{

    let key = Key::<Aes256Gcm>::from_slice(MASTER_KEY);
    let cipher = Aes256Gcm::new(&key);
    let parts: Vec<&str> = stored_str.split('.').collect();
    if parts.len() != 2 {
        return Err(anyhow::anyhow!("Invalid stored password format"));
    }

    let nonce_bytes = hex::decode(parts[0]).context("Failed to decode nonce")?;
    let ciphertext = hex::decode(parts[1]).context("Failed to decode ciphertext")?;
    let plain_bytes = cipher
        .decrypt(nonce_bytes.as_slice().into(), ciphertext.as_slice())
        .map_err(|e| anyhow::anyhow!("Decryption failed: {}", e))?;
    Ok(String::from_utf8(plain_bytes)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_passwd_encryption() {
        let passwd = passwd_encryption("password".to_string()).unwrap();
        assert_eq!(passwd_decrypt(passwd).unwrap(), "password".to_string());
    }
}