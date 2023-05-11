pub fn parse_instruction(mem: &Memory, ptr: u32) -> Result<Instruction, Tx8Error> {
    // instruction pointer is too large
    if ptr > 0xfffff0 {
        return Err(Tx8Error::InstructionError);
    }
    let mut len = 0;

    // Read OpCode
    let op_code = parse_op_code(mem.read_byte(ptr));
    len += 1;

    // if no parameters are passed, then the instruction is fully parsed
    match op_code {
        OpCode::Halt | OpCode::Nop | OpCode::Return => return Ok(Instruction::no_params(op_code)),
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
    Ok(Instruction::with_params(
        op_code,
        first_parameter,
        second_parameter,
        len,
    ))
}

fn parse_op_code(byte: u8) -> OpCode {
    match byte {
        0x00 => OpCode::Halt,
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
        _ => OpCode::Nop,
    }
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
    RegisterAddress(u32),
}

fn parse_parameter(mem: &Memory, ptr: u32, par_mode: ParameterMode) -> (Parameter, u32) {
    match par_mode {
        ParameterMode::Unused => (Parameter::Unused, 0),
        ParameterMode::Constant8 => (Parameter::Constant8(mem.read_byte(ptr)), 1),
        ParameterMode::Constant16 => (Parameter::Constant16(mem.read_short(ptr)), 2),
        ParameterMode::Constant32 => (Parameter::Constant32(mem.read_int(ptr)), 4),
        ParameterMode::AbsoluteAddress => (Parameter::AbsoluteAddress(mem.read_int(ptr)), 4),
        ParameterMode::RelativeAddress => (Parameter::RelativeAddress(mem.read_int(ptr)), 4),
        ParameterMode::Register => (Parameter::Register(mem.read_byte(ptr)), 1),
        ParameterMode::RegisterAddress => (Parameter::RegisterAddress(mem.read_int(ptr)), 4),
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Operation {
    Halt,
    Nop,
    Jump(Parameter),
    JumpEqual(Parameter),
    JumpNotEqual(Parameter),
    JumpGreaterThan(Parameter),
    JumpGreaterEqual(Parameter),
    JumpLessThan(Parameter),
    JumpLessEqual(Parameter),
    CompareSigned(Parameter, Parameter),
    CompareFloat(Parameter, Parameter),
    CompareUnsigned(Parameter, Parameter),
    Call(Parameter),
    SysCall(Parameter),
    Return,
}

#[derive(Clone, Copy, Debug)]
pub struct Instruction {
    pub op: Operation,
    pub len: u32,
}

impl Instruction {
    fn no_params(op: OpCode) -> Self {
        let op = match op {
            OpCode::Halt => Operation::Halt,
            OpCode::Nop => Operation::Nop,
            OpCode::Return => Operation::Return,
            _ => unreachable!("No operation could be found for the no parameter OpCode"),
        };
        Instruction { op, len: 1 }
    }
    fn with_params(op_code: OpCode, first_par: Parameter, sec_par: Parameter, len: u32) -> Self {
        let op = match op_code {
            OpCode::Jump => Operation::Jump(first_par),
            OpCode::JumpEqual => Operation::JumpEqual(first_par),
            OpCode::JumpNotEqual => Operation::JumpNotEqual(first_par),
            OpCode::JumpGreaterThan => Operation::JumpGreaterThan(first_par),
            OpCode::JumpGreaterEqual => Operation::JumpGreaterEqual(first_par),
            OpCode::JumpLessThan => Operation::JumpLessThan(first_par),
            OpCode::JumpLessEqual => Operation::JumpLessEqual(first_par),
            OpCode::CompareSigned => Operation::CompareSigned(first_par, sec_par),
            OpCode::CompareFloat => Operation::CompareFloat(first_par, sec_par),
            OpCode::CompareUnsigned => Operation::CompareUnsigned(first_par, sec_par),
            _ => unreachable!(),
        };
        Instruction { op, len }
    }
}
