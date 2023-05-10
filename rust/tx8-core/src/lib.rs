use std::{array::TryFromSliceError, error::Error, fmt::Display, str::Utf8Error};

pub fn run_code(data: Vec<u8>) {
    for byte in data {
        print!("{}", byte as char);
    }
}

pub fn parse_rom(data: &Vec<u8>) -> Result<&[u8], Tx8Error> {
    // Ensure file is at least 64 bytes long and magic bytes match
    if data.len() < 64 || &data[0..4] != "TX8\0".as_bytes() {
        return Err(Tx8Error::ParseError);
    }
    // assign length
    let program_name_length = data[4] as usize;
    let description_length = u16::from_le_bytes(data[5..7].try_into()?) as usize;
    let data_length = u32::from_le_bytes(data[7..11].try_into()?) as usize;

    let program_name_end = 64 + program_name_length;
    let description_end = program_name_end + description_length;
    let data_end = description_end + data_length;

    // check length
    if data.len() != data_end - 1 {
        return Err(Tx8Error::ParseError);
    }

    let program_name = std::str::from_utf8(&data[64..program_name_end])?;
    let description = std::str::from_utf8(&data[program_name_end..description_end])?;
    println!("Executing program \"{}\"", program_name);
    println!("Description: \"{}\"", description);
    Ok(&data[description_end..data_end])
}

#[derive(Clone, Copy, Debug)]
pub enum Tx8Error {
    ParseError,
}

impl Error for Tx8Error {}

impl Display for Tx8Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ParseError")
    }
}

impl From<TryFromSliceError> for Tx8Error {
    fn from(_value: TryFromSliceError) -> Self {
        Tx8Error::ParseError
    }
}
impl From<Utf8Error> for Tx8Error {
    fn from(_value: Utf8Error) -> Self {
        Tx8Error::ParseError
    }
}
