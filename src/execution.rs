use std::collections::HashMap;

use crate::{
    hardware::{Cpu, Memory},
    instruction::{parse_instruction, Instruction},
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

        let effect = self.execute_instruction(instruction)?;
        // increase instruction pointer
        if instruction.increase_program_counter() {
            self.cpu.p += len;
        }
        Ok(effect)
    }

    pub fn execute_instruction(&mut self, instr: Instruction) -> Result<Effect, Tx8Error> {
        match instr {
            Instruction::Halt => return Ok(Effect::Halted),
            Instruction::Nop => (),
            Instruction::Jump(_, _) => todo!(),
            Instruction::CompareSigned(_, _) => todo!(),
            Instruction::CompareFloat(_, _) => todo!(),
            Instruction::CompareUnsigned(_, _) => todo!(),
            Instruction::Call(_) => todo!(),
            Instruction::SysCall(value) => self.sys_call(value.val)?,
            Instruction::Return => todo!(),
        };
        Ok(Effect::None)
    }

    fn sys_call(&self, val: u32) -> Result<(), Tx8Error> {
        if let Some(&str) = self.sys_call_map.get(&val) {
            match str {
                "print" => println!("successfully triggered print"),
                _ => return Err(Tx8Error::InvalidSysCall),
            }
            Ok(())
        } else {
            Err(Tx8Error::InvalidSysCall)
        }
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
