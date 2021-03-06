use crate::chip::Op;
use rand::prelude::*;

#[derive(Clone)]
pub struct Cpu {
    pub pc: i16,
    pub ram: [u8; 4096],
    pub display: [[bool; 32]; 64],
    pub v: [u8; 16],
    i: u16,
    stack: Vec<u16>,
    pub delay_timer: u8,
    pub sound_timer: u8,
    pub is_waiting_for_input: bool,
    pub save_into_this_vx: u8,
}

impl Cpu {
    pub fn new(contents: Vec<u8>) -> Cpu {
        Cpu {
            pc: 512,
            ram: Cpu::init_ram(contents),
            display: [[false; 32]; 64],
            stack: Vec::new(),
            v: [0; 16],
            i: 0,
            delay_timer: 0,
            sound_timer: 0,
            is_waiting_for_input: false,
            save_into_this_vx: 0,
        }
    }

    fn init_ram(items: Vec<u8>) -> [u8; 4096] {
        let mut ram = [0; 4096];
        let sprites = Cpu::get_sprites();
        for (i, sprite) in sprites.iter().enumerate() {
            ram[i] = *sprite;
        }
        for (i, el) in items.iter().enumerate() {
            ram[512 + i] = *el;
        }
        return ram;
    }
    pub fn execute_op(&mut self, op: Op, pressed_keys: &[u8; 16]) {
        self.pc += 2;
        match op.op {
            0x0 => {
                if op.nn == 0xE0 {
                    self.display.fill([false; 32]);
                } else if op.nn == 0xEE {
                    self.pc = self.stack.pop().expect("Should never pop too far") as i16;
                } else {
                    println!("{:#04X?}", op.nn);
                    println!("{}", op.nn);
                    Cpu::log_not_implemented(op);
                }
            }
            1 => self.pc = op.nnn as i16,
            2 => {
                self.stack.push(self.pc as u16);
                self.pc = op.nnn as i16;
            }
            3 => {
                if self.v[op.x as usize] == op.nn {
                    self.pc += 2;
                }
            }
            4 => {
                if self.v[op.x as usize] != op.nn {
                    self.pc += 2;
                }
            }
            5 => {
                if self.v[op.x as usize] == self.v[op.y as usize] {
                    self.pc += 2;
                }
            }
            6 => self.v[op.x as usize] = op.nn,
            7 => self.v[op.x as usize] = self.v[op.x as usize].wrapping_add(op.nn),
            8 => match op.n {
                0 => self.v[op.x as usize] = self.v[op.y as usize],
                1 => self.v[op.x as usize] = (self.v[op.x as usize] | self.v[op.y as usize]),
                2 => self.v[op.x as usize] = (self.v[op.x as usize] & self.v[op.y as usize]),
                3 => self.v[op.x as usize] = (self.v[op.x as usize] ^ self.v[op.y as usize]),
                4 => {
                    let (result, did_overflow) =
                        self.v[op.x as usize].overflowing_add((self.v[op.y as usize]));
                    self.v[0xF] = 0;
                    if did_overflow {
                        self.v[0xF] = 1;
                    }
                    self.v[op.x as usize] = result;
                }
                5 => {
                    self.v[0xF] = 0;
                    if (self.v[op.x as usize] > self.v[op.y as usize]) {
                        self.v[0xF] = 1;
                    }
                    self.v[op.x as usize] =
                        self.v[op.x as usize].wrapping_sub(self.v[op.y as usize]);
                }
                6 => {
                    self.v[0xF] = self.v[op.x as usize] & 0x1;
                    self.v[op.x as usize] >>= 1;
                }
                7 => {
                    self.v[0xF] = 0;
                    if (self.v[op.y as usize] > self.v[op.x as usize]) {
                        self.v[0xF] = 1;
                    }
                    self.v[op.x as usize] =
                        self.v[op.y as usize].wrapping_sub(self.v[op.x as usize]);
                }
                0xE => {
                    self.v[0xF] = (self.v[op.x as usize] >> 7) & 0x01;
                    self.v[op.x as usize] = self.v[op.x as usize] << 1;
                }
                _ => Cpu::log_not_implemented(op),
            },
            9 => {
                if self.v[op.x as usize] != self.v[op.y as usize] {
                    self.pc += 2
                }
            }
            0xA => {
                self.i = op.nnn;
            }
            0xB => self.pc = (op.nnn + self.v[0x0] as u16) as i16,
            0xC => self.v[op.x as usize] = rand::thread_rng().gen_range(0..=255) & op.n,
            0xD => {
                let x_val = self.v[op.x as usize];
                let y_val = self.v[op.y as usize];

                let x_coord = x_val & 63;
                let y_coord = y_val & 31;
                self.v[0xF as usize] = 0;

                let mut index = 0;
                for row in 0..op.n {
                    let y = y_coord + row;
                    if y >= 32 {
                        break;
                    }

                    let sprite = self.ram[(self.i + index) as usize];

                    for col in 0..8 {
                        let x = x_coord + col;
                        if x >= 64 {
                            break;
                        }

                        let old_pixel = self.display[x as usize][y as usize];
                        let current_pos = 7 - col;
                        let to_shift = 1 << current_pos;

                        // check the current bit is on or not
                        let new_pixel = (sprite & to_shift) != 0x0;
                        if old_pixel && new_pixel {
                            self.display[x as usize][y as usize] = false;
                            self.v[0xF] = 1;
                        } else if new_pixel && !old_pixel {
                            self.display[x as usize][y as usize] = true;
                        }
                    }
                    index += 1;
                }
            }
            0xE => match op.nn {
                0x9E => {
                    if pressed_keys[self.v[op.x as usize] as usize] == 0x1 {
                        self.pc += 2;
                    }
                }
                0xA1 => {
                    if pressed_keys[self.v[op.x as usize] as usize] != 0x1 {
                        self.pc += 2;
                    }
                }
                _ => Cpu::log_not_implemented(op),
            },
            0xF => match op.nn {
                0x07 => self.v[op.x as usize] = self.delay_timer,
                0xA => {
                    self.is_waiting_for_input = true;
                    self.save_into_this_vx = op.x;
                }
                0x15 => self.delay_timer = self.v[op.x as usize],
                0x1E => self.i = self.i.wrapping_add(self.v[op.x as usize] as u16),
                0x18 => self.sound_timer = self.v[op.x as usize],
                0x29 => self.i = (self.v[(op.x) as usize] * 5) as u16,
                0x55 => {
                    for i in 0..=op.x {
                        self.ram[(self.i + i as u16) as usize] = self.v[i as usize];
                    }
                }
                0x33 => {
                    let mut temp = self.v[op.x as usize];
                    self.ram[(self.i + 2) as usize] = temp % 10;
                    temp /= 10;
                    self.ram[(self.i + 1) as usize] = temp % 10;
                    temp /= 10;
                    self.ram[self.i as usize] = temp;
                }
                0x65 => {
                    for i in 0..=op.x {
                        self.v[i as usize] = self.ram[(self.i + i as u16) as usize];
                    }
                }
                _ => {
                    Cpu::log_not_implemented(op);
                }
            },
            _ => {
                Cpu::log_not_implemented(op);
            }
        }
    }

    pub fn log_not_implemented(op: Op) {
        println!("op: {:#04X?}", op.op);
        println!("nn: {:#04X?}", op.nn);
        panic!("No match, you screwed up implementation")
    }

    fn pretty_print(&self) {
        for item in self.ram {
            println!("{:#04X?}", item);
        }
    }
    fn get_sprites() -> [u8; 80] {
        let sprites: [u8; 80] = [
            0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
            0x20, 0x60, 0x20, 0x20, 0x70, // 1
            0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
            0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
            0x90, 0x90, 0xF0, 0x10, 0x10, // 4
            0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
            0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
            0xF0, 0x10, 0x20, 0x40, 0x40, // 7
            0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
            0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
            0xF0, 0x90, 0xF0, 0x90, 0x90, // A
            0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
            0xF0, 0x80, 0x80, 0x80, 0xF0, // C
            0xE0, 0x90, 0x90, 0x90, 0xE0, // D
            0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
            0xF0, 0x80, 0xF0, 0x80, 0x80, // F
        ];
        return sprites;
    }
}
