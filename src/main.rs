use std::num::Wrapping;

type Wu8 = Wrapping<u8>;

#[derive(Debug)]
#[allow(non_camel_case_types)]
pub enum AddressingMode {
    Immediate,
    ZeroPage,
    ZeroPage_X,
    Absolute,
    Absolute_X,
    Absolute_Y,
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

                /* STA */
                0x85 => {
                    mode = AddressingMode::ZeroPage;
                    self.sta(&mode);
                }

                0x95 => {
                    mode = AddressingMode::ZeroPage_X;
                    self.sta(&mode);
                }

                0xAA => self.tax(),
                0xE8 => self.inx(),

                0x00 => return,
                _ => todo!("{}", opscode),
            }
            self.program_counter += self.get_address_size(&mode);
        }
    }

    fn lda(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);
        self.register_a = Wrapping(value);
        self.update_zero_and_negative_flags(self.register_a);
    }
    fn tax(&mut self) {
        self.register_x = self.register_a;
        self.update_zero_and_negative_flags(self.register_x);
    }
    fn inx(&mut self) {
        self.register_x += Wrapping(1);
        self.update_zero_and_negative_flags(self.register_x);
    }
    fn sta(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        self.mem_write(addr, self.register_a.0);
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
            AddressingMode::Absolute_X => {
                let base = Wrapping(self.mem_read_u16(self.program_counter));
                (Wrapping(self.register_x.0 as u16) + base).0
            }
            AddressingMode::Absolute_Y => {
                let base = Wrapping(self.mem_read_u16(self.program_counter));
                (Wrapping((self.register_y).0 as u16) + base).0
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
            AddressingMode::Absolute => 2,
            AddressingMode::Absolute_X => 2,
            AddressingMode::Absolute_Y => 2,
            AddressingMode::Indirect_X => 1,
            AddressingMode::Indirect_Y => 1,
            AddressingMode::NoneAddressing => 0,
        }
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
    fn test_0xaa_tax_move_a_to_x() {
        let mut cpu = CPU::new();
        cpu.init(vec![0xaa, 0x00]);
        cpu.register_a = Wrapping(10);
        cpu.run();

        assert_eq!(cpu.register_x, Wrapping(10))
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
}

fn main() {
    println!("hi");
}
