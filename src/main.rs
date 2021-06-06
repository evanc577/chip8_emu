use crate::cpu::{CPUState, CPU};
use std::env;

mod cpu;
mod display;
mod input;

const INSTR_PER_SEC: u64 = 700;
const SECS_PER_INSTR: f64 = 1.0 / INSTR_PER_SEC as f64;
const NS_PER_INSTR: u64 = (SECS_PER_INSTR * 1_000_000_000.0) as u64;

fn main() {
    let args: Vec<_> = env::args_os().collect();
    let rom = std::fs::read(&args[1]).unwrap();

    let mut cpu = CPU::new(&rom[..]);

    let sdl_context = sdl2::init().unwrap();
    let mut display = display::DisplayWindow::new(&sdl_context);
    let mut input = input::Input::new(&sdl_context);

    let mut halted = false;
    while let Ok(_) = input.poll() {
        if !halted {
            let output = cpu.cycle();
            match output.state {
                CPUState::Running => (),
                CPUState::RunningDraw => display.draw(output.gfx),
                CPUState::Halt => {
                    eprintln!("CPU Halt");
                    halted = true;
                }
            }
        }
        std::thread::sleep(std::time::Duration::from_nanos(NS_PER_INSTR));
    }
}
