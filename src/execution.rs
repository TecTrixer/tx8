use std::{collections::HashMap, io::Read, ops::Neg};

use crate::{
    hardware::{Cpu, Memory},
    instruction::{parse_instruction, Comparison, Instruction, Type},
    parameter::{Size, Value, Writable, Write},
    random::*,
    Tx8Error,
};

#[derive(Clone, Debug)]
pub struct Execution<'a> {
    cpu: Cpu,
    memory: Memory,
    sys_call_map: HashMap<u32, &'a str>,
    rand: Rand,
    input: std::vec::IntoIter<u8>,
}

impl<'a> Execution<'a> {
    pub fn new_with_rom(data: &[u8]) -> Result<Self, Tx8Error> {
        let mut sys_call_map = HashMap::new();
        let sys_calls = [
            "print_u32",
            "print_i32",
            "print_f32",
            "print_u8",
            "print_char",
            "test_af",
            "test_au",
            "test_ai",
            "test_rf",
            "test_r",
            "test_ri",
            "read_char",
        ];
        for sys_call in sys_calls {
            sys_call_map.insert(hash(sys_call), sys_call);
        }
        let rand = Rand::new();
        let mut input = Vec::new();
        let mut stdin = std::io::stdin().lock();
        stdin
            .read_to_end(&mut input)
            .map_err(|_| Tx8Error::NoInputGiven)?;
        Ok(Execution {
            cpu: Cpu::new(),
            memory: Memory::load_rom(data)?,
            sys_call_map,
            rand,
            input: input.into_iter(),
        })
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
            Instruction::Call(val) => self.call(val, len),
            Instruction::SysCall(value) => self.sys_call(value.val)?,
            Instruction::Return => self.ret(),
            Instruction::Load(to, val) => self.load(to, val)?,
            Instruction::Push(val) => self.push(val),
            Instruction::Pop(val) => self.pop(val)?,
            Instruction::Add(to, val, val2, kind) => self.add(to, val, val2, kind)?,
            Instruction::Sub(to, val, val2, kind) => self.sub(to, val, val2, kind)?,
            Instruction::Mul(to, val, val2, kind) => self.mul(to, val, val2, kind)?,
            Instruction::DivMod(to, val, val2, kind, is_div) => {
                self.div(to, val, val2, kind, is_div)?
            }
            Instruction::MaxMin(to, val, val2, kind, is_max) => {
                self.max_min(to, val, val2, kind, is_max)?
            }
            Instruction::AbsSign(to, val, kind, is_abs) => self.abs_sign(to, val, kind, is_abs)?,
            Instruction::And(to, val, val2) => self.and(to, val, val2)?,
            Instruction::Or(to, val, val2) => self.or(to, val, val2)?,
            Instruction::Not(to, val) => self.not(to, val)?,
            Instruction::Nand(to, val, val2) => self.nand(to, val, val2)?,
            Instruction::Xor(to, val, val2) => self.xor(to, val, val2)?,
            Instruction::ShiftLogicalRight(to, val, val2) => self.slr(to, val, val2)?,
            Instruction::ShiftArithRight(to, val, val2) => self.sar(to, val, val2)?,
            Instruction::ShiftLogicLeft(to, val, val2) => self.sll(to, val, val2)?,
            Instruction::RotateRight(to, val, val2) => self.ror(to, val, val2)?,
            Instruction::RotateLeft(to, val, val2) => self.rol(to, val, val2)?,
            Instruction::Set(to, val, val2) => self.set(to, val, val2)?,
            Instruction::Clear(to, val, val2) => self.clear(to, val, val2)?,
            Instruction::Toggle(to, val, val2) => self.toggle(to, val, val2)?,
            Instruction::Test(val, val2) => self.test(val, val2),
            Instruction::Sin(to, val) => self.sin(to, val)?,
            Instruction::Cos(to, val) => self.cos(to, val)?,
            Instruction::Tan(to, val) => self.tan(to, val)?,
            Instruction::ArcSin(to, val) => self.arcsin(to, val)?,
            Instruction::ArcCos(to, val) => self.arccos(to, val)?,
            Instruction::ArcTan(to, val) => self.arctan(to, val)?,
            Instruction::ArcTan2(to, val, val2) => self.arctan2(to, val, val2)?,
            Instruction::Sqrt(to, val) => self.sqrt(to, val)?,
            Instruction::Pow(to, val, val2) => self.pow(to, val, val2)?,
            Instruction::Exp(to, val) => self.exp(to, val)?,
            Instruction::Log(to, val) => self.log(to, val)?,
            Instruction::Log2(to, val) => self.log2(to, val)?,
            Instruction::Log10(to, val) => self.log10(to, val)?,
            Instruction::Rand(to) => self.rand(to)?,
            Instruction::RSeed(val) => self.rseed(val),
            Instruction::ItoF(to, val) => self.i_to_f(to, val)?,
            Instruction::FtoI(to, val) => self.f_to_i(to, val)?,
            Instruction::UtoF(to, val) => self.u_to_f(to, val)?,
            Instruction::FtoU(to, val) => self.f_to_f(to, val)?,
        };
        Ok(Effect::None)
    }

    fn sys_call(&mut self, val: u32) -> Result<(), Tx8Error> {
        if let Some(&str) = self.sys_call_map.get(&val) {
            match str {
                "print_u32" => print!("{}", self.memory.read_int(self.cpu.s)),
                "print_i32" => print!("{}", self.memory.read_int(self.cpu.s) as i32),
                "print_f32" => print!("{}", f32::from_bits(self.memory.read_int(self.cpu.s))),
                "print_char" => print!("{}", self.memory.read_int(self.cpu.s) as u8 as char),
                "print_u8" => print!("{}", self.memory.read_int(self.cpu.s) as u8),
                "test_af" => println!("{}", f32::from_bits(self.cpu.a)),
                "test_au" => println!("{:x}", self.cpu.a),
                "test_ai" => println!("{}", self.cpu.a as i32),
                "test_rf" => println!("{}", f32::from_bits(self.cpu.r)),
                "test_r" => println!("{:x}", self.cpu.r),
                "test_ri" => println!("{}", self.cpu.r as i32),
                "read_char" => {
                    if let Some(char) = self.input.next() {
                        self.cpu.o = char as u8 as u32;
                    } else {
                        return Err(Tx8Error::NoInputGiven);
                    }
                }
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

    fn load(&mut self, to: Writable, val: Value) -> Result<(), Tx8Error> {
        to.write_size(&mut self.memory, &mut self.cpu, val.val, val.size)
    }

    fn push(&mut self, val: Value) {
        self.cpu.s -= val.size.bytes();
        match val.size {
            Size::Byte => self.memory.write_byte(self.cpu.s, (val.val & 0xff) as u8),
            Size::Short => self
                .memory
                .write_short(self.cpu.s, (val.val & 0xffff) as u16),
            Size::Int => self.memory.write_int(self.cpu.s, val.val),
        }
    }

    fn pop(&mut self, val: Writable) -> Result<(), Tx8Error> {
        let value = match val.size() {
            Size::Byte => self.memory.read_byte(self.cpu.s) as u32,
            Size::Short => self.memory.read_short(self.cpu.s) as u32,
            Size::Int => self.memory.read_int(self.cpu.s),
        };
        val.write(&mut self.memory, &mut self.cpu, value)?;
        self.cpu.s += val.size().bytes();
        Ok(())
    }

    fn call(&mut self, val: Value, len: u32) {
        self.push(Value::new(self.cpu.p + len, Size::Int));
        self.cpu.p = val.val;
    }
    fn ret(&mut self) {
        let val = self.memory.read_int(self.cpu.s);
        self.cpu.s += 4;
        self.cpu.p = val;
    }

    fn add(
        &mut self,
        to: Writable,
        first: Value,
        second: Value,
        kind: Type,
    ) -> Result<(), Tx8Error> {
        let (res_signed, overflow_signed) = match to.size() {
            Size::Byte => {
                let (res, over) = (first.val as i8).overflowing_add(second.val as i8);
                (res as i32, over)
            }
            Size::Short => {
                let (res, over) = (first.val as i16).overflowing_add(second.val as i16);
                (res as i32, over)
            }
            Size::Int => (first.val as i32).overflowing_add(second.val as i32),
        };
        let (res, overflow) = match to.size() {
            Size::Byte => {
                let (res, over) = (first.val as u8).overflowing_add(second.val as u8);
                (res as u32, over)
            }
            Size::Short => {
                let (res, over) = (first.val as u16).overflowing_add(second.val as u16);
                (res as u32, over)
            }
            Size::Int => first.val.overflowing_add(second.val),
        };

        match kind {
            Type::Signed => to.write(&mut self.memory, &mut self.cpu, res_signed as u32)?,
            Type::Unsigned => to.write(&mut self.memory, &mut self.cpu, res)?,
            Type::Float => {
                let res = f32::to_bits(f32::from_bits(first.val) + f32::from_bits(second.val));
                to.write(&mut self.memory, &mut self.cpu, res)?;
                return Ok(());
            }
        }

        self.cpu.r = if overflow { 0b1 } else { 0b0 } | if overflow_signed { 0b10 } else { 0b0 };
        Ok(())
    }
    fn sub(
        &mut self,
        to: Writable,
        first: Value,
        second: Value,
        kind: Type,
    ) -> Result<(), Tx8Error> {
        let (res_signed, overflow_signed) = match to.size() {
            Size::Byte => {
                let (res, over) = (first.val as i8).overflowing_sub(second.val as i8);
                (res as i32, over)
            }
            Size::Short => {
                let (res, over) = (first.val as i16).overflowing_sub(second.val as i16);
                (res as i32, over)
            }
            Size::Int => (first.val as i32).overflowing_sub(second.val as i32),
        };
        let (res, overflow) = match to.size() {
            Size::Byte => {
                let (res, over) = (first.val as u8).overflowing_sub(second.val as u8);
                (res as u32, over)
            }
            Size::Short => {
                let (res, over) = (first.val as u16).overflowing_sub(second.val as u16);
                (res as u32, over)
            }
            Size::Int => first.val.overflowing_sub(second.val),
        };

        match kind {
            Type::Signed => to.write(&mut self.memory, &mut self.cpu, res_signed as u32)?,
            Type::Unsigned => to.write(&mut self.memory, &mut self.cpu, res)?,
            Type::Float => {
                let res = f32::to_bits(f32::from_bits(first.val) - f32::from_bits(second.val));
                to.write(&mut self.memory, &mut self.cpu, res)?;
                return Ok(());
            }
        }

        self.cpu.r = if overflow { 0b1 } else { 0b0 } | if overflow_signed { 0b10 } else { 0b0 };
        Ok(())
    }

    fn mul(&mut self, to: Writable, val: Value, val2: Value, kind: Type) -> Result<(), Tx8Error> {
        match kind {
            Type::Signed => {
                let res = val.val as i32 as i64 * val2.val as i32 as i64;
                to.write(&mut self.memory, &mut self.cpu, res as u32)?;
                self.cpu.r = (res >> 32) as u32;
            }
            Type::Unsigned => {
                let res = val.val as u64 * val2.val as u64;
                to.write(&mut self.memory, &mut self.cpu, res as u32)?;
                self.cpu.r = (res >> 32) as u32;
            }
            Type::Float => {
                let res = f32::to_bits(f32::from_bits(val.val) * f32::from_bits(val2.val));
                to.write(&mut self.memory, &mut self.cpu, res as u32)?;
            }
        }
        Ok(())
    }

    fn div(
        &mut self,
        to: Writable,
        val: Value,
        val2: Value,
        kind: Type,
        is_div: bool,
    ) -> Result<(), Tx8Error> {
        if kind != Type::Float && val2.val == 0 {
            return Err(Tx8Error::DivisionByZero);
        }
        let (res, remainder) = match kind {
            Type::Signed => {
                let res = val.val as i32 / val2.val as i32;
                let remainder = val.val as i32 % val2.val as i32;
                (res as u32, remainder as u32)
            }
            Type::Unsigned => {
                let res = val.val / val2.val;
                let remainder = val.val % val2.val;
                (res, remainder)
            }
            Type::Float => {
                let res = f32::to_bits(f32::from_bits(val.val) / f32::from_bits(val2.val));
                let remainder = f32::to_bits(f32::from_bits(val.val) % f32::from_bits(val2.val));
                (res, remainder)
            }
        };
        if is_div {
            to.write(&mut self.memory, &mut self.cpu, res)?;
            self.cpu.r = remainder;
        } else {
            to.write(&mut self.memory, &mut self.cpu, remainder)?;
            self.cpu.r = res;
        }
        Ok(())
    }

    fn max_min(
        &mut self,
        to: Writable,
        val: Value,
        val2: Value,
        kind: Type,
        is_max: bool,
    ) -> Result<(), Tx8Error> {
        let (max, min) = match kind {
            Type::Signed => {
                if val.val as i32 > val2.val as i32 {
                    (val.val, val2.val)
                } else {
                    (val2.val, val.val)
                }
            }
            Type::Unsigned => {
                if val.val > val2.val {
                    (val.val, val2.val)
                } else {
                    (val2.val, val.val)
                }
            }
            Type::Float => {
                if f32::from_bits(val.val) > f32::from_bits(val2.val) {
                    (val.val, val2.val)
                } else {
                    (val2.val, val.val)
                }
            }
        };
        if is_max {
            to.write(&mut self.memory, &mut self.cpu, max)?;
            self.cpu.r = min;
        } else {
            to.write(&mut self.memory, &mut self.cpu, min)?;
            self.cpu.r = max;
        }
        Ok(())
    }

    fn abs_sign(
        &mut self,
        to: Writable,
        val: Value,
        kind: Type,
        is_abs: bool,
    ) -> Result<(), Tx8Error> {
        let (res, sign) = match kind {
            Type::Signed => {
                let value = val.val as i32;
                if value == 0 {
                    (val.val, 0)
                } else if value < 0 {
                    (value.neg() as u32, -1i32 as u32)
                } else {
                    (val.val, 1)
                }
            }
            Type::Unsigned => unreachable!(),
            Type::Float => {
                let value = f32::from_bits(val.val);
                if value == 0.0 {
                    (val.val, f32::to_bits(0.0))
                } else if value < 0.0 {
                    (f32::to_bits(value.neg()), f32::to_bits(-1.0))
                } else {
                    (val.val, f32::to_bits(1.0))
                }
            }
        };
        if is_abs {
            to.write(&mut self.memory, &mut self.cpu, res)?;
            self.cpu.r = sign as u32;
        } else {
            to.write(&mut self.memory, &mut self.cpu, sign as u32)?;
            self.cpu.r = res;
        }
        Ok(())
    }

    fn and(&mut self, to: Writable, val: Value, val2: Value) -> Result<(), Tx8Error> {
        to.write(&mut self.memory, &mut self.cpu, val.val & val2.val)
    }

    fn or(&mut self, to: Writable, val: Value, val2: Value) -> Result<(), Tx8Error> {
        to.write(&mut self.memory, &mut self.cpu, val.val | val2.val)
    }

    fn not(&mut self, to: Writable, val: Value) -> Result<(), Tx8Error> {
        to.write(&mut self.memory, &mut self.cpu, !val.val)
    }

    fn nand(&mut self, to: Writable, val: Value, val2: Value) -> Result<(), Tx8Error> {
        to.write(&mut self.memory, &mut self.cpu, !(val.val & val2.val))
    }

    fn xor(&mut self, to: Writable, val: Value, val2: Value) -> Result<(), Tx8Error> {
        to.write(&mut self.memory, &mut self.cpu, val.val ^ val2.val)
    }

    fn slr(&mut self, to: Writable, val: Value, val2: Value) -> Result<(), Tx8Error> {
        let filter = match to.size() {
            Size::Byte => 0b111,
            Size::Short => 0b1111,
            Size::Int => 0b11111,
        };
        let shift_amount = val2.val & filter;
        let res = val.val >> shift_amount;
        let shifted_out = val.val & ((1 << shift_amount) - 1);
        to.write(&mut self.memory, &mut self.cpu, res)?;
        self.cpu.r = shifted_out;
        Ok(())
    }

    fn sar(&mut self, to: Writable, val: Value, val2: Value) -> Result<(), Tx8Error> {
        let filter = match to.size() {
            Size::Byte => 0b111,
            Size::Short => 0b1111,
            Size::Int => 0b11111,
        };
        let shift_amount = val2.val & filter;
        let res = match to.size() {
            Size::Byte => (val.val as i8 >> shift_amount) as i32,
            Size::Short => (val.val as i16 >> shift_amount) as i32,
            Size::Int => val.val as i32 >> shift_amount,
        };
        let shifted_out = val.val & ((1 << shift_amount) - 1);
        to.write(&mut self.memory, &mut self.cpu, res as u32)?;
        self.cpu.r = shifted_out;
        Ok(())
    }

    fn sll(&mut self, to: Writable, val: Value, val2: Value) -> Result<(), Tx8Error> {
        let (filter, size) = match to.size() {
            Size::Byte => (0b111, 8),
            Size::Short => (0b1111, 16),
            Size::Int => (0b11111, 32),
        };
        let shift_amount = val2.val & filter;
        let res = val.val << shift_amount;
        let shifted_out = if shift_amount == 0 {
            0
        } else {
            val.val >> (size - shift_amount)
        };
        to.write(&mut self.memory, &mut self.cpu, res)?;
        self.cpu.r = shifted_out;
        Ok(())
    }

    fn ror(&mut self, to: Writable, val: Value, val2: Value) -> Result<(), Tx8Error> {
        let res = match to.size() {
            Size::Byte => (val.val as u8).rotate_right(val2.val) as u32,
            Size::Short => (val.val as u16).rotate_right(val2.val) as u32,
            Size::Int => val.val.rotate_right(val2.val),
        };
        to.write(&mut self.memory, &mut self.cpu, res)
    }

    fn rol(&mut self, to: Writable, val: Value, val2: Value) -> Result<(), Tx8Error> {
        let res = match to.size() {
            Size::Byte => (val.val as u8).rotate_left(val2.val) as u32,
            Size::Short => (val.val as u16).rotate_left(val2.val) as u32,
            Size::Int => val.val.rotate_left(val2.val),
        };
        to.write(&mut self.memory, &mut self.cpu, res)
    }

    fn set(&mut self, to: Writable, val: Value, val2: Value) -> Result<(), Tx8Error> {
        let filter = match to.size() {
            Size::Byte => 0b111,
            Size::Short => 0b1111,
            Size::Int => 0b11111,
        };
        let i = val2.val & filter;
        let res = val.val | (1 << i);
        to.write(&mut self.memory, &mut self.cpu, res)?;
        let bit = val.val & (1 << i);
        if bit != 0 {
            self.cpu.r = 1;
        } else {
            self.cpu.r = 0;
        }
        Ok(())
    }

    fn clear(&mut self, to: Writable, val: Value, val2: Value) -> Result<(), Tx8Error> {
        let filter = match to.size() {
            Size::Byte => 0b111,
            Size::Short => 0b1111,
            Size::Int => 0b11111,
        };
        let i = val2.val & filter;
        let res = val.val & (!(1 << i));
        to.write(&mut self.memory, &mut self.cpu, res)?;
        let bit = val.val & (1 << i);
        if bit != 0 {
            self.cpu.r = 1;
        } else {
            self.cpu.r = 0;
        }
        Ok(())
    }

    fn toggle(&mut self, to: Writable, val: Value, val2: Value) -> Result<(), Tx8Error> {
        let filter = match to.size() {
            Size::Byte => 0b111,
            Size::Short => 0b1111,
            Size::Int => 0b11111,
        };
        let i = val2.val & filter;
        let res = val.val ^ (1 << i);
        to.write(&mut self.memory, &mut self.cpu, res)?;
        let bit = val.val & (1 << i);
        if bit != 0 {
            self.cpu.r = 1;
        } else {
            self.cpu.r = 0;
        }
        Ok(())
    }

    fn test(&mut self, val: Value, val2: Value) {
        let filter = match val.size {
            Size::Byte => 0b111,
            Size::Short => 0b1111,
            Size::Int => 0b11111,
        };
        let i = val2.val & filter;
        let res = val.val & (1 << i);
        if res != 0 {
            self.cpu.r = 1;
        } else {
            self.cpu.r = 0;
        }
    }

    fn sin(&mut self, to: Writable, val: Value) -> Result<(), Tx8Error> {
        let res = f32::from_bits(val.val).sin();
        to.write(&mut self.memory, &mut self.cpu, f32::to_bits(res))
    }

    fn cos(&mut self, to: Writable, val: Value) -> Result<(), Tx8Error> {
        let res = f32::from_bits(val.val).cos();
        to.write(&mut self.memory, &mut self.cpu, f32::to_bits(res))
    }

    fn tan(&mut self, to: Writable, val: Value) -> Result<(), Tx8Error> {
        let res = f32::from_bits(val.val).tan();
        to.write(&mut self.memory, &mut self.cpu, f32::to_bits(res))
    }

    fn arcsin(&mut self, to: Writable, val: Value) -> Result<(), Tx8Error> {
        let res = f32::from_bits(val.val).asin();
        to.write(&mut self.memory, &mut self.cpu, f32::to_bits(res))
    }

    fn arccos(&mut self, to: Writable, val: Value) -> Result<(), Tx8Error> {
        let res = f32::from_bits(val.val).acos();
        to.write(&mut self.memory, &mut self.cpu, f32::to_bits(res))
    }

    fn arctan(&mut self, to: Writable, val: Value) -> Result<(), Tx8Error> {
        let res = f32::from_bits(val.val).atan();
        to.write(&mut self.memory, &mut self.cpu, f32::to_bits(res))
    }

    fn arctan2(&mut self, to: Writable, val: Value, val2: Value) -> Result<(), Tx8Error> {
        let res = f32::from_bits(val.val).atan2(f32::from_bits(val2.val));
        to.write(&mut self.memory, &mut self.cpu, f32::to_bits(res))
    }

    fn sqrt(&mut self, to: Writable, val: Value) -> Result<(), Tx8Error> {
        let res = f32::from_bits(val.val).sqrt();
        to.write(&mut self.memory, &mut self.cpu, f32::to_bits(res))
    }

    fn pow(&mut self, to: Writable, val: Value, val2: Value) -> Result<(), Tx8Error> {
        let res = f32::from_bits(val.val).powf(f32::from_bits(val2.val));
        to.write(&mut self.memory, &mut self.cpu, f32::to_bits(res))
    }

    fn exp(&mut self, to: Writable, val: Value) -> Result<(), Tx8Error> {
        let res = f32::from_bits(val.val).exp();
        to.write(&mut self.memory, &mut self.cpu, f32::to_bits(res))
    }

    fn log(&mut self, to: Writable, val: Value) -> Result<(), Tx8Error> {
        let res = f32::from_bits(val.val).ln();
        to.write(&mut self.memory, &mut self.cpu, f32::to_bits(res))
    }

    fn log2(&mut self, to: Writable, val: Value) -> Result<(), Tx8Error> {
        let res = f32::from_bits(val.val).log2();
        to.write(&mut self.memory, &mut self.cpu, f32::to_bits(res))
    }

    fn log10(&mut self, to: Writable, val: Value) -> Result<(), Tx8Error> {
        let res = f32::from_bits(val.val).log10();
        to.write(&mut self.memory, &mut self.cpu, f32::to_bits(res))
    }

    fn i_to_f(&mut self, to: Writable, val: Value) -> Result<(), Tx8Error> {
        to.write(
            &mut self.memory,
            &mut self.cpu,
            f32::to_bits(val.val as i32 as f32),
        )
    }

    fn f_to_i(&mut self, to: Writable, val: Value) -> Result<(), Tx8Error> {
        to.write(
            &mut self.memory,
            &mut self.cpu,
            f32::from_bits(val.val) as i32 as u32,
        )
    }

    fn u_to_f(&mut self, to: Writable, val: Value) -> Result<(), Tx8Error> {
        to.write(
            &mut self.memory,
            &mut self.cpu,
            f32::to_bits(val.val as f32),
        )
    }

    fn f_to_f(&mut self, to: Writable, val: Value) -> Result<(), Tx8Error> {
        to.write(
            &mut self.memory,
            &mut self.cpu,
            f32::from_bits(val.val) as u32,
        )
    }

    fn rand(&mut self, to: Writable) -> Result<(), Tx8Error> {
        let res = self.rand.next();
        let float = res as f32 / RANGE as f32;
        to.write(&mut self.memory, &mut self.cpu, f32::to_bits(float))?;
        self.cpu.r = res;
        Ok(())
    }

    fn rseed(&mut self, val: Value) {
        self.rand.set_seed(val.val);
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
