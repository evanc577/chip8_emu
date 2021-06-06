use sdl2::event::Event;
use sdl2::keyboard::Keycode;

pub struct Input {
    events: sdl2::EventPump,
}

impl Input {
    pub fn new(context: &sdl2::Sdl) -> Self {
        Self {
            events: context.event_pump().unwrap(),
        }
    }

    pub fn poll(&mut self) -> Result<(), ()> {
        for event in self.events.poll_iter() {
            match event {
                Event::Quit { .. } => return Err(()),
                Event::KeyDown {
                    keycode: Some(key), ..
                } => match key {
                    Keycode::Q | Keycode::Escape => return Err(()),
                    _ => (),
                },
                _ => (),
            }
        }

        Ok(())
    }
}
