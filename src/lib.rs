mod errors;
pub use errors::Tx8Error;

mod instruction;

mod hardware;
use hardware::{Cpu, Memory};

mod execution;
use execution::{Effect, Execution};

pub fn run_code(data: Vec<u8>) -> Result<(), Tx8Error> {
    let data = parse_rom(&data)?;
    let mut execution = Execution::new_with_rom(data);
    println!("Program output:");
    loop {
        if let Effect::Halted = execution.next_step()? {
            println!("\nProgram halted");
            break;
        }
    }
    Ok(())
}

fn parse_rom(data: &Vec<u8>) -> Result<&[u8], Tx8Error> {
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
    if data.len() != data_end {
        return Err(Tx8Error::ParseError);
    }

    let program_name = std::str::from_utf8(&data[64..program_name_end])?;
    let description = std::str::from_utf8(&data[program_name_end..description_end])?;
    println!("Executing program \"{}\"", program_name);
    println!("Description: {}", description);
    Ok(&data[description_end..data_end])
}
