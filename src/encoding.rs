use std::str;

fn encode_ascii85_chunk(chunk: &[u8; 4]) -> crate::Result<String> {
    let mut encoded = String::with_capacity(4);

    let mut pattern: u32 = ((((((0b0 | u32::from(chunk[0])) << 8) | u32::from(chunk[1])) << 8)
        | u32::from(chunk[2]))
        << 8)
        | u32::from(chunk[3]);

    for i in 1..=5 {
        let ascii_ch: u32 = (pattern % 85) + 33;
        pattern /= 85;

        if i <= 4 && chunk[4 - i] == b'\0' {
            continue;
        }

        if ascii_ch < 33 || 117 < ascii_ch {
            return Err(crate::Error::new(
                crate::ErrorKind::Syntax,
                "received unprintable character",
            ));
        }

        encoded.push(ascii_ch.try_into().unwrap());
    }

    Ok(encoded.chars().rev().collect())
}

pub fn encode_ascii85(raw: &str) -> crate::Result<String> {
    // Encoded string will be roughly 5/4 times larger than the raw string
    let mut encoded = String::with_capacity((raw.len() * 5) / 4);

    let mut chunk = [b'\0', b'\0', b'\0', b'\0'];
    for (i, ch) in raw.bytes().enumerate() {
        let chunk_index = i % 4;
        chunk[chunk_index] = ch;

        // Collected a complete chunk
        if chunk_index == 3 || i == raw.len() - 1 {
            encoded.push_str(&encode_ascii85_chunk(&chunk)?);
            chunk[0] = b'\0';
            chunk[1] = b'\0';
            chunk[2] = b'\0';
            chunk[3] = b'\0';
        }
    }

    Ok(encoded)
}

pub fn decode_ascii85_chunk(chunk: &[u8; 5]) -> crate::Result<String> {
    let mut decoded = String::with_capacity(4);

    const POWERS: [u32; 5] = [85 * 85 * 85 * 85, 85 * 85 * 85, 85 * 85, 85, 1];
    let mut pattern: u32 = 0b0;
    for (i, ch) in chunk.iter().enumerate() {
        pattern += u32::from((*ch).max(33) - 33) * POWERS[i];
    }

    const CLEAR: u32 = 0b11111111;
    for shift in (0..25).step_by(8).rev() {
        let ch = (pattern >> shift) & CLEAR;
        decoded.push(ch.try_into().expect("something broke"));
    }

    Ok(decoded)
}

pub fn decode_ascii85(encoded: &str) -> crate::Result<String> {
    // Decoded string will be roughly 4/5ths the size of the encoded string
    let mut decoded = String::with_capacity((encoded.len() * 4) / 5);

    let mut chunk = [b'u', b'u', b'u', b'u', b'u'];
    for (i, ch) in encoded.bytes().enumerate() {
        let chunk_index = i % 5;
        chunk[chunk_index] = ch;

        // Collected a complete chunk
        if chunk_index == 4 {
            decoded.push_str(&decode_ascii85_chunk(&chunk)?);
            chunk[0] = b'u';
            chunk[1] = b'u';
            chunk[2] = b'u';
            chunk[3] = b'u';
            chunk[4] = b'u';
        } else if i == encoded.len() - 1 {
            let mut partial = decode_ascii85_chunk(&chunk)?;
            let pad_amount = 5 - (encoded.len() % 5);
            if pad_amount != 5 {
                let _ = partial.split_off(partial.len() - (1 + pad_amount));
            }
            decoded.push_str(&partial);
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
}
