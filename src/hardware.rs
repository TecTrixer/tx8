use crate::Tx8Error;

const MB_16: usize = 16_777_216;
const MB_4: usize = 4_194_304;

#[derive(Clone, Copy, Debug)]
pub struct Cpu {
    pub a: u32,
    pub b: u32,
    pub c: u32,
    pub d: u32,
    pub r: u32,
    pub o: u32,
    pub s: u32,
    pub p: u32,
}

impl Cpu {
    pub fn new() -> Self {
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
pub struct Memory {
    array: Vec<u8>,
}

impl Memory {
    pub fn load_rom(data: &[u8]) -> Self {
        let mut array = vec![0; MB_16];
        array[MB_4..MB_4 + data.len()].copy_from_slice(data);
        Memory { array }
    }
    pub fn read_byte(&self, ptr: u32) -> u8 {
        let ptr = truncate_ptr(ptr);
        self.read(ptr)
    }

    pub fn read(&self, ptr: usize) -> u8 {
        // If the pointer is out of bounds return 0, otherwise the byte
        match self.array.get(ptr) {
            Some(byte) => *byte,
            None => 0,
        }
    }

    pub fn read_short(&self, ptr: u32) -> u16 {
        let ptr = truncate_ptr(ptr);

        let bytes = [self.read(ptr), self.read(ptr + 1)];
        u16::from_le_bytes(bytes)
    }
    pub fn read_24bit(&self, ptr: u32) -> u32 {
        let ptr = truncate_ptr(ptr);
        let bytes = [self.read(ptr), self.read(ptr + 1), self.read(ptr + 2), 0];
        u32::from_le_bytes(bytes)
    }
    pub fn read_int(&self, ptr: u32) -> u32 {
        let ptr = truncate_ptr(ptr);

        let bytes = [
            self.read(ptr),
            self.read(ptr + 1),
            self.read(ptr + 2),
            self.read(ptr + 3),
        ];
        u32::from_le_bytes(bytes)
    }

    pub fn write(&mut self, ptr: usize, val: u8) -> Result<(), Tx8Error> {
        if ptr >= self.array.len() {
            Err(Tx8Error::OutOfBoundsWrite)
        } else {
            self.array[ptr] = val;
            Ok(())
        }
    }

    pub fn write_byte(&mut self, ptr: u32, val: u8) -> Result<(), Tx8Error> {
        let ptr = truncate_ptr(ptr);
        self.write(ptr, val)
    }
    pub fn write_short(&mut self, ptr: u32, val: u16) -> Result<(), Tx8Error> {
        let ptr = truncate_ptr(ptr);
        let [first, second] = val.to_le_bytes();
        self.write(ptr, first)?;
        self.write(ptr + 1, second)
    }
    pub fn write_int(&mut self, ptr: u32, val: u32) -> Result<(), Tx8Error> {
        let ptr = truncate_ptr(ptr);
        let [first, second, third, fourth] = val.to_le_bytes();
        self.write(ptr, first)?;
        self.write(ptr + 1, second)?;
        self.write(ptr + 2, third)?;
        self.write(ptr + 3, fourth)
    }
}

fn truncate_ptr(ptr: u32) -> usize {
    // take only the last 24 bit
    0xffffff & ptr as usize
}
