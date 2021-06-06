use sdl2::event::Event;

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
            if let Event::Quit { .. } = event {
                return Err(());
            };
        }

        Ok(())
    }
}
