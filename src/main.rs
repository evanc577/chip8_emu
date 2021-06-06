use crate::cpu::{CPUState, CycleInput, CPU};
use drivers::{AudioDriver, DisplayDriver, InputDriver};

mod cpu;
mod drivers;
mod config;

// 60Hz timers
const TIMER_DURATION: std::time::Duration = std::time::Duration::from_nanos(16_666_666);

fn main() {
    let config = config::get_config();

    // set tick rate
    let secs_per_instr: f64 = 1.0 / config.rate as f64;
    let ns_per_instr: u64 = (secs_per_instr * 1_000_000_000.0) as u64;
    let target_sleep_duration: std::time::Duration = std::time::Duration::from_nanos(ns_per_instr);

    // Load ROM file
    let rom = std::fs::read(config.rom_file).unwrap();

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
    let mut time_start = std::time::Instant::now();
    while let Ok(keys) = input.poll() {
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
        }
        if output.beep {
            audio.start_beep();
        } else {
            audio.stop_beep();
        }

        // sleep remaining duration
        let time_end = std::time::Instant::now();
        let time_elapsed = time_end - time_start;
        std::thread::sleep(
            target_sleep_duration
                .checked_sub(time_elapsed)
                .unwrap_or(std::time::Duration::from_nanos(0)),
        );
        time_start = time_end;
    }
}
