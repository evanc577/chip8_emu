use crate::cpu::{CPUState, CycleInput, CPU};
use drivers::{AudioDriver, DisplayDriver, InputDriver};
use std::io::{stderr, Write};

mod config;
mod cpu;
mod drivers;

// 60Hz timers
const TIMER_DURATION: std::time::Duration = std::time::Duration::from_nanos(16_666_666);

fn main() {
    // Read configuration from command line
    let config = config::get_config();

    // set tick rate
    let secs_per_instr: f64 = 1.0 / config.rate as f64;
    let ns_per_instr: u64 = (secs_per_instr * 1_000_000_000f64) as u64;
    let target_sleep_duration: std::time::Duration = std::time::Duration::from_nanos(ns_per_instr);

    // Load ROM file
    let rom = match std::fs::read(&config.rom_file) {
        Ok(v) => v,
        Err(e) => {
            writeln!(
                &mut stderr(),
                "{:?}: {}",
                config.rom_file,
                e
            )
            .ok();
            std::process::exit(1);
        }
    };

    // Initialize emulated CPU
    let mut cpu = CPU::new(&rom[..]);

    // Initialize drivers
    let sdl_context = sdl2::init().unwrap();
    let mut display_driver = DisplayDriver::new(&sdl_context);
    let mut input_driver = InputDriver::new(&sdl_context);
    let audio_driver = AudioDriver::new(&sdl_context);

    // Initialize periodic timer
    let ticker = crossbeam::channel::tick(TIMER_DURATION);

    // Main loop
    let mut time_start = std::time::Instant::now();
    while let Ok(keys) = input_driver.poll() {
        // Generate inputs
        let input = CycleInput {
            keys,
            decrement_timer: ticker.try_recv().is_ok(),
        };

        // Run 1 CPU cycle
        let output = cpu.cycle(&input);

        // Process outputs
        if let CPUState::RunningDraw = output.state {
            display_driver.draw(output.gfx)
        }
        audio_driver.beep(output.beep);

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
