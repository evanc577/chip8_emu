use crate::cpu::{PixelState, DISPLAY_H, DISPLAY_W};
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;

const PIXEL_SIZE: usize = 20;

pub struct DisplayDriver {
    canvas: Canvas<Window>,
}

impl DisplayDriver {
    pub fn new(context: &sdl2::Sdl) -> Self {
        sdl2::hint::set("SDL_VIDEO_X11_NET_WM_BYPASS_COMPOSITOR", "0");

        let video_subsystem = context.video().unwrap();
        let window = video_subsystem
            .window(
                "CHIP-8",
                (PIXEL_SIZE * DISPLAY_W) as u32,
                (PIXEL_SIZE * DISPLAY_H) as u32,
            )
            .position_centered()
            .vulkan()
            .build()
            .unwrap();

        let mut canvas = window
            .into_canvas()
            .accelerated()
            .present_vsync()
            .build()
            .unwrap();

        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();
        canvas.present();

        Self { canvas }
    }

    pub fn draw(&mut self, gfx: &[PixelState]) {
        for y in 0..DISPLAY_H {
            for x in 0..DISPLAY_W {
                let offset = y * DISPLAY_W + x;
                let color = match gfx[offset] {
                    PixelState::On => Color::RGB(255, 255, 255),
                    PixelState::Off => Color::RGB(0, 0, 0),
                };

                let pix_x: i32 = (x * PIXEL_SIZE) as i32;
                let pix_y: i32 = (y * PIXEL_SIZE) as i32;
                self.canvas.set_draw_color(color);
                let _ = self.canvas.fill_rect(Rect::new(
                    pix_x,
                    pix_y,
                    PIXEL_SIZE as u32,
                    PIXEL_SIZE as u32,
                ));
            }
        }

        self.canvas.present();
    }
}
