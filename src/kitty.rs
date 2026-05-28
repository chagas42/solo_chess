use std::io::Write;

const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
const CHUNK_SIZE: usize = 4096;

pub fn base64_encode(data: &[u8]) -> String {
    let mut out = String::with_capacity((data.len() + 2) / 3 * 4);
    let mut i = 0;
    while i + 3 <= data.len() {
        let n = ((data[i] as u32) << 16) | ((data[i + 1] as u32) << 8) | (data[i + 2] as u32);
        out.push(ALPHABET[((n >> 18) & 0x3F) as usize] as char);
        out.push(ALPHABET[((n >> 12) & 0x3F) as usize] as char);
        out.push(ALPHABET[((n >> 6) & 0x3F) as usize] as char);
        out.push(ALPHABET[(n & 0x3F) as usize] as char);
        i += 3;
    }
    let rem = data.len() - i;
    if rem == 1 {
        let n = (data[i] as u32) << 16;
        out.push(ALPHABET[((n >> 18) & 0x3F) as usize] as char);
        out.push(ALPHABET[((n >> 12) & 0x3F) as usize] as char);
        out.push('=');
        out.push('=');
    } else if rem == 2 {
        let n = ((data[i] as u32) << 16) | ((data[i + 1] as u32) << 8);
        out.push(ALPHABET[((n >> 18) & 0x3F) as usize] as char);
        out.push(ALPHABET[((n >> 12) & 0x3F) as usize] as char);
        out.push(ALPHABET[((n >> 6) & 0x3F) as usize] as char);
        out.push('=');
    }
    out
}

pub fn transmit_image(buf: &mut Vec<u8>, id: u32, png_data: &[u8]) {
    let b64 = base64_encode(png_data);
    let bytes = b64.as_bytes();
    let total = bytes.len().div_ceil(CHUNK_SIZE);

    for i in 0..total {
        let start = i * CHUNK_SIZE;
        let end = (start + CHUNK_SIZE).min(bytes.len());
        let chunk = &bytes[start..end];
        let more = if i < total - 1 { 1 } else { 0 };

        if i == 0 {
            write!(buf, "\x1b_Gf=100,a=t,t=d,i={},q=2,m={};", id, more).unwrap();
        } else {
            write!(buf, "\x1b_Gm={};", more).unwrap();
        }
        buf.extend_from_slice(chunk);
        write!(buf, "\x1b\\").unwrap();
    }
}

pub fn place_image(buf: &mut Vec<u8>, id: u32, cols: u32, rows: u32) {
    write!(buf, "\x1b_Ga=p,i={},c={},r={},q=2\x1b\\", id, cols, rows).unwrap();
}

pub fn clear_placements(buf: &mut Vec<u8>) {
    write!(buf, "\x1b_Ga=d,d=a,q=2\x1b\\").unwrap();
}
