extern crate sha1;

static WS_GUID: &'static str = "258EAFA5-E914-47DA-95CA-C5AB0DC85B11";
static BASE64: &'static [u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
//const MAX_HEADERS: usize = 124;

pub fn hash_key(key: &[u8]) -> String {
    let mut hasher = sha1::Sha1::new();

    hasher.update(key);
    hasher.update(WS_GUID.as_bytes());

    encode_base64(&hasher.digest().bytes())
}

// This code is based on rustc_serialize base64 STANDARD
fn encode_base64(data: &[u8]) -> String {
    let len = data.len();
    let mod_len = len % 3;

    let mut encoded = vec![b'='; (len + 2) / 3 * 4];
    {
        let mut in_iter = data[..len - mod_len].iter().map(|&c| c as u32);
        let mut out_iter = encoded.iter_mut();

        let enc = |val| BASE64[val as usize];
        let mut write = |val| *out_iter.next().unwrap() = val;

        while let (Some(one), Some(two), Some(three)) =
            (in_iter.next(), in_iter.next(), in_iter.next()) {
            let g24 = one << 16 | two << 8 | three;
            write(enc((g24 >> 18) & 63));
            write(enc((g24 >> 12) & 63));
            write(enc((g24 >> 6) & 63));
            write(enc(g24 & 63));
        }

        match mod_len {
            1 => {
                let pad = (data[len - 1] as u32) << 16;
                write(enc((pad >> 18) & 63));
                write(enc((pad >> 12) & 63));
            }
            2 => {
                let pad = (data[len - 2] as u32) << 16 | (data[len - 1] as u32) << 8;
                write(enc((pad >> 18) & 63));
                write(enc((pad >> 12) & 63));
                write(enc((pad >> 6) & 63));
            }
            _ => (),
        }
    }

    String::from_utf8(encoded).unwrap()
}
