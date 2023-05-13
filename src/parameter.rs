use crate::{
    hardware::{Cpu, Memory},
    Size::*,
    Tx8Error,
};

#[derive(Clone, Copy, Debug)]
pub enum ParameterMode {
    Unused,
    Constant8,
    Constant16,
    Constant32,
    AbsoluteAddress,
    RelativeAddress,
    Register,
    RegisterAddress,
}
pub fn parse_par_mode(byte: u8) -> Result<ParameterMode, Tx8Error> {
    match byte {
        0x0 => Ok(ParameterMode::Unused),
        0x1 => Ok(ParameterMode::Constant8),
        0x2 => Ok(ParameterMode::Constant16),
        0x3 => Ok(ParameterMode::Constant32),
        0x4 => Ok(ParameterMode::AbsoluteAddress),
        0x5 => Ok(ParameterMode::RelativeAddress),
        0x6 => Ok(ParameterMode::Register),
        0x7 => Ok(ParameterMode::RegisterAddress),
        _ => Err(Tx8Error::InstructionError),
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Parameter {
    Unused,
    Constant8(u8),
    Constant16(u16),
    Constant32(u32),
    AbsoluteAddress(u32),
    RelativeAddress(u32),
    Register(u8),
    RegisterAddress(u8),
}

pub fn parse_parameter(mem: &Memory, ptr: u32, par_mode: ParameterMode) -> (Parameter, u32) {
    match par_mode {
        ParameterMode::Unused => (Parameter::Unused, 0),
        ParameterMode::Constant8 => (Parameter::Constant8(mem.read_byte(ptr)), 1),
        ParameterMode::Constant16 => (Parameter::Constant16(mem.read_short(ptr)), 2),
        ParameterMode::Constant32 => (Parameter::Constant32(mem.read_int(ptr)), 4),
        ParameterMode::AbsoluteAddress => (Parameter::AbsoluteAddress(mem.read_24bit(ptr)), 3),
        ParameterMode::RelativeAddress => (Parameter::RelativeAddress(mem.read_24bit(ptr)), 3),
        ParameterMode::Register => (Parameter::Register(mem.read_byte(ptr)), 1),
        ParameterMode::RegisterAddress => (Parameter::RegisterAddress(mem.read_byte(ptr)), 1),
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Value {
    pub val: u32,
    pub size: Size,
}

impl Value {
    pub fn new(val: u32, size: Size) -> Self {
        Value { val, size }
    }
    pub fn from_par(
        par: Parameter,
        cpu: &Cpu,
        mem: &Memory,
        mem_size: Size,
    ) -> Result<Self, Tx8Error> {
        match par {
            Parameter::Unused => Err(Tx8Error::InstructionError),
            Parameter::Constant8(x) => Ok(Value::new(x as u32, Byte)),
            Parameter::Constant16(x) => Ok(Value::new(x as u32, Short)),
            Parameter::Constant32(x) => Ok(Value::new(x as u32, Int)),
            Parameter::AbsoluteAddress(ptr) => match mem_size {
                Byte => Ok(Value::new(mem.read_byte(ptr) as u32, Byte)),
                Short => Ok(Value::new(mem.read_short(ptr) as u32, Short)),
                Int => Ok(Value::new(mem.read_int(ptr), Int)),
            },
            Parameter::RelativeAddress(ptr) => match mem_size {
                Byte => Ok(Value::new(mem.read_byte(ptr + cpu.o) as u32, Byte)),
                Short => Ok(Value::new(mem.read_short(ptr + cpu.o) as u32, Short)),
                Int => Ok(Value::new(mem.read_int(ptr + cpu.o), Int)),
            },
            Parameter::Register(r) => {
                let val = match 0xf & r {
                    0x00 => cpu.a,
                    0x01 => cpu.b,
                    0x02 => cpu.c,
                    0x03 => cpu.d,
                    0x04 => cpu.r,
                    0x05 => cpu.o,
                    0x06 => cpu.p,
                    0x07 => cpu.s,
                    _ => return Err(Tx8Error::InvalidRegister),
                };
                let size = get_reg_size(r);
                let filter = match size {
                    Byte => 0xff,
                    Short => 0xffff,
                    Int => 0xffffffff,
                };
                Ok(Value::new(val & filter, size))
            }
            Parameter::RegisterAddress(r) => {
                let val = match 0xf & r {
                    0x00 => cpu.a,
                    0x01 => cpu.b,
                    0x02 => cpu.c,
                    0x03 => cpu.d,
                    0x04 => cpu.r,
                    0x05 => cpu.o,
                    0x06 => cpu.p,
                    0x07 => cpu.s,
                    _ => return Err(Tx8Error::InvalidRegister),
                };
                let filter = match get_reg_size(r) {
                    Byte => 0xff,
                    Short => 0xffff,
                    Int => 0xffffffff,
                };
                match mem_size {
                    Byte => Ok(Value::new(mem.read_byte(val & filter) as u32, Byte)),
                    Short => Ok(Value::new(mem.read_short(val & filter) as u32, Short)),
                    Int => Ok(Value::new(mem.read_int(val & filter), Int)),
                }
            }
        }
    }
    pub fn from_par_signed(
        par: Parameter,
        cpu: &Cpu,
        mem: &Memory,
        mem_size: Size,
    ) -> Result<Self, Tx8Error> {
        match par {
            Parameter::Unused => Err(Tx8Error::InstructionError),
            Parameter::Constant8(x) => Ok(Value::new(x as i8 as i32 as u32, Byte)),
            Parameter::Constant16(x) => Ok(Value::new(x as i16 as i32 as u32, Short)),
            Parameter::Constant32(x) => Ok(Value::new(x as u32, Int)),
            Parameter::AbsoluteAddress(ptr) => match mem_size {
                Byte => Ok(Value::new(mem.read_byte(ptr) as i8 as i32 as u32, Byte)),
                Short => Ok(Value::new(mem.read_short(ptr) as i16 as i32 as u32, Short)),
                Int => Ok(Value::new(mem.read_int(ptr), Int)),
            },
            Parameter::RelativeAddress(ptr) => match mem_size {
                Byte => Ok(Value::new(
                    mem.read_byte(ptr + cpu.o) as i8 as i32 as u32,
                    Byte,
                )),
                Short => Ok(Value::new(
                    mem.read_short(ptr + cpu.o) as i16 as i32 as u32,
                    Short,
                )),
                Int => Ok(Value::new(mem.read_int(ptr + cpu.o), Int)),
            },
            Parameter::Register(r) => {
                let val = match 0xf & r {
                    0x00 => cpu.a,
                    0x01 => cpu.b,
                    0x02 => cpu.c,
                    0x03 => cpu.d,
                    0x04 => cpu.r,
                    0x05 => cpu.o,
                    0x06 => cpu.p,
                    0x07 => cpu.s,
                    _ => return Err(Tx8Error::InvalidRegister),
                };
                let size = get_reg_size(r);
                let val = match size {
                    Byte => (val & 0xff) as i8 as i32 as u32,
                    Short => (val & 0xffff) as i16 as i32 as u32,
                    Int => val,
                };
                Ok(Value::new(val, size))
            }
            Parameter::RegisterAddress(r) => {
                let val = match 0xf & r {
                    0x00 => cpu.a,
                    0x01 => cpu.b,
                    0x02 => cpu.c,
                    0x03 => cpu.d,
                    0x04 => cpu.r,
                    0x05 => cpu.o,
                    0x06 => cpu.p,
                    0x07 => cpu.s,
                    _ => return Err(Tx8Error::InvalidRegister),
                };
                let filter = match get_reg_size(r) {
                    Byte => 0xff,
                    Short => 0xffff,
                    Int => 0xffffffff,
                };
                match mem_size {
                    Byte => Ok(Value::new(
                        mem.read_byte(val & filter) as i8 as i32 as u32,
                        Byte,
                    )),
                    Short => Ok(Value::new(
                        mem.read_short(val & filter) as i16 as i32 as u32,
                        Short,
                    )),
                    Int => Ok(Value::new(mem.read_int(val & filter), Int)),
                }
            }
        }
    }
}

fn get_reg_size(byte: u8) -> Size {
    // TODO: maybe get more efficient, was lazy, therefore counted all possibilities
    match byte {
        0x00 | 0x01 | 0x02 | 0x03 | 0x04 | 0x05 | 0x06 | 0x07 => Int,
        0x20 | 0x21 | 0x22 | 0x23 | 0x24 | 0x25 | 0x26 | 0x27 => Short,
        0x10 | 0x11 | 0x12 | 0x13 | 0x14 | 0x15 | 0x16 | 0x17 => Byte,
        _ => Int,
    }
}

#[derive(Copy, Clone, Debug)]
pub enum Size {
    Byte,
    Short,
    Int,
}

impl Size {
    pub fn bytes(&self) -> u32 {
        match self {
            Byte => 1,
            Short => 2,
            Int => 4,
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum Writable {
    AbsoluteAddress(AbsoluteAddress),
    RelativeAddress(RelativeAddress),
    Register(Register),
    RegisterAddress(RegisterAddress),
}
impl Writable {
    pub fn from_par(par: Parameter) -> Result<Writable, Tx8Error> {
        match par {
            Parameter::Unused => Err(Tx8Error::InstructionError),
            Parameter::Constant8(_) => Err(Tx8Error::InstructionError),
            Parameter::Constant16(_) => Err(Tx8Error::InstructionError),
            Parameter::Constant32(_) => Err(Tx8Error::InstructionError),
            Parameter::AbsoluteAddress(x) => Ok(Writable::AbsoluteAddress(AbsoluteAddress(x))),
            Parameter::RelativeAddress(x) => Ok(Writable::RelativeAddress(RelativeAddress(x))),
            Parameter::Register(x) => Ok(Writable::Register(Register(x))),
            Parameter::RegisterAddress(x) => Ok(Writable::RegisterAddress(RegisterAddress(x))),
        }
    }
}

impl Write for Writable {
    fn write(self, mem: &mut Memory, cpu: &mut Cpu, val: u32) -> Result<(), Tx8Error> {
        match self {
            Writable::AbsoluteAddress(x) => x.write(mem, cpu, val),
            Writable::RelativeAddress(x) => x.write(mem, cpu, val),
            Writable::Register(x) => x.write(mem, cpu, val),
            Writable::RegisterAddress(x) => x.write(mem, cpu, val),
        }
    }
    fn size(&self) -> Size {
        match self {
            Writable::AbsoluteAddress(x) => x.size(),
            Writable::RelativeAddress(x) => x.size(),
            Writable::Register(x) => x.size(),
            Writable::RegisterAddress(x) => x.size(),
        }
    }

    fn write_size(
        self,
        mem: &mut Memory,
        cpu: &mut Cpu,
        val: u32,
        size: Size,
    ) -> Result<(), Tx8Error> {
        match self {
            Writable::AbsoluteAddress(x) => x.write_size(mem, cpu, val, size),
            Writable::RelativeAddress(x) => x.write_size(mem, cpu, val, size),
            Writable::Register(x) => x.write_size(mem, cpu, val, size),
            Writable::RegisterAddress(x) => x.write_size(mem, cpu, val, size),
        }
    }
}

pub trait Write {
    fn write(self, mem: &mut Memory, cpu: &mut Cpu, val: u32) -> Result<(), Tx8Error>;
    fn write_size(
        self,
        mem: &mut Memory,
        cpu: &mut Cpu,
        val: u32,
        size: Size,
    ) -> Result<(), Tx8Error>;
    fn size(&self) -> Size;
}
#[derive(Copy, Clone, Debug)]
pub struct AbsoluteAddress(u32);

impl Write for AbsoluteAddress {
    fn write(self, mem: &mut Memory, cpu: &mut Cpu, val: u32) -> Result<(), Tx8Error> {
        self.write_size(mem, cpu, val, Byte)
    }
    fn size(&self) -> Size {
        Int
    }

    fn write_size(
        self,
        mem: &mut Memory,
        _: &mut Cpu,
        val: u32,
        size: Size,
    ) -> Result<(), Tx8Error> {
        match size {
            Byte => mem.write_byte(self.0, val as u8),
            Short => mem.write_short(self.0, val as u16),
            Int => mem.write_int(self.0, val),
        };
        Ok(())
    }
}

#[derive(Copy, Clone, Debug)]
pub struct RelativeAddress(u32);

impl Write for RelativeAddress {
    fn write(self, mem: &mut Memory, cpu: &mut Cpu, val: u32) -> Result<(), Tx8Error> {
        self.write_size(mem, cpu, val, Byte)
    }
    fn size(&self) -> Size {
        Int
    }

    fn write_size(
        self,
        mem: &mut Memory,
        cpu: &mut Cpu,
        val: u32,
        size: Size,
    ) -> Result<(), Tx8Error> {
        let ptr = self.0 + cpu.o;
        match size {
            Byte => mem.write_byte(ptr, val as u8),
            Short => mem.write_short(ptr, val as u16),
            Int => mem.write_int(ptr, val),
        };
        Ok(())
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Register(pub u8);

impl Write for Register {
    fn write(self, mem: &mut Memory, cpu: &mut Cpu, val: u32) -> Result<(), Tx8Error> {
        let size = match self.0 & 0xf0 {
            0x00 => Int,
            0x20 => Short,
            0x10 => Byte,
            _ => return Err(Tx8Error::InvalidRegister),
        };
        self.write_size(mem, cpu, val, size)
    }

    fn size(&self) -> Size {
        get_reg_size(self.0)
    }

    fn write_size(
        self,
        _: &mut Memory,
        cpu: &mut Cpu,
        val: u32,
        size: Size,
    ) -> Result<(), Tx8Error> {
        let (mask, mask2) = match size {
            Byte => (0xffffff00, 0xff),
            Short => (0xffff0000, 0xffff),
            Int => (0x0, 0xffffffff),
        };
        match self.0 & 0xf {
            0x00 => cpu.a = (cpu.a & mask) | (val & mask2),
            0x01 => cpu.b = (cpu.b & mask) | (val & mask2),
            0x02 => cpu.c = (cpu.c & mask) | (val & mask2),
            0x03 => cpu.d = (cpu.d & mask) | (val & mask2),
            0x04 => cpu.r = (cpu.r & mask) | (val & mask2),
            0x05 => cpu.o = (cpu.o & mask) | (val & mask2),
            0x06 => cpu.p = (cpu.p & mask) | (val & mask2),
            0x07 => cpu.s = (cpu.s & mask) | (val & mask2),
            _ => return Err(Tx8Error::InvalidRegister),
        };

        Ok(())
    }
}

#[derive(Copy, Clone, Debug)]
pub struct RegisterAddress(u8);

impl Write for RegisterAddress {
    fn write(self, mem: &mut Memory, cpu: &mut Cpu, val: u32) -> Result<(), Tx8Error> {
        self.write_size(mem, cpu, val, Int)
    }
    fn size(&self) -> Size {
        Int
    }

    fn write_size(
        self,
        mem: &mut Memory,
        cpu: &mut Cpu,
        val: u32,
        size: Size,
    ) -> Result<(), Tx8Error> {
        let ptr = match self.0 {
            0x00 => cpu.a,
            0x01 => cpu.b,
            0x02 => cpu.c,
            0x03 => cpu.d,
            0x04 => cpu.r,
            0x05 => cpu.o,
            0x06 => cpu.p,
            0x07 => cpu.s,
            0x10 => cpu.a & 0xff,
            0x11 => cpu.b & 0xff,
            0x12 => cpu.c & 0xff,
            0x13 => cpu.d & 0xff,
            0x14 => cpu.r & 0xff,
            0x15 => cpu.o & 0xff,
            0x16 => cpu.p & 0xff,
            0x17 => cpu.s & 0xff,
            0x20 => cpu.a & 0xffff,
            0x21 => cpu.b & 0xffff,
            0x22 => cpu.c & 0xffff,
            0x23 => cpu.d & 0xffff,
            0x24 => cpu.r & 0xffff,
            0x25 => cpu.o & 0xffff,
            0x26 => cpu.p & 0xffff,
            0x27 => cpu.s & 0xffff,
            _ => return Err(Tx8Error::InvalidRegister),
        };
        match size {
            Byte => mem.write_byte(ptr, val as u8),
            Short => mem.write_short(ptr, val as u16),
            Int => mem.write_int(ptr, val),
        }
        Ok(())
    }
}
