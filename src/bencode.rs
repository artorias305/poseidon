use std::collections::BTreeMap;

#[derive(Debug, PartialEq)]
pub enum DecodeError {
    MissingColon,
    InvalidValue,
    ParseLength,
    ParseIntError,
    MissingTerminatingE,
}

impl std::fmt::Display for DecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let msg = match self {
            DecodeError::MissingColon => "bencoded value missing semicolon",
            DecodeError::InvalidValue => "invalid bencoded value",
            DecodeError::ParseLength => "failed to parse bencoded value length",
            DecodeError::ParseIntError => "failed to parse int",
            DecodeError::MissingTerminatingE => "couldn't find terminating e",
        };
        write!(f, "{}", msg)
    }
}

impl std::error::Error for DecodeError {}

#[derive(Debug, PartialEq)]
pub enum BencodeValue {
    String(String),
    Int(i64),
    List(Vec<BencodeValue>),
    Map(BTreeMap<String, BencodeValue>),
}

fn decode_inner(input: &str) -> Result<(BencodeValue, usize), DecodeError> {
    if input.is_empty() {
        return Err(DecodeError::InvalidValue);
    }

    match input.as_bytes()[0] {
        b'0'..=b'9' => {
            let colon = input.find(':').ok_or(DecodeError::MissingColon)?;

            let len = input[..colon]
                .parse::<usize>()
                .map_err(|_| DecodeError::ParseLength)?;

            let start = colon + 1;
            let end = start + len;

            if end > input.len() {
                return Err(DecodeError::InvalidValue);
            }

            Ok((BencodeValue::String(input[start..end].to_string()), end))
        }

        b'i' => {
            let e = input[1..]
                .find('e')
                .ok_or(DecodeError::MissingTerminatingE)?;

            let value = input[1..1 + e]
                .parse::<i64>()
                .map_err(|_| DecodeError::ParseIntError)?;

            Ok((BencodeValue::Int(value), e + 2))
        }

        b'l' => {
            let mut list = Vec::new();
            let mut pos = 1;

            loop {
                if pos >= input.len() {
                    return Err(DecodeError::MissingTerminatingE);
                }

                if input.as_bytes()[pos] == b'e' {
                    break;
                }

                let (value, consumed) = decode_inner(&input[pos..])?;
                list.push(value);
                pos += consumed;
            }

            Ok((BencodeValue::List(list), pos + 1))
        }

        b'd' => {
            let mut map = BTreeMap::new();
            let mut pos = 1;

            loop {
                if pos >= input.len() {
                    return Err(DecodeError::MissingTerminatingE);
                }

                if input.as_bytes()[pos] == b'e' {
                    break;
                }

                let (key, consumed) = decode_inner(&input[pos..])?;
                let key = match key {
                    BencodeValue::String(s) => s,
                    _ => return Err(DecodeError::InvalidValue),
                };
                pos += consumed;

                let (value, consumed) = decode_inner(&input[pos..])?;
                pos += consumed;

                map.insert(key, value);
            }

            Ok((BencodeValue::Map(map), pos + 1))
        }

        _ => Err(DecodeError::InvalidValue),
    }
}

pub fn decode(input: &str) -> Result<BencodeValue, DecodeError> {
    let (value, consumed) = decode_inner(input)?;

    if consumed != input.len() {
        return Err(DecodeError::InvalidValue);
    }

    Ok(value)
}
