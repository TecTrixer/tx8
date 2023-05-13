use crate::{Cpu, Memory, Tx8Error};

pub fn parse_instruction(
    cpu: &Cpu,
    mem: &Memory,
    ptr: u32,
) -> Result<(Instruction, u32), Tx8Error> {
    // instruction pointer is too large
    if ptr > 0xfffff0 {
        return Err(Tx8Error::InstructionError);
    }
    let mut len = 0;

    // Read OpCode
    let op_code = parse_op_code(mem.read_byte(ptr))?;
    len += 1;

    // if no parameters are passed, then the instruction is fully parsed
    match op_code {
        OpCode::Halt | OpCode::Nop | OpCode::Return => {
            return Ok((Instruction::no_params(op_code), 0))
        }
        _ => (),
    };

    // Read parameter mode
    let parameter_mode_byte = mem.read_byte(ptr + len);
    let first_parameter = parse_par_mode(parameter_mode_byte >> 4)?;
    let second_parameter = parse_par_mode(parameter_mode_byte & 0x0f)?;
    len += 1;

    let (first_parameter, par_len) = parse_parameter(mem, ptr + len, first_parameter);
    len += par_len;
    let (second_parameter, par_len) = parse_parameter(mem, ptr + len, second_parameter);
    len += par_len;
    Ok((
        Instruction::with_params(op_code, first_parameter, second_parameter, cpu, mem)?,
        len,
    ))
}

#[derive(Clone, Copy, Debug)]
enum ParameterMode {
    Unused,
    Constant8,
    Constant16,
    Constant32,
    AbsoluteAddress,
    RelativeAddress,
    Register,
    RegisterAddress,
}
fn parse_par_mode(byte: u8) -> Result<ParameterMode, Tx8Error> {
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

fn parse_parameter(mem: &Memory, ptr: u32, par_mode: ParameterMode) -> (Parameter, u32) {
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
    fn new(val: u32, size: Size) -> Self {
        Value { val, size }
    }
    fn from_par(par: Parameter, cpu: &Cpu, mem: &Memory) -> Result<Self, Tx8Error> {
        match par {
            Parameter::Unused => Err(Tx8Error::InstructionError),
            Parameter::Constant8(x) => Ok(Value::new(x as u32, Size::Byte)),
            Parameter::Constant16(x) => Ok(Value::new(x as u32, Size::Short)),
            Parameter::Constant32(x) => Ok(Value::new(x as u32, Size::Int)),
            Parameter::AbsoluteAddress(ptr) => Ok(Value::new(mem.read_int(ptr), Size::Int)),
            Parameter::RelativeAddress(ptr) => Ok(Value::new(mem.read_int(ptr + cpu.o), Size::Int)),
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
                    Size::Byte => 0xff,
                    Size::Short => 0xffff,
                    Size::Int => 0xffffffff,
                };
                Ok(Value::new(mem.read_int(val & filter), Size::Int))
            }
        }
    }
}

fn get_reg_size(byte: u8) -> Size {
    // TODO: maybe get more efficient, was lazy, therefore counted all possibilities
    match byte {
        0x00 | 0x01 | 0x02 | 0x03 | 0x04 | 0x05 | 0x06 | 0x07 => Size::Int,
        0x20 | 0x21 | 0x22 | 0x23 | 0x24 | 0x25 | 0x26 | 0x27 => Size::Short,
        0x10 | 0x11 | 0x12 | 0x13 | 0x14 | 0x15 | 0x16 | 0x17 => Size::Byte,
        _ => Size::Int,
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
            Size::Byte => 1,
            Size::Short => 2,
            Size::Int => 4,
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
    fn from_par(par: Parameter) -> Result<Writable, Tx8Error> {
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
}

pub trait Write {
    fn write(self, mem: &mut Memory, cpu: &mut Cpu, val: u32) -> Result<(), Tx8Error>;
    fn size(&self) -> Size;
}
#[derive(Copy, Clone, Debug)]
pub struct AbsoluteAddress(u32);

impl Write for AbsoluteAddress {
    fn write(self, mem: &mut Memory, _: &mut Cpu, val: u32) -> Result<(), Tx8Error> {
        mem.write_int(self.0, val)
    }
    fn size(&self) -> Size {
        Size::Int
    }
}

#[derive(Copy, Clone, Debug)]
pub struct RelativeAddress(u32);

impl Write for RelativeAddress {
    fn write(self, mem: &mut Memory, cpu: &mut Cpu, val: u32) -> Result<(), Tx8Error> {
        let ptr = self.0 + cpu.o;
        mem.write_int(ptr, val)
    }
    fn size(&self) -> Size {
        Size::Int
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Register(u8);

// TODO: maybe extract mapping into its own function later
impl Write for Register {
    fn write(self, _: &mut Memory, cpu: &mut Cpu, val: u32) -> Result<(), Tx8Error> {
        match self.0 {
            0x00 => cpu.a = val,
            0x01 => cpu.b = val,
            0x02 => cpu.c = val,
            0x03 => cpu.d = val,
            0x04 => cpu.r = val,
            0x05 => cpu.o = val,
            0x06 => cpu.p = val,
            0x07 => cpu.s = val,
            0x10 => cpu.a = (cpu.a & 0xffffff00) | (0xff & val),
            0x11 => cpu.b = (cpu.b & 0xffffff00) | (0xff & val),
            0x12 => cpu.c = (cpu.c & 0xffffff00) | (0xff & val),
            0x13 => cpu.d = (cpu.d & 0xffffff00) | (0xff & val),
            0x14 => cpu.r = (cpu.r & 0xffffff00) | (0xff & val),
            0x15 => cpu.o = (cpu.o & 0xffffff00) | (0xff & val),
            0x16 => cpu.p = (cpu.p & 0xffffff00) | (0xff & val),
            0x17 => cpu.s = (cpu.s & 0xffffff00) | (0xff & val),
            0x20 => cpu.a = (cpu.a & 0xffff0000) | (0xffff & val),
            0x21 => cpu.b = (cpu.b & 0xffff0000) | (0xffff & val),
            0x22 => cpu.c = (cpu.c & 0xffff0000) | (0xffff & val),
            0x23 => cpu.d = (cpu.d & 0xffff0000) | (0xffff & val),
            0x24 => cpu.r = (cpu.r & 0xffff0000) | (0xffff & val),
            0x25 => cpu.o = (cpu.o & 0xffff0000) | (0xffff & val),
            0x26 => cpu.p = (cpu.p & 0xffff0000) | (0xffff & val),
            0x27 => cpu.s = (cpu.s & 0xffff0000) | (0xffff & val),
            _ => return Err(Tx8Error::InvalidRegister),
        }
        Ok(())
    }

    fn size(&self) -> Size {
        get_reg_size(self.0)
    }
}

#[derive(Copy, Clone, Debug)]
pub struct RegisterAddress(u8);

impl Write for RegisterAddress {
    fn write(self, mem: &mut Memory, cpu: &mut Cpu, val: u32) -> Result<(), Tx8Error> {
        match self.0 {
            0x00 => mem.write_int(cpu.a, val),
            0x01 => mem.write_int(cpu.b, val),
            0x02 => mem.write_int(cpu.c, val),
            0x03 => mem.write_int(cpu.d, val),
            0x04 => mem.write_int(cpu.r, val),
            0x05 => mem.write_int(cpu.o, val),
            0x06 => mem.write_int(cpu.p, val),
            0x07 => mem.write_int(cpu.s, val),
            0x10 => mem.write_int(cpu.a & 0xff, val),
            0x11 => mem.write_int(cpu.b & 0xff, val),
            0x12 => mem.write_int(cpu.c & 0xff, val),
            0x13 => mem.write_int(cpu.d & 0xff, val),
            0x14 => mem.write_int(cpu.r & 0xff, val),
            0x15 => mem.write_int(cpu.o & 0xff, val),
            0x16 => mem.write_int(cpu.p & 0xff, val),
            0x17 => mem.write_int(cpu.s & 0xff, val),
            0x20 => mem.write_int(cpu.a & 0xffff, val),
            0x21 => mem.write_int(cpu.b & 0xffff, val),
            0x22 => mem.write_int(cpu.c & 0xffff, val),
            0x23 => mem.write_int(cpu.d & 0xffff, val),
            0x24 => mem.write_int(cpu.r & 0xffff, val),
            0x25 => mem.write_int(cpu.o & 0xffff, val),
            0x26 => mem.write_int(cpu.p & 0xffff, val),
            0x27 => mem.write_int(cpu.s & 0xffff, val),
            _ => Err(Tx8Error::InvalidRegister),
        }
    }
    fn size(&self) -> Size {
        Size::Int
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Comparison {
    None,
    Equal,
    NotEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
}

#[derive(Clone, Copy, Debug)]
pub enum Instruction {
    Halt,
    Nop,
    Jump(Value, Comparison),
    CompareSigned(Value, Value),
    CompareFloat(Value, Value),
    CompareUnsigned(Value, Value),
    Call(Value),
    SysCall(Value),
    Return,
    Load(Writable, Value),
    Push(Value),
}

impl Instruction {
    fn no_params(op: OpCode) -> Self {
        match op {
            OpCode::Halt => Instruction::Halt,
            OpCode::Nop => Instruction::Nop,
            OpCode::Return => Instruction::Return,
            _ => unreachable!("No operation could be found for the no parameter OpCode"),
        }
    }
    fn with_params(
        op_code: OpCode,
        first_par: Parameter,
        sec_par: Parameter,
        cpu: &Cpu,
        mem: &Memory,
    ) -> Result<Self, Tx8Error> {
        Ok(match op_code {
            OpCode::Jump => {
                Instruction::Jump(Value::from_par(first_par, cpu, mem)?, Comparison::None)
            }
            OpCode::JumpEqual => {
                Instruction::Jump(Value::from_par(first_par, cpu, mem)?, Comparison::Equal)
            }
            OpCode::JumpNotEqual => {
                Instruction::Jump(Value::from_par(first_par, cpu, mem)?, Comparison::NotEqual)
            }
            OpCode::JumpGreaterThan => {
                Instruction::Jump(Value::from_par(first_par, cpu, mem)?, Comparison::Greater)
            }
            OpCode::JumpGreaterEqual => Instruction::Jump(
                Value::from_par(first_par, cpu, mem)?,
                Comparison::GreaterEqual,
            ),
            OpCode::JumpLessThan => {
                Instruction::Jump(Value::from_par(first_par, cpu, mem)?, Comparison::Less)
            }
            OpCode::JumpLessEqual => {
                Instruction::Jump(Value::from_par(first_par, cpu, mem)?, Comparison::LessEqual)
            }
            OpCode::CompareSigned => Instruction::CompareSigned(
                Value::from_par(first_par, cpu, mem)?,
                Value::from_par(sec_par, cpu, mem)?,
            ),
            OpCode::CompareFloat => Instruction::CompareFloat(
                Value::from_par(first_par, cpu, mem)?,
                Value::from_par(sec_par, cpu, mem)?,
            ),
            OpCode::CompareUnsigned => Instruction::CompareUnsigned(
                Value::from_par(first_par, cpu, mem)?,
                Value::from_par(sec_par, cpu, mem)?,
            ),
            OpCode::Call => Instruction::Call(Value::from_par(first_par, cpu, mem)?),
            OpCode::SysCall => Instruction::SysCall(Value::from_par(first_par, cpu, mem)?),
            OpCode::Halt => unreachable!(),
            OpCode::Nop => unreachable!(),
            OpCode::Return => unreachable!(),
            OpCode::Load => Instruction::Load(
                Writable::from_par(first_par)?,
                Value::from_par(sec_par, cpu, mem)?,
            ),
            OpCode::Push => Instruction::Push(Value::from_par(first_par, cpu, mem)?),
        })
    }

    pub fn increase_program_counter(&self) -> bool {
        match self {
            Instruction::Halt => false,
            Instruction::Nop => true,
            Instruction::Jump(_, _) => false,
            Instruction::CompareSigned(_, _) => true,
            Instruction::CompareFloat(_, _) => true,
            Instruction::CompareUnsigned(_, _) => true,
            Instruction::Call(_) => false,
            Instruction::SysCall(_) => true,
            Instruction::Return => false,
            Instruction::Load(_, _) => true,
            Instruction::Push(_) => true,
        }
    }
}
fn parse_op_code(byte: u8) -> Result<OpCode, Tx8Error> {
    Ok(match byte {
        0x00 => OpCode::Halt,
        0x01 => OpCode::Nop,
        0x02 => OpCode::Jump,
        0x03 => OpCode::JumpEqual,
        0x04 => OpCode::JumpNotEqual,
        0x05 => OpCode::JumpGreaterThan,
        0x06 => OpCode::JumpGreaterEqual,
        0x07 => OpCode::JumpLessThan,
        0x08 => OpCode::JumpLessEqual,
        0x09 => OpCode::CompareSigned,
        0x0a => OpCode::CompareFloat,
        0x0b => OpCode::CompareUnsigned,
        0x0c => OpCode::Call,
        0x0d => OpCode::Return,
        0x0e => OpCode::SysCall,
        0x10 => OpCode::Load,
        0x1d => OpCode::Push,
        _ => return Err(Tx8Error::InvalidOpCode),
    })
}

#[derive(Clone, Copy, Debug)]
enum OpCode {
    Halt,
    Nop,
    JumpGreaterThan,
    JumpNotEqual,
    JumpEqual,
    Jump,
    JumpGreaterEqual,
    JumpLessThan,
    JumpLessEqual,
    CompareSigned,
    CompareFloat,
    CompareUnsigned,
    Call,
    Return,
    SysCall,
    Load,
    Push,
}
