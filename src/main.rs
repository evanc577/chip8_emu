use crate::cpu::{CPUState, CPU};
use std::env;

mod cpu;
mod display;
mod input;

const CYCLES_PER_SEC: usize = 700;

fn main() {
    let args: Vec<_> = env::args_os().collect();
    let rom = std::fs::read(&args[1]).unwrap();

    let mut cpu = CPU::new(&rom[..]);

    let sdl_context = sdl2::init().unwrap();
    let mut display = display::DisplayWindow::new(&sdl_context);
    let mut input = input::Input::new(&sdl_context);

    while let Ok(_) = input.poll() {
        let output = cpu.cycle();
        match output.state {
            CPUState::Running => (),
            CPUState::RunningDraw => display.draw(output.gfx),
            CPUState::Halt => {
                eprintln!("CPU Halt");
                std::thread::sleep(std::time::Duration::from_secs(10));
                break;
            }
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
}
