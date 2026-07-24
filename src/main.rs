use clap::{Parser, Subcommand};

#[derive(Subcommand)]
enum Commands {
    Decode { encoded_value: String },
}

#[derive(Parser)]
#[command(version, about = "CLI for torrent written in rust")]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, PartialEq)]
enum DecodeBencodeError {
    MissingColon,
    InvalidValue,
    ParseLength,
    ParseIntError,
    MissingTerminatingE,
}

impl std::fmt::Display for DecodeBencodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let msg = match self {
            DecodeBencodeError::MissingColon => "bencoded value missing semicolon",
            DecodeBencodeError::InvalidValue => "invalid bencoded value",
            DecodeBencodeError::ParseLength => "failed to parse bencoded value length",
            DecodeBencodeError::ParseIntError => "failed to parse int",
            DecodeBencodeError::MissingTerminatingE => "couldn't find terminating e",
        };
        write!(f, "{}", msg)
    }
}

impl std::error::Error for DecodeBencodeError {}

#[derive(Debug, PartialEq)]
enum BencodeValue {
    String(String),
    Int(i64),
    List(Vec<BencodeValue>),
}

impl BencodeValue {
    fn encoded_length(&self) -> usize {
        match self {
            BencodeValue::String(s) => s.len() + s.chars().count().to_string().len() + 1,
            BencodeValue::Int(i) => i.to_string().len() + 2,
            BencodeValue::List(list) => {
                let inner: usize = list.iter().map(|v| v.encoded_length()).sum();
                inner + 2
            }
        }
    }
}

fn decode_bencode(bencoded_value: &str) -> Result<BencodeValue, DecodeBencodeError> {
    if bencoded_value.is_empty() {
        return Err(DecodeBencodeError::InvalidValue);
    }

    let first_char = bencoded_value.chars().next().unwrap();

    if first_char.is_ascii_digit() {
        let colon_index = bencoded_value
            .find(':')
            .ok_or(DecodeBencodeError::MissingColon)?;
        let length = bencoded_value[..colon_index]
            .parse::<usize>()
            .map_err(|_| DecodeBencodeError::ParseLength)?;
        let start = colon_index + 1;

        if start + length > bencoded_value.len() {
            return Err(DecodeBencodeError::InvalidValue);
        }

        Ok(BencodeValue::String(
            bencoded_value[start..start + length].to_string(),
        ))
    } else if first_char == 'i' && bencoded_value.ends_with('e') {
        let e_index = bencoded_value[1..]
            .find('e')
            .ok_or(DecodeBencodeError::MissingTerminatingE)?;
        let value = bencoded_value[1..1 + e_index]
            .parse::<i64>()
            .map_err(|_| DecodeBencodeError::ParseIntError)?;
        Ok(BencodeValue::Int(value))
    } else if first_char == 'l' && bencoded_value.ends_with('e') {
        let mut list = Vec::new();
        let mut pos = 1;
        let end = bencoded_value.len() - 1;

        while pos < end {
            let element = decode_bencode(&bencoded_value[pos..])?;
            pos += element.encoded_length();
            list.push(element);
        }

        Ok(BencodeValue::List(list))
    } else {
        Err(DecodeBencodeError::InvalidValue)
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    match args.command {
        Commands::Decode { encoded_value } => {
            let decoded_value = decode_bencode(&encoded_value)?;
            dbg!(decoded_value);
        }
    }

    Ok(())
}
