pub fn nice_id(length: usize) -> String {
    const URLSAFE: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
    (0..length)
        .map(|_| {
            let idx = rand::random_range(0..URLSAFE.len());
            URLSAFE[idx] as char
        })
        .collect()
}
