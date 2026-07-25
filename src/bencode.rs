use std::collections::BTreeMap;

#[derive(Debug, PartialEq, thiserror::Error)]
pub enum DecodeError {
    #[error("missing colon")]
    MissingColon,

    #[error("invalid value")]
    InvalidValue,

    #[error("could not parse length")]
    ParseLength,

    #[error("could not parse int")]
    ParseIntError,

    #[error("missing terminating 'e'")]
    MissingTerminatingE,

    #[error("invalid UTF-8")]
    InvalidUtf8,
}

#[derive(Debug, PartialEq)]
pub enum BencodeValue {
    String(Vec<u8>),
    Int(i64),
    List(Vec<BencodeValue>),
    Map(BTreeMap<String, BencodeValue>),
}

fn decode_inner(input: &[u8]) -> Result<(BencodeValue, usize), DecodeError> {
    let mut start = 0;
    while start < input.len() && matches!(input[start], b' ' | b'\n' | b'\r' | b'\t') {
        start += 1;
    }
    let input = &input[start..];

    if input.is_empty() {
        return Err(DecodeError::InvalidValue);
    }

    match input[0] {
        b'0'..=b'9' => {
            let colon = input
                .iter()
                .position(|&b| b == b':')
                .ok_or(DecodeError::MissingColon)?;

            let len = std::str::from_utf8(&input[..colon])
                .map_err(|_| DecodeError::InvalidUtf8)?
                .parse::<usize>()
                .map_err(|_| DecodeError::ParseLength)?;

            let content_start = colon + 1;
            let end = content_start + len;

            if end > input.len() {
                return Err(DecodeError::InvalidValue);
            }

            Ok((
                BencodeValue::String(input[content_start..end].to_vec()),
                end + start,
            ))
        }

        b'i' => {
            let e = input[1..]
                .iter()
                .position(|&b| b == b'e')
                .ok_or(DecodeError::MissingTerminatingE)?;

            let value = std::str::from_utf8(&input[1..1 + e])
                .map_err(|_| DecodeError::InvalidUtf8)?
                .parse::<i64>()
                .map_err(|_| DecodeError::ParseIntError)?;

            Ok((BencodeValue::Int(value), e + 2 + start))
        }

        b'l' => {
            let mut list = Vec::new();
            let mut pos = 1;

            loop {
                while pos < input.len() && matches!(input[pos], b' ' | b'\n' | b'\r' | b'\t') {
                    pos += 1
                }

                if pos >= input.len() {
                    return Err(DecodeError::MissingTerminatingE);
                }

                if input[pos] == b'e' {
                    break;
                }

                let (value, consumed) = decode_inner(&input[pos..])?;
                list.push(value);
                pos += consumed;
            }

            Ok((BencodeValue::List(list), pos + 1 + start))
        }

        b'd' => {
            let mut map = BTreeMap::new();
            let mut pos = 1;

            loop {
                while pos < input.len() && matches!(input[pos], b' ' | b'\n' | b'\r' | b'\t') {
                    pos += 1
                }

                if pos >= input.len() {
                    return Err(DecodeError::MissingTerminatingE);
                }

                if input[pos] == b'e' {
                    break;
                }

                let (key, consumed) = decode_inner(&input[pos..])?;
                let key = match key {
                    BencodeValue::String(s) => {
                        String::from_utf8(s).map_err(|_| DecodeError::InvalidUtf8)?
                    }
                    _ => return Err(DecodeError::InvalidValue),
                };
                pos += consumed;

                let (value, consumed) = decode_inner(&input[pos..])?;
                pos += consumed;

                map.insert(key, value);
            }

            Ok((BencodeValue::Map(map), pos + 1 + start))
        }

        _ => Err(DecodeError::InvalidValue),
    }
}

pub fn decode(input: &[u8]) -> Result<BencodeValue, DecodeError> {
    let (value, consumed) = decode_inner(input)?;

    let mut remaining = &input[consumed..];
    while !remaining.is_empty() && matches!(remaining[0], b' ' | b'\n' | b'\r' | b'\t') {
        remaining = &remaining[1..];
    }

    if !remaining.is_empty() {
        return Err(DecodeError::InvalidValue);
    }

    Ok(value)
}

pub fn encode(value: &BencodeValue) -> Vec<u8> {
    match value {
        BencodeValue::String(s) => {
            let mut out = format!("{}:", s.len()).into_bytes();
            out.extend_from_slice(s);
            out
        }
        BencodeValue::Int(n) => format!("i{}e", n).into_bytes(),
        BencodeValue::List(list) => {
            let mut out = vec![b'l'];
            for item in list {
                out.extend(encode(item));
            }
            out.push(b'e');
            out
        }
        BencodeValue::Map(map) => {
            let mut out = vec![b'd'];
            for (k, v) in map {
                out.extend(encode(&BencodeValue::String(k.clone().into())));
                out.extend(encode(v));
            }
            out.push(b'e');
            out
        }
    }
}
