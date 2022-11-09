mod core_functions;
mod opcode_methods;
pub mod memory;
pub mod stack;

use memory::Memory;
use stack::Stack;

use crate::opcodes;
use bitflags::bitflags;
use std::collections::HashMap;

// Use bitflags to make bit operations more straightforward
bitflags! {
    pub struct CpuFlags: u8 {
        const CARRY             = 0b00000001;
        const ZERO              = 0b00000010;
        const INTERRUPT_DISABLE = 0b00000100;
        const DECIMAL_MODE      = 0b00001000;
        const BREAK             = 0b00010000;
        const BREAK2            = 0b00100000;
        const OVERFLOW          = 0b01000000;
        const NEGATIVE          = 0b10000000;
    }
}

// Stack memory space is [0x0100 .. 0x1FF]
// Stack starts at 0x1FF and grows down
// On reset of stack pointer: LDX #$FF, TXS
const STACK_RESET: u8 = 0xff;
const STACK: u16 = 0x0100;

#[derive(Debug)]
pub enum AddressingMode {
    Immediate,
    ZeroPage,
    ZeroPageX,
    ZeroPageY,
    Absolute,
    AbsoluteX,
    AbsoluteY,
    IndirectX,
    IndirectY,
    NonAddressing,
}

pub struct CPU {
    pub register_a: u8,
    pub register_x: u8,
    pub register_y: u8,
    pub status: CpuFlags,
    pub stack_pointer: u8,
    pub program_counter: u16,
    memory: [u8; 0xFFFF],
}

impl CPU {
    pub fn new() -> Self {
        CPU {
            register_a: 0,
            register_x: 0,
            register_y: 0,
            stack_pointer: STACK_RESET,
            program_counter: 0,
            status: CpuFlags::from_bits_truncate(0b00100100), // Set break 2 and interrupt disable: https://stackoverflow.com/questions/16913423/why-is-the-initial-state-of-the-interrupt-flag-of-the-6502-a-1
            memory: [0; 0xFFFF], // 0xFFFF size array filled with 0
        }
    }

    pub fn reset(&mut self) {
        self.register_a = 0;
        self.register_x = 0;
        self.register_y = 0;
        self.stack_pointer = STACK_RESET;
        self.status = CpuFlags::from_bits_truncate(0b00100100);
        self.program_counter = self.mem_read_u16(0xFFFC);
    }

    pub fn load_and_run(&mut self, program: Vec<u8>) {
        self.load(program);
        self.reset();
        self.run()
    }

    pub fn load(&mut self, program: Vec<u8>) {
        // From 0x8000 to 0x8000 + program length
        // .. means the full length of &program
        // copy from slice into memory at that range
        // Basically, we're loading the op codes from the program into memory starting from 0x8000;
        // Program ROM is from 0x8000 to 0xFFFF
        // // (Changed to 0x0600 for the snake game)
        self.memory[0x0600 .. (0x0600 + program.len())].copy_from_slice(&program[..]);
        self.mem_write_u16(0xFFFC, 0x0600);
    } 

    pub fn run(&mut self) {
        self.run_with_callback(|_| {});
    }

    // Defined this for the snack game
    // So we can pass the handle user input and screen state update
    // So we can perform all of those methods between each op cycle
    pub fn run_with_callback<F>(&mut self, mut callback: F)
    where 
        F: FnMut(&mut CPU),
    {
        let ref opcodes: HashMap<u8, &'static opcodes::OpCode> = *opcodes::OPCODES_MAP;

        loop {
            let code = self.mem_read(self.program_counter);
            self.program_counter += 1;
            let program_counter_state = self.program_counter;

            let opcode = opcodes.get(&code).unwrap();

            match code {
                // ADC
                0x69 | 0x65 | 0x75 | 0x6d | 0x7d | 0x79 | 0x61 | 0x71 => self.adc(&opcode.mode),

                // AND
                0x29 | 0x25 | 0x35 | 0x2d | 0x3d | 0x39 | 0x21 | 0x31 => self.and(&opcode.mode),

                // ASL
                0x0a => self.asl_accumulator(),
                0x06 | 0x16 | 0x0e | 0x1e => self.asl(&opcode.mode),

                // BCC
                0x90 => self.branch(!self.status.contains(CpuFlags::CARRY)),

                // BCS
                0xb0 => self.branch(self.status.contains(CpuFlags::CARRY)),

                // BEQ
                0xf0 => self.branch(self.status.contains(CpuFlags::ZERO)),

                // BIT
                0x24 | 0x2c => self.bit(&opcode.mode),

                // BMI
                0x30 => self.branch(self.status.contains(CpuFlags::NEGATIVE)),

                // BNE
                0xd0 => self.branch(!self.status.contains(CpuFlags::ZERO)),

                // BPL
                0x10 => self.branch(!self.status.contains(CpuFlags::NEGATIVE)),

                // BRK
                0x00 => return,

                // BVC
                0x50 => self.branch(!self.status.contains(CpuFlags::OVERFLOW)),

                // BVS
                0x70 => self.branch(self.status.contains(CpuFlags::OVERFLOW)),

                // CLC
                0x18 => self.clear_carry_flag(), 

                // CLD
                0xd8 => self.clear_decimal_flag(), 

                // CLI
                0x58 => self.clear_interrupt_flag(), 

                // CLV
                0xb8 => self.clear_overflow_flag(),

                // CMP
                0xc9 | 0xc5 | 0xd5 | 0xcd | 0xdd | 0xd9 | 0xc1 | 0xd1 => self.cmp(&opcode.mode, self.register_a),

                // CPX
                0xe0 | 0xe4 | 0xec => self.cmp(&opcode.mode, self.register_x),

                // CPY
                0xc0 | 0xc4 | 0xcc => self.cmp(&opcode.mode, self.register_y),

                // DEC
                0xc6 | 0xd6 | 0xce | 0xde => self.decrement_memory(&opcode.mode),

                // DEX
                0xca => self.decrement_register_x(),

                // DEY
                0x88 => self.decrement_register_y(),

                // EOR
                0x49 | 0x45 | 0x55 | 0x4d | 0x5d | 0x59 | 0x41 | 0x51 => self.exclusive_or(&opcode.mode),

                // INC
                0xe6 | 0xf6 | 0xee | 0xfe => self.increment_memory(&opcode.mode),

                // INX
                0xe8 => self.increment_register_x(), 

                // INY 
                0xc8 => self.increment_register_y(), 

                // JMP - Absolute
                0x4c => self.jump_absolute(),
                
                // Jmp - Indirect
                0x6c => self.jump_indirect(), 

                // JSR
                0x20 => self.jump_sub_routine(),

                // LDA 
                0xa9 | 0xa5 | 0xb5 | 0xad | 0xbd | 0xb9 | 0xa1 | 0xb1 => self.load_a_register(&opcode.mode),

                // LDX
                0xa2 | 0xa6 | 0xb6 | 0xae | 0xbe => self.load_x_register(&opcode.mode),

                // LDY
                0xa0 | 0xa4 | 0xb4 | 0xac | 0xbc => self.load_y_register(&opcode.mode),

                // LSR
                0x4a => self.logical_shift_right_accumulator(),
                0x46 | 0x56 | 0x4e | 0x5e => self.logical_shift_right(&opcode.mode),

                // NOP
                0xea => {},

                // ORA
                0x09 | 0x05 | 0x15 | 0x0d | 0x1d | 0x19 | 0x01 | 0x11 => self.logical_inclusive_or(&opcode.mode),

                // PHA
                0x48 => self.stack_push(self.register_a),

                // PHP
                0x08 => self.push_processor_status(),

                // PLA
                0x68 => self.pull_accumulator(),

                // PLP
                0x28 => self.pull_processor_status(),

                // ROL
                0x2a => self.rotate_left_accumulator(),
                0x26 | 0x36 | 0x2e | 0x3e => self.rotate_left(&opcode.mode), 

                // ROR
                0x6a => self.rotate_right_accumulator(),
                0x66 | 0x76 | 0x6e | 0x7e => self.rotate_right(&opcode.mode),

                // RTI
                0x40 => self.return_from_interrupt(),

                // RTS
                0x60 => self.program_counter = self.stack_pop_u16() + 1,

                // SBC
                0xe9 | 0xe5 | 0xf5 | 0xed | 0xfd | 0xf9 | 0xe1 | 0xf1 => self.subtract_with_carry(&opcode.mode),

                // SEC
                0x38 => self.set_carry_flag(),

                // SED
                0xf8 => self.set_decimal_flag(),

                // SEI
                0x78 => self.set_interrupt_disable(),

                // STA
                0x85 | 0x95 | 0x8d | 0x9d | 0x99 | 0x81 | 0x91 => self.store_accumulator(&opcode.mode),

                // STX
                0x86 | 0x96 | 0x8e => self.store_x_register(&opcode.mode),

                // STY
                0x84 | 0x94 | 0x8c => self.store_y_register(&opcode.mode),

                // TAX
                0xAA => self.transfer_accumulator_x(),

                // TAY
                0xa8 => self.transfer_accumulator_y(),

                // TSX
                0xBA => self.transfer_stack_pointer_to_x(),

                // TXA 
                0x8A => self.transfer_x_accumulator(),

                // TXS
                0x9A => self.transfer_x_to_stack_pointer(),

                // TYA
                0x98 => self.transfer_y_accumulator(),

                _ => panic!("Unexpected op code"),

            }

            if program_counter_state == self.program_counter {
                self.program_counter += (opcode.len - 1) as u16;
            }

            callback(self);
        }
    }
}

#[cfg(test)]
mod test {
    use super::{CPU, memory::Memory}; 

    #[test]
    fn test_0xa9_lda_immediate_load_data() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xA9, 0x05, 0x00]);
        assert_eq!(cpu.register_a, 0x05);
        assert!(cpu.status.bits() & 0b0000_0010 == 0b00);
        assert!(cpu.status.bits() & 0b1000_0000 == 0);
    }

    #[test]
    fn test_0xa9_lda_zero_flag() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xA9, 0x00, 0x00]);
        assert!(cpu.status.bits() & 0b0000_0010 == 0b10);
    }

    #[test]
    fn test_0xaa_tax_move_a_to_x() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0x0a, 0xaa, 0x00]);
        assert_eq!(cpu.register_x, 10);
    }

    #[test]
    fn test_5_ops_working_together() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0xc0, 0xaa, 0xe8, 0x00]);
        assert_eq!(cpu.register_x, 0xc1);
    }

    #[test]
    fn test_inx_overflow() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0xff, 0xaa, 0xe8, 0xe8, 0x00]);
        assert_eq!(cpu.register_x, 1);
    }
    
    #[test]
    fn test_lda_from_memory() {
        let mut cpu = CPU::new();
        cpu.mem_write(0x10, 0x55);
        cpu.load_and_run(vec![0xa5, 0x10, 0x00]);
        assert_eq!(cpu.register_a, 0x55);
    }

    #[test]
    fn test_adc_immediate() {
        let mut cpu = CPU::new();
        // Adding 255 to 5
        cpu.load_and_run(vec![0xA9, 0x05, 0x69, 0xFF, 0x00]);
        assert_eq!(cpu.register_a, 4);
    }
}
