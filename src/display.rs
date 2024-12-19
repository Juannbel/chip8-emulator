use sdl2::render::WindowCanvas;

pub struct Display {
    canvas: WindowCanvas,
    pixels: [u64; Display::DISPLAY_HEIGHT],
}

impl Display {
    const DISPLAY_WIDTH: usize = 64;
    const DISPLAY_HEIGHT: usize = 32;
    const FOREGROUND_COLOR: (u8, u8, u8) = (60, 163, 214);
    const BACKGROUND_COLOR: (u8, u8, u8) = (0, 0, 0);

    pub fn new(canvas: WindowCanvas) -> Display {
        Display {
            canvas,
            pixels: [0; Display::DISPLAY_HEIGHT],
        }
    }

    pub fn clear(&mut self) {
        for i in 0..self.pixels.len() {
            self.pixels[i] = 0;
        }
    }

    pub fn render(&mut self) {
        self.canvas.set_draw_color(Display::BACKGROUND_COLOR);
        self.canvas.clear();

        let window_width = self.canvas.window().size().0 as usize;
        let window_height = self.canvas.window().size().1 as usize;
        let block = std::cmp::min(
            window_width / Display::DISPLAY_WIDTH,
            window_height / Display::DISPLAY_HEIGHT,
        );
        let start_x = (window_width - Display::DISPLAY_WIDTH * block) / 2;
        let start_y = (window_height - Display::DISPLAY_HEIGHT * block) / 2;

        self.canvas.set_draw_color(Display::FOREGROUND_COLOR);
        for y in 0..Display::DISPLAY_HEIGHT {
            for x in 0..Display::DISPLAY_WIDTH {
                if self.is_pixel_on(x, y) {
                    self.canvas
                        .fill_rect(sdl2::rect::Rect::new(
                            (start_x + x * block) as i32,
                            (start_y + y * block) as i32,
                            block as u32,
                            block as u32,
                        ))
                        .unwrap();
                }
            }
        }

        self.canvas
            .draw_rect(sdl2::rect::Rect::new(
                start_x as i32 - 1,
                start_y as i32 - 1,
                (Display::DISPLAY_WIDTH * block + 2) as u32,
                (Display::DISPLAY_HEIGHT * block + 2) as u32,
            ))
            .unwrap();

        self.canvas.present();
    }

    pub fn draw(&mut self, x: u8, y: u8, sprite: u8) -> bool {
        let mut collision = false;
        for i in 0..8 {
            if sprite.checked_shr(7 - i).unwrap_or(0) & 0x1 == 0x1 {
                let dx = (x as usize + i as usize) % Display::DISPLAY_WIDTH;
                let dy = y as usize % Display::DISPLAY_HEIGHT;
                if self.is_pixel_on(dx, dy) {
                    collision = true;
                }
                self.pixels[dy] ^= 0x1_u64.checked_shl((63 - dx) as u32).unwrap_or(0);
            }
        }
        collision
    }

    fn is_pixel_on(&self, x: usize, y: usize) -> bool {
        self.pixels[y].checked_shr((63 - x) as u32).unwrap_or(0) & 1 == 1
    }
}
