use crate::parameter::*;
use crate::Size::*;
use crate::{Cpu, Memory, Tx8Error};

pub fn parse_instruction(
    cpu: &Cpu,
    mem: &Memory,
    ptr: u32,
) -> Result<(Instruction, u32), Tx8Error> {
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
    Pop(Writable),
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
            OpCode::Jump => Instruction::Jump(
                Value::from_par(first_par, cpu, mem, Byte)?,
                Comparison::None,
            ),
            OpCode::JumpEqual => Instruction::Jump(
                Value::from_par(first_par, cpu, mem, Byte)?,
                Comparison::Equal,
            ),
            OpCode::JumpNotEqual => Instruction::Jump(
                Value::from_par(first_par, cpu, mem, Byte)?,
                Comparison::NotEqual,
            ),
            OpCode::JumpGreaterThan => Instruction::Jump(
                Value::from_par(first_par, cpu, mem, Byte)?,
                Comparison::Greater,
            ),
            OpCode::JumpGreaterEqual => Instruction::Jump(
                Value::from_par(first_par, cpu, mem, Byte)?,
                Comparison::GreaterEqual,
            ),
            OpCode::JumpLessThan => Instruction::Jump(
                Value::from_par(first_par, cpu, mem, Byte)?,
                Comparison::Less,
            ),
            OpCode::JumpLessEqual => Instruction::Jump(
                Value::from_par(first_par, cpu, mem, Byte)?,
                Comparison::LessEqual,
            ),
            OpCode::CompareSigned => Instruction::CompareSigned(
                Value::from_par(first_par, cpu, mem, Byte)?,
                Value::from_par(sec_par, cpu, mem, Byte)?,
            ),
            OpCode::CompareFloat => Instruction::CompareFloat(
                Value::from_par(first_par, cpu, mem, Byte)?,
                Value::from_par(sec_par, cpu, mem, Byte)?,
            ),
            OpCode::CompareUnsigned => Instruction::CompareUnsigned(
                Value::from_par(first_par, cpu, mem, Byte)?,
                Value::from_par(sec_par, cpu, mem, Byte)?,
            ),
            OpCode::Call => Instruction::Call(Value::from_par(first_par, cpu, mem, Byte)?),
            OpCode::SysCall => Instruction::SysCall(Value::from_par(first_par, cpu, mem, Byte)?),
            OpCode::Halt => unreachable!(),
            OpCode::Nop => unreachable!(),
            OpCode::Return => unreachable!(),
            OpCode::Load => Instruction::Load(
                Writable::from_par(first_par)?,
                Value::from_par(sec_par, cpu, mem, Byte)?,
            ),
            OpCode::Push => Instruction::Push(Value::from_par(first_par, cpu, mem, Byte)?),
            OpCode::LoadSigned => Instruction::Load(
                Writable::from_par(first_par)?,
                Value::from_par_signed(sec_par, cpu, mem, Byte)?,
            ),
            OpCode::LoadA => Instruction::Load(
                Writable::Register(Register(0x00)),
                Value::from_par(first_par, cpu, mem, Byte)?,
            ),
            OpCode::StoreA => {
                Instruction::Load(Writable::from_par(first_par)?, Value::new(cpu.a, Int))
            }
            OpCode::LoadB => Instruction::Load(
                Writable::Register(Register(0x01)),
                Value::from_par(first_par, cpu, mem, Byte)?,
            ),
            OpCode::StoreB => {
                Instruction::Load(Writable::from_par(first_par)?, Value::new(cpu.b, Int))
            }
            OpCode::LoadC => Instruction::Load(
                Writable::Register(Register(0x02)),
                Value::from_par(first_par, cpu, mem, Byte)?,
            ),
            OpCode::StoreC => {
                Instruction::Load(Writable::from_par(first_par)?, Value::new(cpu.c, Int))
            }
            OpCode::LoadD => Instruction::Load(
                Writable::Register(Register(0x03)),
                Value::from_par(first_par, cpu, mem, Byte)?,
            ),
            OpCode::StoreD => {
                Instruction::Load(Writable::from_par(first_par)?, Value::new(cpu.d, Int))
            }
            OpCode::Zero => Instruction::Load(Writable::from_par(first_par)?, Value::new(0, Byte)),
            OpCode::Pop => Instruction::Pop(Writable::from_par(first_par)?),
            OpCode::LoadWord => Instruction::Load(
                Writable::from_par(first_par)?,
                Value::from_par(sec_par, cpu, mem, Int)?,
            ),
            OpCode::LoadWordSigned => Instruction::Load(
                Writable::from_par(first_par)?,
                Value::from_par_signed(sec_par, cpu, mem, Int)?,
            ),
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
            Instruction::Pop(_) => false,
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
        0x11 => OpCode::LoadSigned,
        0x12 => OpCode::LoadWord,
        0x13 => OpCode::LoadWordSigned,
        0x14 => OpCode::LoadA,
        0x15 => OpCode::StoreA,
        0x16 => OpCode::LoadB,
        0x17 => OpCode::StoreB,
        0x18 => OpCode::LoadC,
        0x19 => OpCode::StoreC,
        0x1a => OpCode::LoadD,
        0x1b => OpCode::StoreD,
        0x1c => OpCode::Zero,
        0x1d => OpCode::Push,
        0x1e => OpCode::Pop,
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
    LoadSigned,
    LoadWord,
    LoadWordSigned,
    LoadA,
    StoreA,
    LoadB,
    StoreB,
    LoadC,
    StoreC,
    LoadD,
    StoreD,
    Zero,
    Push,
    Pop,
}
