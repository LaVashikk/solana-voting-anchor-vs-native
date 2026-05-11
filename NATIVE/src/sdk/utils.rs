pub fn string_to_bytes<const N: usize>(s: &str) -> Option<[u8; N]> {
    let mut bytes_arr = [0; N];
    let s_bytes = s.as_bytes();

    if s_bytes.len() > N {
        return None;
    }

    let len = s_bytes.len();
    bytes_arr[..len].copy_from_slice(&s_bytes[..len]);
    Some(bytes_arr)
}


pub fn bytes_to_string<const N: usize>(bytes: &[u8; N]) -> String {
    let len = bytes.iter().position(|&b| b == 0).unwrap_or(N);

    String::from_utf8_lossy(&bytes[..len])
        .trim_matches('\0')
        .to_string()
}
