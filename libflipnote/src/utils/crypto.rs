use sha1_checked::{Digest, Sha1};

pub fn hash_data(data: &[u8]) -> Vec<u8> {
    let mut hasher = Sha1::new();
    hasher.update(data);
    hasher.finalize().to_vec()
}
