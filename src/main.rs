use std::num::Wrapping;

type Wu8 = Wrapping<u8>;

/*
Done: STY, STX, LDA, BRK, TAX, TXA, JMP
Partial: STA, LDY, LDX, JSR, RTS
TODO: ADC, AND, ASL, BCC, BCS, BEQ, BIT, BMI, BNE, BPL, BVC, BVS, CLC, CLD, CLI
CLV, CMP, CPX, CPY, DEC, DEX, DEY, EOR, INC, INX, INY, JMP, LSR, NOP, ORA,
PHA, PHP, PLA, PLP, ROL, ROR, RTI, SBC, SEC, SED, SEI, TAY, TSX, TXA, TXS,
TYA
 */

#[derive(Debug)]
#[allow(non_camel_case_types)]
pub enum AddressingMode {
    Immediate,
    ZeroPage,
    ZeroPage_X,
    ZeroPage_Y,
    Absolute,
    Absolute_X,
    Absolute_Y,
    Indirect,
    Indirect_X,
    Indirect_Y,
    NoneAddressing,
}

pub struct CPU {
    pub register_a: Wu8,
    pub register_x: Wu8,
    pub register_y: Wu8,
    pub status: u8,
    pub program_counter: u16,
    pub stack_pointer: u8,
    pub stack_location: u16,
    pub stack_size: u8,
    memory: [u8; 0xFFFF],
}

impl CPU {
    pub fn new() -> Self {
        CPU {
            register_a: Wrapping(0),
            register_x: Wrapping(0),
            register_y: Wrapping(0),
            status: 0,
            program_counter: 0,
            memory: [0; 0xFFFF],
            stack_pointer: 0xFF,
            stack_location: 0x100,
            stack_size: 0xFF,
        }
    }

    fn mem_read(&self, addr: u16) -> u8 {
        self.memory[addr as usize]
    }

    fn mem_write(&mut self, addr: u16, data: u8) {
        self.memory[addr as usize] = data;
    }

    fn mem_read_u16(&mut self, pos: u16) -> u16 {
        let lo = self.mem_read(pos) as u16;
        let hi = self.mem_read(pos + 1) as u16;
        (hi << 8) | (lo as u16)
    }

    fn stack_push(&mut self, byte: u8) {
        self.mem_write(self.stack_pointer.into(), byte);
        self.stack_pointer -= 1;
    }

    fn stack_pop(&mut self) -> u8 {
        let val = self.mem_read(self.stack_pointer.into());
        self.stack_pointer += 1;
        val
    }

    fn mem_write_u16(&mut self, pos: u16, data: u16) {
        let hi = (data >> 8) as u8;
        let lo = (data & 0xff) as u8;
        self.mem_write(pos, lo);
        self.mem_write(pos + 1, hi);
    }

    pub fn reset(&mut self) {
        self.register_a = Wrapping(0);
        self.register_x = Wrapping(0);
        self.register_y = Wrapping(0);
        self.status = 0;

        self.program_counter = self.mem_read_u16(0xFFFC);
    }

    pub fn load(&mut self, program: Vec<u8>) {
        self.memory[0x8000..(0x8000 + program.len())].copy_from_slice(&program[..]);
        self.mem_write_u16(0xFFFC, 0x8000);
    }

    pub fn init(&mut self, program: Vec<u8>) {
        self.load(program);
        self.reset();
    }

    pub fn load_and_run(&mut self, program: Vec<u8>) {
        self.load(program);
        self.reset();
        self.run()
    }

    pub fn run(&mut self) {
        // note: we move  intialization of program_counter from here to load function
        let mut mode: AddressingMode;
        loop {
            let opscode = self.mem_read(self.program_counter);
            self.program_counter += 1;
            mode = AddressingMode::NoneAddressing;

            match opscode {
                /* LDA */
                0xA9 => {
                    mode = AddressingMode::Immediate;
                    self.lda(&mode);
                }
                0xA5 => {
                    mode = AddressingMode::ZeroPage;
                    self.lda(&mode);
                }
                0xB5 => {
                    mode = AddressingMode::ZeroPage_X;
                    self.lda(&mode);
                }
                0xAD => {
                    mode = AddressingMode::Absolute;
                    self.lda(&mode);
                }

                0xBD => {
                    mode = AddressingMode::Absolute_X;
                    self.lda(&mode);
                }

                0xB9 => {
                    mode = AddressingMode::Absolute_Y;
                    self.lda(&mode);
                }

                0xA1 => {
                    mode = AddressingMode::Indirect_X;
                    self.lda(&mode);
                }

                0xB1 => {
                    mode = AddressingMode::Indirect_Y;
                    self.lda(&mode);
                }

                /* LDY */
                0xA0 => {
                    mode = AddressingMode::Immediate;
                    self.ldy(&mode);
                }

                /* LDX */
                0xA2 => {
                    mode = AddressingMode::Immediate;
                    self.ldx(&mode);
                }

                /* STA */
                0x85 => {
                    mode = AddressingMode::ZeroPage;
                    self.sta(&mode);
                }

                0x95 => {
                    mode = AddressingMode::ZeroPage_X;
                    self.sta(&mode);
                }

                /* STX */
                0x8E => {
                    mode = AddressingMode::Absolute;
                    self.stx(&mode);
                }

                0x86 => {
                    mode = AddressingMode::ZeroPage;
                    self.stx(&mode);
                }
                0x96 => {
                    mode = AddressingMode::ZeroPage_Y;
                    self.stx(&mode);
                }

                /* STY */
                0x8C => {
                    mode = AddressingMode::Absolute;
                    self.sty(&mode);
                }

                0x84 => {
                    mode = AddressingMode::ZeroPage;
                    self.sty(&mode);
                }
                0x94 => {
                    mode = AddressingMode::ZeroPage_X;
                    self.sty(&mode);
                }

                /* JMP */
                0x4c => {
                    mode = AddressingMode::Absolute;
                    self.jmp(&mode);
                    continue;
                }
                0x6c => {
                    mode = AddressingMode::Indirect;
                    self.jmp(&mode);
                    continue;
                }
                /* JSR */
                0x20 => {
                    mode = AddressingMode::Absolute;
                    self.jsr(&mode);
                    continue;
                }
                /* RTs */
                0x40 => {
                    self.rts();
                    continue;
                }

                0xAA => self.tax(),
                0x8A => self.txa(),
                0xE8 => self.inx(),

                0x00 => return,
                _ => todo!("{:X?}", opscode),
            }
            self.program_counter += self.get_address_size(&mode);
        }
    }

    fn update_zero_and_negative_flags(&mut self, result: Wu8) {
        if result == Wrapping(0) {
            self.status = self.status | 0b0000_0010;
        } else {
            self.status = self.status & 0b1111_1101;
        }

        if result & Wrapping(0b1000_0000) != Wrapping(0) {
            self.status = self.status | 0b1000_0000;
        } else {
            self.status = self.status & 0b0111_1111;
        }
    }

    fn get_operand_address(&mut self, mode: &AddressingMode) -> u16 {
        match mode {
            AddressingMode::Immediate => self.program_counter,
            AddressingMode::ZeroPage => self.mem_read(self.program_counter) as u16,
            AddressingMode::Absolute => self.mem_read_u16(self.program_counter),
            AddressingMode::ZeroPage_X => {
                let pos = Wrapping(self.mem_read(self.program_counter));
                let addr = (self.register_x + pos).0 as u16;
                addr
            }
            AddressingMode::ZeroPage_Y => {
                let pos = Wrapping(self.mem_read(self.program_counter));
                let addr = (pos + self.register_y).0 as u16;
                addr
            }

            AddressingMode::Absolute_X => {
                let base = Wrapping(self.mem_read_u16(self.program_counter));
                (Wrapping(self.register_x.0 as u16) + base).0
            }
            AddressingMode::Absolute_Y => {
                let base = Wrapping(self.mem_read_u16(self.program_counter));
                (Wrapping((self.register_y).0 as u16) + base).0
            }
            AddressingMode::Indirect => {
                let base = Wrapping(self.mem_read(self.program_counter));

                let lo = self.mem_read(base.0 as u16);
                let hi = self.mem_read((base + Wrapping(1)).0 as u16);
                (hi as u16) << 8 | (lo as u16)
            }
            AddressingMode::Indirect_X => {
                let base = Wrapping(self.mem_read(self.program_counter));

                let ptr = base + self.register_x;
                let lo = self.mem_read(ptr.0 as u16);
                let hi = self.mem_read((ptr + Wrapping(1)).0 as u16);
                (hi as u16) << 8 | (lo as u16)
            }
            AddressingMode::Indirect_Y => {
                let base = self.mem_read(self.program_counter);

                let lo = self.mem_read(base as u16);
                let hi = self.mem_read((Wrapping(base) + Wrapping(1)).0 as u16);
                let deref_base = Wrapping((hi as u16) << 8 | (lo as u16));
                let deref = deref_base + Wrapping(self.register_y.0 as u16);
                deref.0
            }
            AddressingMode::NoneAddressing => {
                panic!("mode {:?} is not supported", mode);
            }
        }
    }

    fn get_address_size(&mut self, mode: &AddressingMode) -> u16 {
        match mode {
            AddressingMode::Immediate => 1,
            AddressingMode::ZeroPage => 1,
            AddressingMode::ZeroPage_X => 1,
            AddressingMode::ZeroPage_Y => 1,
            AddressingMode::Absolute => 2,
            AddressingMode::Absolute_X => 2,
            AddressingMode::Absolute_Y => 2,
            AddressingMode::Indirect => 1,
            AddressingMode::Indirect_X => 1,
            AddressingMode::Indirect_Y => 1,
            AddressingMode::NoneAddressing => 0,
        }
    }

    fn lda(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);
        self.register_a = Wrapping(value);
        self.update_zero_and_negative_flags(self.register_a);
    }
    fn ldy(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);
        self.register_y = Wrapping(value);
        self.update_zero_and_negative_flags(self.register_y);
    }
    fn ldx(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);
        self.register_x = Wrapping(value);
        self.update_zero_and_negative_flags(self.register_x);
    }
    fn tax(&mut self) {
        self.register_x = self.register_a;
        self.update_zero_and_negative_flags(self.register_x);
    }
    fn txa(&mut self) {
        self.register_a = self.register_x;
        self.update_zero_and_negative_flags(self.register_a);
    }
    fn inx(&mut self) {
        self.register_x += Wrapping(1);
        self.update_zero_and_negative_flags(self.register_x);
    }
    fn sta(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        self.mem_write(addr, self.register_a.0);
    }
    fn stx(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        self.mem_write(addr, self.register_x.0);
    }
    fn sty(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        self.mem_write(addr, self.register_y.0);
    }
    fn jmp(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        self.program_counter = addr;
    }
    fn jsr(&mut self, mode: &AddressingMode) {
        /* push the address - 1 onto the stack before transferring control
         * to the following address
         */
        self.program_counter += self.get_address_size(&mode);
        let addr = self.get_operand_address(mode);
        let save_addr = self.program_counter - 1;
        let lo = (save_addr & 0xff) as u8;
        let hi = (save_addr >> 8) as u8;
        self.stack_push(lo);
        self.stack_push(hi);
        self.program_counter = addr;
    }
    fn rts(&mut self) {
        let lo = self.stack_pop();
        let hi = self.stack_pop();
        let popped: u16 = (hi as u16) << 8 + lo as u16;
        self.program_counter = popped + 1;
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_0xa9_lda_immidiate_load_data() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0x05, 0x00]);
        assert_eq!(cpu.register_a.0, 0x05);
        assert!(cpu.status & 0b0000_0010 == 0b00);
        assert!(cpu.status & 0b1000_0000 == 0);
    }

    #[test]
    fn test_ldx_immidiate_load_data() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa0, 0x05, 0x00]);
        assert_eq!(cpu.register_y.0, 0x05);
        assert!(cpu.status & 0b0000_0010 == 0b00);
        assert!(cpu.status & 0b1000_0000 == 0);
    }

    #[test]
    fn test_ldy_immidiate_load_data() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa2, 0x05, 0x00]);
        assert_eq!(cpu.register_x.0, 0x05);
        assert!(cpu.status & 0b0000_0010 == 0b00);
        assert!(cpu.status & 0b1000_0000 == 0);
    }

    #[test]
    fn test_0xa9_lda_zero_flag() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0x00, 0x00]);
        assert!(cpu.status & 0b0000_0010 == 0b10);
        println!("{}", cpu.program_counter);
    }

    #[test]
    fn test_5_ops_working_together() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0xc0, 0xaa, 0xe8, 0x00]);
        assert_eq!(cpu.register_x.0, 0xc1)
    }

    #[test]
    fn test_combined_ld_st() {
        let mut cpu = CPU::new();

        cpu.load_and_run(vec![
            0xa0, 0x01, 0xa9, 0x03, 0x85, 0x01, 0xa9, 0x07, 0x85, 0x02, 0xa2, 0x0a, 0x8e, 0x04,
            0x07, 0xb1, 0x01, 0x00,
        ]);

        assert_eq!(cpu.register_a.0, 0x0a);
        assert_eq!(cpu.register_y.0, 0x01);
        assert_eq!(cpu.register_x.0, 0x0a);
    }

    #[test]
    fn test_0xaa_tax_move_a_to_x() {
        let mut cpu = CPU::new();
        cpu.init(vec![0xaa, 0x00]);
        cpu.register_a = Wrapping(10);
        cpu.run();

        assert_eq!(cpu.register_x, Wrapping(10))
    }

    #[test]
    fn test_txa() {
        let mut cpu = CPU::new();
        cpu.init(vec![0x8a, 0x00]);
        cpu.register_x = Wrapping(10);
        cpu.run();

        assert_eq!(cpu.register_a, Wrapping(10))
    }

    #[test]
    fn test_inx_overflow() {
        let mut cpu = CPU::new();
        cpu.init(vec![0xe8, 0xe8, 0x00]);
        cpu.register_x = Wrapping(0xff);
        cpu.run();

        assert_eq!(cpu.register_x, Wrapping(1))
    }

    #[test]
    fn test_lda_from_memory() {
        let mut cpu = CPU::new();
        cpu.mem_write(0x10, 0x55);

        cpu.load_and_run(vec![0xa5, 0x10, 0x00]);

        assert_eq!(cpu.register_a.0, 0x55);
    }

    #[test]
    fn test_lda_from_memory_x0() {
        let mut cpu = CPU::new();
        cpu.mem_write(0x10, 0x55);

        cpu.load_and_run(vec![0xa5, 0x10, 0x00]);

        assert_eq!(cpu.register_a.0, 0x55);
    }

    #[test]
    fn test_lda_from_memory_x() {
        let mut cpu = CPU::new();
        cpu.mem_write(0x19, 0x55);

        cpu.init(vec![0xb5, 0x10, 0x00]);
        cpu.register_x = Wrapping(9);
        cpu.run();

        assert_eq!(cpu.register_a.0, 0x55);
    }

    #[test]
    fn test_lda_abs() {
        let mut cpu = CPU::new();
        cpu.mem_write(0x10, 0x55);

        cpu.init(vec![0xad, 0x10, 0x00, 0x00]);
        cpu.register_x = Wrapping(9);
        cpu.run();

        assert_eq!(cpu.register_a.0, 0x55);
    }

    #[test]
    fn test_lda_abs_x() {
        let mut cpu = CPU::new();
        cpu.mem_write(0x19, 0x55);

        cpu.init(vec![0xbd, 0x10, 0x00, 0x00]);
        cpu.register_x = Wrapping(9);
        cpu.run();

        assert_eq!(cpu.register_a.0, 0x55);
    }

    #[test]
    fn test_lda_abs_y() {
        let mut cpu = CPU::new();
        cpu.mem_write(0x19, 0x55);

        cpu.init(vec![0xb9, 0x10, 0x00, 0x00]);
        cpu.register_y = Wrapping(9);
        cpu.run();

        assert_eq!(cpu.register_a.0, 0x55);
    }

    #[test]
    fn test_lda_ind_x() {
        let mut cpu = CPU::new();
        cpu.mem_write(0x0A, 0x32);
        cpu.mem_write(0x32, 0xFF);

        cpu.init(vec![0xa1, 0x01, 0x00]);
        cpu.register_x = Wrapping(9);
        cpu.run();

        assert_eq!(cpu.register_a.0, 0xFF);
    }

    #[test]
    fn test_lda_ind_y0() {
        let mut cpu = CPU::new();
        cpu.mem_write(0x00, 0x32);
        cpu.mem_write(0x32, 0xFE);

        cpu.init(vec![0xb1, 0x00, 0x00]);
        cpu.run();
        println!("{:?}", &cpu.memory[..=63]);

        assert_eq!(cpu.register_a.0, 0xFE);
    }

    #[test]
    fn test_lda_ind_y() {
        let mut cpu = CPU::new();
        cpu.mem_write(0x01, 0x03);
        cpu.mem_write(0x02, 0x07);
        cpu.mem_write(0x0704, 0x0a);

        cpu.init(vec![0xb1, 0x01, 0x00]);
        cpu.register_y = Wrapping(0x01);
        cpu.run();
        assert_eq!(cpu.register_a.0, 0x0a);
    }

    #[test]
    fn test_sta_zp() {
        let mut cpu = CPU::new();

        cpu.init(vec![0x85, 0x01, 0x00]);
        cpu.register_a = Wrapping(0xff);
        cpu.run();
        assert_eq!(cpu.mem_read(0x01), 0xff);
    }

    #[test]
    fn test_sta_zp_x() {
        let mut cpu = CPU::new();

        cpu.init(vec![0x95, 0x01, 0x00]);
        cpu.register_a = Wrapping(0xff);
        cpu.register_x = Wrapping(0x01);
        cpu.run();
        assert_eq!(cpu.mem_read(0x02), 0xff);
    }

    #[test]
    fn test_stx_abs() {
        // TODO: this tests technically tests absolute, but we should try with
        // two bytes
        let mut cpu = CPU::new();

        cpu.init(vec![0x8e, 0x01, 0x00]);
        cpu.register_x = Wrapping(0xff);
        cpu.run();
        assert_eq!(cpu.mem_read(0x01), 0xff);
    }

    #[test]
    fn test_stx_zp() {
        let mut cpu = CPU::new();

        cpu.init(vec![0x86, 0x01, 0x00]);
        cpu.register_x = Wrapping(0xff);
        cpu.run();
        assert_eq!(cpu.mem_read(0x01), 0xff);
    }

    #[test]
    fn test_stx_zp_y() {
        let mut cpu = CPU::new();

        cpu.init(vec![0x96, 0x01, 0x00]);
        cpu.register_x = Wrapping(0xff);
        cpu.register_y = Wrapping(0x01);
        cpu.run();
        assert_eq!(cpu.mem_read(0x02), 0xff);
    }
    /* STY */
    #[test]
    fn test_sty_abs() {
        // TODO: this tests technically tests absolute, but we should try with
        // two bytes
        let mut cpu = CPU::new();

        cpu.init(vec![0x8c, 0x01, 0x00]);
        cpu.register_y = Wrapping(0xff);
        cpu.run();
        assert_eq!(cpu.mem_read(0x01), 0xff);
    }

    #[test]
    fn test_sty_zp() {
        let mut cpu = CPU::new();

        cpu.init(vec![0x84, 0x01, 0x00]);
        cpu.register_y = Wrapping(0xff);
        cpu.run();
        assert_eq!(cpu.mem_read(0x01), 0xff);
    }

    #[test]
    fn test_sty_zp_x() {
        let mut cpu = CPU::new();

        cpu.init(vec![0x94, 0x01, 0x00]);
        cpu.register_y = Wrapping(0xff);
        cpu.register_x = Wrapping(0x01);
        cpu.run();
        assert_eq!(cpu.mem_read(0x02), 0xff);
    }

    #[test]
    fn test_jmp_abs() {
        let mut cpu = CPU::new();
        cpu.init(vec![0x4c, 0x01, 0x00, 0x00]);
        cpu.run();
        assert_eq!(cpu.program_counter, 0x02); // pc increments for brk
    }

    #[test]
    fn test_jmp_indirect() {
        let mut cpu = CPU::new();
        cpu.init(vec![0x6c, 0x01, 0x00, 0x00]);
        cpu.mem_write(0x01, 0x32);
        cpu.run();
        assert_eq!(cpu.program_counter, 0x33); // pc increments for brk
    }

    #[test]
    fn test_game() {
        let mut cpu = CPU::new();
        let game_code = vec![
            0x20, 0x06, 0x06, 0x20, 0x38, 0x06, 0x20, 0x0d, 0x06, 0x20, 0x2a, 0x06, 0x60, 0xa9,
            0x02, 0x85, 0x02, 0xa9, 0x04, 0x85, 0x03, 0xa9, 0x11, 0x85, 0x10, 0xa9, 0x10, 0x85,
            0x12, 0xa9, 0x0f, 0x85, 0x14, 0xa9, 0x04, 0x85, 0x11, 0x85, 0x13, 0x85, 0x15, 0x60,
            0xa5, 0xfe, 0x85, 0x00, 0xa5, 0xfe, 0x29, 0x03, 0x18, 0x69, 0x02, 0x85, 0x01, 0x60,
            0x20, 0x4d, 0x06, 0x20, 0x8d, 0x06, 0x20, 0xc3, 0x06, 0x20, 0x19, 0x07, 0x20, 0x20,
            0x07, 0x20, 0x2d, 0x07, 0x4c, 0x38, 0x06, 0xa5, 0xff, 0xc9, 0x77, 0xf0, 0x0d, 0xc9,
            0x64, 0xf0, 0x14, 0xc9, 0x73, 0xf0, 0x1b, 0xc9, 0x61, 0xf0, 0x22, 0x60, 0xa9, 0x04,
            0x24, 0x02, 0xd0, 0x26, 0xa9, 0x01, 0x85, 0x02, 0x60, 0xa9, 0x08, 0x24, 0x02, 0xd0,
            0x1b, 0xa9, 0x02, 0x85, 0x02, 0x60, 0xa9, 0x01, 0x24, 0x02, 0xd0, 0x10, 0xa9, 0x04,
            0x85, 0x02, 0x60, 0xa9, 0x02, 0x24, 0x02, 0xd0, 0x05, 0xa9, 0x08, 0x85, 0x02, 0x60,
            0x60, 0x20, 0x94, 0x06, 0x20, 0xa8, 0x06, 0x60, 0xa5, 0x00, 0xc5, 0x10, 0xd0, 0x0d,
            0xa5, 0x01, 0xc5, 0x11, 0xd0, 0x07, 0xe6, 0x03, 0xe6, 0x03, 0x20, 0x2a, 0x06, 0x60,
            0xa2, 0x02, 0xb5, 0x10, 0xc5, 0x10, 0xd0, 0x06, 0xb5, 0x11, 0xc5, 0x11, 0xf0, 0x09,
            0xe8, 0xe8, 0xe4, 0x03, 0xf0, 0x06, 0x4c, 0xaa, 0x06, 0x4c, 0x35, 0x07, 0x60, 0xa6,
            0x03, 0xca, 0x8a, 0xb5, 0x10, 0x95, 0x12, 0xca, 0x10, 0xf9, 0xa5, 0x02, 0x4a, 0xb0,
            0x09, 0x4a, 0xb0, 0x19, 0x4a, 0xb0, 0x1f, 0x4a, 0xb0, 0x2f, 0xa5, 0x10, 0x38, 0xe9,
            0x20, 0x85, 0x10, 0x90, 0x01, 0x60, 0xc6, 0x11, 0xa9, 0x01, 0xc5, 0x11, 0xf0, 0x28,
            0x60, 0xe6, 0x10, 0xa9, 0x1f, 0x24, 0x10, 0xf0, 0x1f, 0x60, 0xa5, 0x10, 0x18, 0x69,
            0x20, 0x85, 0x10, 0xb0, 0x01, 0x60, 0xe6, 0x11, 0xa9, 0x06, 0xc5, 0x11, 0xf0, 0x0c,
            0x60, 0xc6, 0x10, 0xa5, 0x10, 0x29, 0x1f, 0xc9, 0x1f, 0xf0, 0x01, 0x60, 0x4c, 0x35,
            0x07, 0xa0, 0x00, 0xa5, 0xfe, 0x91, 0x00, 0x60, 0xa6, 0x03, 0xa9, 0x00, 0x81, 0x10,
            0xa2, 0x00, 0xa9, 0x01, 0x81, 0x10, 0x60, 0xa2, 0x00, 0xea, 0xea, 0xca, 0xd0, 0xfb,
            0x60,
        ];

        cpu.init(game_code);
        cpu.run();
    }
}

fn main() {
    println!("hi");
}
