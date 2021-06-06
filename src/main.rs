use crate::cpu::{CPUState, CPU, CycleInput};
use drivers::{DisplayDriver, AudioDriver, InputDriver};
use std::env;

mod cpu;
mod drivers;

const INSTR_PER_SEC: u64 = 10000;
const SECS_PER_INSTR: f64 = 1.0 / INSTR_PER_SEC as f64;
const NS_PER_INSTR: u64 = (SECS_PER_INSTR * 1_000_000_000.0) as u64;
const TARGET_SLEEP_DURATION: std::time::Duration = std::time::Duration::from_nanos(NS_PER_INSTR);
const TIMER_DURATION: std::time::Duration = std::time::Duration::from_nanos(16_666_666);

fn main() {
    // Load ROM file
    let args: Vec<_> = env::args_os().collect();
    let rom = std::fs::read(&args[1]).unwrap();

    // Initialize CPU
    let mut cpu = CPU::new(&rom[..]);

    // Initialize drivers
    let sdl_context = sdl2::init().unwrap();
    let mut display = DisplayDriver::new(&sdl_context);
    let mut input = InputDriver::new(&sdl_context);
    let audio = AudioDriver::new(&&sdl_context);

    // Initialize periodic timer
    let ticker = crossbeam::channel::tick(TIMER_DURATION);

    // Main loop
    let mut halted = false;
    let mut time_start = std::time::Instant::now();
    while let Ok(keys) = input.poll() {
        if !halted {
            let input = CycleInput {
                keys,
                decrement_timer: match ticker.try_recv() {
                    Ok(_) => true,
                    Err(_) => false,
                },
            };
            let output = cpu.cycle(&input);
            match output.state {
                CPUState::Running => (),
                CPUState::RunningDraw => display.draw(output.gfx),
                CPUState::Halt => {
                    eprintln!("CPU Halt");
                    halted = true;
                }
            }
            if output.beep {
                audio.start_beep();
            } else {
                audio.stop_beep();
            }
        }

        // sleep remaining duration
        let time_end = std::time::Instant::now();
        let time_elapsed = time_end - time_start;
        std::thread::sleep(
            TARGET_SLEEP_DURATION
                .checked_sub(time_elapsed)
                .unwrap_or(std::time::Duration::from_nanos(0)),
        );
        time_start = time_end;
    }
}
