mod errors;
pub use errors::Tx8Error;

mod instruction;
pub use instruction::parse_instruction;

pub fn run_code(data: Vec<u8>) {
    let _ = parse_rom(&data);
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
    if data.len() != data_end {
        return Err(Tx8Error::ParseError);
    }

    let program_name = std::str::from_utf8(&data[64..program_name_end])?;
    let description = std::str::from_utf8(&data[program_name_end..description_end])?;
    println!("Executing program \"{}\"", program_name);
    println!("Description: {}", description);
    Ok(&data[description_end..data_end])
}

const MB_16: usize = 16_777_216;
const MB_4: usize = 4_194_304;

#[derive(Clone, Copy, Debug)]
struct Cpu {
    a: u32,
    b: u32,
    c: u32,
    d: u32,
    r: u32,
    o: u32,
    s: u32,
    p: u32,
}

impl Cpu {
    fn new() -> Self {
        Cpu {
            a: 0,
            b: 0,
            c: 0,
            d: 0,
            r: 0,
            o: 0,
            s: 0xc02000,
            p: MB_4 as u32,
        }
    }
}

#[derive(Clone, Debug)]
struct Memory {
    array: Vec<u8>,
}

impl Memory {
    fn load_rom(data: &[u8]) -> Self {
        let mut array = Vec::with_capacity(MB_16);
        array[MB_4..MB_4 + data.len()].copy_from_slice(data);
        Memory { array }
    }
    fn read_byte(&self, ptr: u32) -> u8 {
        // take only the last 24bit of the pointer
        let ptr = (0xffffff & ptr) as usize;
        self.read(ptr)
    }

    fn read(&self, ptr: usize) -> u8 {
        // If the pointer is out of bounds return 0, otherwise the byte
        match self.array.get(ptr) {
            Some(byte) => *byte,
            None => 0,
        }
    }

    fn read_short(&self, ptr: u32) -> u16 {
        // take only the last 24bit of the pointer
        let ptr = (0xffffff & ptr) as usize;

        let bytes = [self.read(ptr), self.read(ptr + 1)];
        u16::from_le_bytes(bytes)
    }
    fn read_int(&self, ptr: u32) -> u32 {
        // take only the last 24bit of the pointer
        let ptr = (0xffffff & ptr) as usize;

        let bytes = [
            self.read(ptr),
            self.read(ptr + 1),
            self.read(ptr + 2),
            self.read(ptr + 3),
        ];
        u32::from_le_bytes(bytes)
    }
}

#[derive(Clone, Debug)]
struct Execution {
    cpu: Cpu,
    memory: Memory,
}

impl Execution {
    fn new_with_rom(data: &[u8]) -> Self {
        Execution {
            cpu: Cpu::new(),
            memory: Memory::load_rom(data),
        }
    }
}

