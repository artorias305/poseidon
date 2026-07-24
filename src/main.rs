#[derive(Debug)]
enum DecodeBencodeError {
    MissingColon,
    InvalidValue,
    ParseLength,
    ParseIntError,
}

impl std::fmt::Display for DecodeBencodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let msg = match self {
            DecodeBencodeError::MissingColon => "bencoded value missing semicolon",
            DecodeBencodeError::InvalidValue => "invalid bencoded value",
            DecodeBencodeError::ParseLength => "failed to parse bencoded value length",
            DecodeBencodeError::ParseIntError => "failed to parse int",
        };
        write!(f, "{}", msg)
    }
}

impl std::error::Error for DecodeBencodeError {}

fn decode_bencode(bencoded_value: &str) -> Result<String, DecodeBencodeError> {
    if bencoded_value.chars().nth(0).unwrap().is_ascii_digit() {
        let colon_index = bencoded_value
            .find(':')
            .ok_or(DecodeBencodeError::MissingColon)?;
        let length = bencoded_value[..colon_index]
            .parse::<usize>()
            .map_err(|_| DecodeBencodeError::ParseLength)?;
        let start = colon_index + 1;
        Ok(bencoded_value[start..start + length].to_string())
    } else if bencoded_value.chars().nth(0).unwrap() == 'i'
        && bencoded_value.chars().last().unwrap() == 'e'
    {
        let value = bencoded_value[1..bencoded_value.len() - 1]
            .parse::<i64>()
            .map_err(|_| DecodeBencodeError::ParseIntError)?;
        Ok(value.to_string())
    } else {
        Err(DecodeBencodeError::InvalidValue)
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = std::env::args().collect::<Vec<String>>();

    if args.len() < 3 {
        eprintln!("Usage: {} <command> <args>", args.get(0).unwrap());
        return Ok(());
    }

    let command = args.get(1).unwrap();

    match command.as_str() {
        "decode" => {
            let encoded_str = args.get(2).unwrap();
            let decoded_str = decode_bencode(encoded_str)?;
            dbg!(decoded_str);
        }
        _ => {
            println!("invalid command");
        }
    }

    Ok(())
}
