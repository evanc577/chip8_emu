use crate::cpu::{CPUState, CycleInput, CPU};
use drivers::{AudioDriver, DisplayDriver, InputDriver};
use std::io::{stderr, Write};

mod config;
mod cpu;
mod drivers;

// 60Hz Chip-8 timers
const CH8_TIMER_DURATION: std::time::Duration = std::time::Duration::from_nanos(16_666_666);

// Limit window refresh rate to 100Hz
const DRAW_TIMER_DURATION: std::time::Duration = std::time::Duration::from_millis(10);

// Performance monitoring timers
const PERF_TIMER_DURATION: std::time::Duration = std::time::Duration::from_secs(1);

fn main() {
    // Read configuration from command line
    let config = config::get_config();

    // set tick rate
    let target_sleep_duration = std::time::Duration::from_nanos(match config.rate {
        Some(rate) => {
            let secs_per_instr: f64 = 1.0 / rate as f64;
            (secs_per_instr * 1_000_000_000f64) as u64
        }
        None => 0,
    });

    // Load ROM file
    let rom = match std::fs::read(&config.rom_file) {
        Ok(v) => v,
        Err(e) => {
            writeln!(&mut stderr(), "{:?}: {}", config.rom_file, e).ok();
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

    // Initialize periodic timers
    let ch8_ticker = crossbeam::channel::tick(CH8_TIMER_DURATION);
    let perf_ticker = crossbeam::channel::tick(PERF_TIMER_DURATION);
    let draw_ticker = crossbeam::channel::tick(DRAW_TIMER_DURATION);

    // Main loop
    let mut perf_counter: usize = 0;
    let mut draw_queued = false;
    while let Ok(keys) = input_driver.poll() {
        let time_start = std::time::Instant::now();

        // Generate inputs
        let input = CycleInput {
            keys,
            decrement_timer: ch8_ticker.try_recv().is_ok(),
        };

        // Run 1 CPU cycle
        let output = cpu.cycle(&input);

        // Performance monitoring
        perf_counter += 1;
        let perf = match perf_ticker.try_recv() {
            Ok(_) => {
                let temp = perf_counter;
                perf_counter = 0;
                Some(temp)
            },
            Err(_) => None,
        };

        // Process outputs
        let mut draw = || display_driver.draw(output.gfx, perf);
        let draw_tick = draw_ticker.try_recv().is_ok();
        if CPUState::RunningDraw == output.state {
            if draw_tick {
                draw();
                draw_queued = false;
            } else {
                draw_queued = true;
            }
        } else if draw_tick && draw_queued || perf.is_some() {
            draw();
            draw_queued = false;
        }

        audio_driver.beep(output.beep);

        // sleep remaining duration
        let time_end = std::time::Instant::now();
        let time_elapsed = time_end - time_start;
        let sleep_duration = target_sleep_duration
            .checked_sub(time_elapsed)
            .unwrap_or(std::time::Duration::from_nanos(0));
        spin_sleep::sleep(sleep_duration);
    }
}
