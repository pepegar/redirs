use rand::{thread_rng, RngCore};

pub fn gen_replica_id() -> String {
    let mut rng = thread_rng();
    let mut bytes = vec![0u8; 40 / 2];
    rng.fill_bytes(&mut bytes);
    let hex_string: String = bytes.iter().map(|byte| format!("{:02x}", byte)).collect();
    hex_string
}
