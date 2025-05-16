use anyhow::Result;
use rand::RngCore;

fn main() -> Result<()> {
    let mut key = [0u8; 64];
    rand::rng().fill_bytes(&mut key);
    println!("HMAC key:\n{:?}", key);

    let b64u = base64_url::encode(&key);
    println!("Base64 key:\n{:?}", b64u);

    Ok(())
}
