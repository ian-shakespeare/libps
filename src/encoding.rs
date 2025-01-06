use std::str;

use crate::{Error, ErrorKind};

fn fill_buffer(buf: &mut [u8], ch: u8) {
    for b in buf {
        *b = ch;
    }
}

#[allow(dead_code)]
pub fn encode_ascii85(raw: &str) -> crate::Result<String> {
    // Encoded string will be roughly 5/4 times larger than the raw string
    let mut encoded = String::with_capacity((raw.len() * 5) / 4);

    let mut chunk = [b'\0', b'\0', b'\0', b'\0'];
    for (i, ch) in raw.bytes().enumerate() {
        let chunk_index = i % 4;
        chunk[chunk_index] = ch;

        // Collected a complete chunk
        if chunk_index == 3 || i == raw.len() - 1 {
            let mut encoded_chunk = String::with_capacity(4);

            let mut pattern: u32 = (((((u32::from(chunk[0]) << 8) | u32::from(chunk[1])) << 8)
                | u32::from(chunk[2]))
                << 8)
                | u32::from(chunk[3]);

            for i in 1..=5 {
                let ascii_ch: u32 = (pattern % 85) + 33;
                pattern /= 85;

                if i <= 4 && chunk[4 - i] == b'\0' {
                    continue;
                }

                if !(33..=117).contains(&ascii_ch) {
                    return Err(Error::new(
                        ErrorKind::Syntax,
                        "received unprintable character",
                    ));
                }

                encoded_chunk.push(ascii_ch.try_into().unwrap());
            }

            encoded_chunk = encoded_chunk.chars().rev().collect();

            encoded.push_str(&encoded_chunk);
            fill_buffer(&mut chunk, b'\0');
        }
    }

    Ok(encoded)
}

pub fn decode_ascii85(encoded: &str) -> crate::Result<String> {
    // Decoded string will be roughly 4/5ths the size of the encoded string
    let mut decoded = String::with_capacity((encoded.len() * 4) / 5);

    let mut chunk = [b'u', b'u', b'u', b'u', b'u'];
    for (i, ch) in encoded.bytes().enumerate() {
        let chunk_index = i % 5;
        chunk[chunk_index] = ch;

        // Collected a complete chunk
        let is_last = i == encoded.len() - 1;
        if chunk_index == 4 || is_last {
            let mut decoded_chunk = String::with_capacity(4);

            const POWERS: [u32; 5] = [85 * 85 * 85 * 85, 85 * 85 * 85, 85 * 85, 85, 1];
            let pattern = chunk
                .iter()
                .zip(POWERS)
                .fold(0b0, |pattern, (ch, power)| -> u32 {
                    pattern + u32::from((*ch).max(33) - 33) * power
                });

            for shift in [24, 16, 8, 0] {
                match char::from_u32((pattern >> shift) & 0b11111111) {
                    None => return Err(Error::from(ErrorKind::Syntax)),
                    Some(ch) => decoded_chunk.push(ch),
                }
            }
            decoded.push_str(&decoded_chunk);
            fill_buffer(&mut chunk, b'u');
        }
    }

    // Cut off empty end
    let pad_amount = 5 - (encoded.len() % 5);
    if pad_amount != 5 {
        // TODO: Investigate if there is a better way (split_off doesn't work)
        for _ in 0..pad_amount {
            decoded.pop();
        }
    }

    Ok(decoded)
}

#[allow(dead_code)]
pub fn encode_hex(s: &str) -> crate::Result<String> {
    let mut encoded = String::with_capacity(s.len() * 2);

    for ch in s.bytes() {
        let ch = format!("{:X}", ch);
        encoded.push_str(&ch);
    }

    Ok(encoded)
}

pub fn decode_hex(encoded: &str) -> crate::Result<String> {
    let mut decoded = String::with_capacity(encoded.len() / 2);

    let mut chunk = [b'\0', b'\0'];
    for (i, ch) in encoded.to_uppercase().bytes().enumerate() {
        let chunk_index = i % 2;
        chunk[chunk_index] = ch;

        // Collected a complete chunk
        let is_last = i == encoded.len() - 1;
        if chunk_index == 1 || is_last {
            if is_last && chunk[1] == b'\0' {
                chunk[1] = b'0';
            }

            if let Ok(code) = str::from_utf8(&chunk) {
                match u8::from_str_radix(code, 16) {
                    Err(_) => return Err(Error::from(ErrorKind::Syntax)),
                    Ok(ch) => decoded.push(ch.into()),
                }
            }

            fill_buffer(&mut chunk, b'\0');
        }
    }

    Ok(decoded)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error;

    #[test]
    fn test_encode_ascii85() -> Result<(), Box<dyn error::Error>> {
        let cases = [
            ("", ""),
            (".", "/c"),
            ("sure", "F*2M7"),
            ("this is some text", "FD,B0+DGm>F)Po,+EV1>F8"),
            (
                "Man is distinguished, not only by his reason, but by this singular passion \
                from other animals, which is a lust of the mind, that by a perseverance of \
                delight in the continued and indefatigable generation of knowledge, exceeds \
                the short vehemence of any carnal pleasure.",
                "9jqo^BlbD-BleB1DJ+*+F(f,q/0JhKF<GL>Cj@.4Gp$d7F!,L7@<6@)/0JDEF<G%<+EV:2F!,O<\
                DJ+*.@<*K0@<6L(Df-\\0Ec5e;DffZ(EZee.Bl.9pF\"AGXBPCsi+DGm>@3BB/F*&OCAfu2/AKYi(\
                DIb:@FD,*)+C]U=@3BN#EcYf8ATD3s@q?d$AftVqCh[NqF<G:8+EV:.+Cf>-FD5W8ARlolDIal(\
                DId<j@<?3r@:F%a+D58'ATD4$Bl@l3De:,-DJs`8ARoFb/0JMK@qB4^F!,R<AKZ&-DfTqBG%G>u\
                D.RTpAKYo'+CT/5+Cei#DII?(E,9)oF*2M7/c",
            ),
        ];

        for (input, expect) in cases {
            let received = encode_ascii85(input)?;

            assert_eq!(expect, &received);
        }

        Ok(())
    }

    #[test]
    fn test_decode_ascii85() -> Result<(), Box<dyn error::Error>> {
        let cases = [
            ("", ""),
            ("/c", "."),
            ("F*2M7", "sure"),
            ("FD,B0+DGm>F)Po,+EV1>F8", "this is some text"),
            (
                "9jqo^BlbD-BleB1DJ+*+F(f,q/0JhKF<GL>Cj@.4Gp$d7F!,L7@<6@)/0JDEF<G%<+EV:2F!,O<\
                DJ+*.@<*K0@<6L(Df-\\0Ec5e;DffZ(EZee.Bl.9pF\"AGXBPCsi+DGm>@3BB/F*&OCAfu2/AKYi(\
                DIb:@FD,*)+C]U=@3BN#EcYf8ATD3s@q?d$AftVqCh[NqF<G:8+EV:.+Cf>-FD5W8ARlolDIal(\
                DId<j@<?3r@:F%a+D58'ATD4$Bl@l3De:,-DJs`8ARoFb/0JMK@qB4^F!,R<AKZ&-DfTqBG%G>u\
                D.RTpAKYo'+CT/5+Cei#DII?(E,9)oF*2M7/c",
                "Man is distinguished, not only by his reason, but by this singular passion \
                from other animals, which is a lust of the mind, that by a perseverance of \
                delight in the continued and indefatigable generation of knowledge, exceeds \
                the short vehemence of any carnal pleasure.",
            ),
        ];

        for (input, expect) in cases {
            let received = decode_ascii85(input)?;

            assert_eq!(expect, &received);
        }

        Ok(())
    }

    #[test]
    fn test_encode_hex() -> Result<(), Box<dyn error::Error>> {
        let cases = [
            ("", ""),
            (".", "2E"),
            ("this is some text", "7468697320697320736F6D652074657874"),
        ];

        for (input, expect) in cases {
            let received = encode_hex(input)?;

            assert_eq!(expect, &received);
        }

        Ok(())
    }

    #[test]
    fn test_decode_hex() -> Result<(), Box<dyn error::Error>> {
        let cases = [
            ("", ""),
            ("4", "@"),
            ("2E", "."),
            ("7468697320697320736F6D652074657874", "this is some text"),
        ];

        for (input, expect) in cases {
            let received = decode_hex(input)?;

            assert_eq!(expect, &received);
        }

        Ok(())
    }
}
