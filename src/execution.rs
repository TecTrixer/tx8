use std::collections::HashMap;

use crate::{
    hardware::{Cpu, Memory},
    instruction::{parse_instruction, Comparison, Instruction, Size, Value, Writable, Write},
    Tx8Error,
};

#[derive(Clone, Debug)]
pub struct Execution<'a> {
    cpu: Cpu,
    memory: Memory,
    sys_call_map: HashMap<u32, &'a str>,
}

impl<'a> Execution<'a> {
    pub fn new_with_rom(data: &[u8]) -> Self {
        let mut sys_call_map = HashMap::new();
        let sys_calls = ["print"];
        for sys_call in sys_calls {
            sys_call_map.insert(hash(sys_call), sys_call);
        }
        Execution {
            cpu: Cpu::new(),
            memory: Memory::load_rom(data),
            sys_call_map,
        }
    }
    pub fn next_step(&mut self) -> Result<Effect, Tx8Error> {
        let (instruction, len) = parse_instruction(&self.cpu, &self.memory, self.cpu.p)?;

        let effect = self.execute_instruction(instruction, len)?;
        // increase instruction pointer
        if instruction.increase_program_counter() {
            self.cpu.p += len;
        }
        Ok(effect)
    }

    pub fn execute_instruction(
        &mut self,
        instr: Instruction,
        len: u32,
    ) -> Result<Effect, Tx8Error> {
        match instr {
            Instruction::Halt => return Ok(Effect::Halted),
            Instruction::Nop => (),
            Instruction::Jump(value, comp) => self.jump(value.val, comp, len),
            Instruction::CompareSigned(val, val2) => self.compare_signed(val, val2.val),
            Instruction::CompareFloat(val, val2) => self.compare_float(val.val, val2.val),
            Instruction::CompareUnsigned(val, val2) => self.compare_unsigned(val.val, val2.val),
            Instruction::Call(_) => todo!(),
            Instruction::SysCall(value) => self.sys_call(value.val)?,
            Instruction::Return => todo!(),
            Instruction::Load(to, val) => self.load(to, val.val)?,
        };
        Ok(Effect::None)
    }

    fn sys_call(&self, val: u32) -> Result<(), Tx8Error> {
        if let Some(&str) = self.sys_call_map.get(&val) {
            match str {
                "print" => print!("{}", self.memory.read_byte(self.cpu.s) as char),
                _ => return Err(Tx8Error::InvalidSysCall),
            }
            Ok(())
        } else {
            Err(Tx8Error::InvalidSysCall)
        }
    }

    fn jump(&mut self, val: u32, comp: Comparison, instr_len: u32) {
        let r = self.cpu.r as i32;
        let cond = match comp {
            Comparison::None => true,
            Comparison::Equal => r == 0,
            Comparison::NotEqual => r != 0,
            Comparison::Greater => r > 0,
            Comparison::GreaterEqual => r >= 0,
            Comparison::Less => r < 0,
            Comparison::LessEqual => r <= 0,
        };
        if cond {
            self.cpu.p = val;
        } else {
            self.cpu.p += instr_len;
        }
    }

    fn compare_signed(&mut self, val: Value, val2: u32) {
        self.cpu.r = match val.size {
            Size::Byte => i8::signum((val.val as i8) - (val2 as i8)) as u32,
            Size::Short => i16::signum((val.val as i16) - (val2 as i16)) as u32,
            Size::Int => i32::signum((val.val as i32) - (val2 as i32)) as u32,
        };
    }
    fn compare_float(&mut self, val: u32, val2: u32) {
        self.cpu.r = f32::signum(f32::from_bits(val) - f32::from_bits(val2)) as i32 as u32;
    }
    fn compare_unsigned(&mut self, val: u32, val2: u32) {
        self.cpu.r = i64::signum((val as i64) - (val2 as i64)) as u32;
    }

    fn load(&mut self, to: Writable, val: u32) -> Result<(), Tx8Error> {
        to.write(&mut self.memory, &mut self.cpu, val)
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Effect {
    None,
    Halted,
}

fn hash(s: &str) -> u32 {
    let mut s = s.chars();
    let mut h = s.next().unwrap_or(0 as char) as u32;
    for c in s {
        h = (h << 5) - h + (c as u32);
    }
    h
}
