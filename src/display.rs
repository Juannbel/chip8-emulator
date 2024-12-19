use sdl2::render::WindowCanvas;

const DISPLAY_WIDTH: usize = 64;
const DISPLAY_HEIGHT: usize = 32;
const FOREGROUND_COLOR: (u8, u8, u8) = (60, 163, 214);
const BACKGROUND_COLOR: (u8, u8, u8) = (0, 0, 0);

pub struct Display<'a> {
    canvas: &'a mut WindowCanvas,
    pixels: [[bool; DISPLAY_WIDTH]; DISPLAY_HEIGHT],
}

impl Display<'_> {
    pub fn new(canvas: &mut WindowCanvas) -> Display {
        Display {
            canvas,
            pixels: [[false; DISPLAY_WIDTH]; DISPLAY_HEIGHT],
        }
    }

    pub fn clear(&mut self) {
        for i in 0..self.pixels.len() {
            for j in 0..self.pixels[0].len() {
                self.pixels[i][j] = false;
            }
        }
    }

    pub fn render(&mut self) {
        self.canvas.set_draw_color(BACKGROUND_COLOR);
        self.canvas.clear();

        let window_width = self.canvas.window().size().0 as usize;
        let window_height = self.canvas.window().size().1 as usize;
        let block = std::cmp::min(window_width / DISPLAY_WIDTH, window_height / DISPLAY_HEIGHT);
        let start_x = (window_width - DISPLAY_WIDTH * block) / 2;
        let start_y = (window_height - DISPLAY_HEIGHT * block) / 2;

        self.canvas.set_draw_color(FOREGROUND_COLOR);
        for y in 0..DISPLAY_HEIGHT {
            for x in 0..DISPLAY_WIDTH {
                if self.pixels[y][x] {
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
                (DISPLAY_WIDTH * block + 2) as u32,
                (DISPLAY_HEIGHT * block + 2) as u32,
            ))
            .unwrap();

        self.canvas.present();
    }

    pub fn draw(&mut self, x: u8, y: u8, sprite: u8) -> bool {
        let mut collision = false;
        for i in 0..8 {
            if sprite.checked_shr(7 - i).unwrap_or(0) & 0x1 == 0x1 {
                let dx = (x as usize + i as usize) % DISPLAY_WIDTH;
                let dy = y as usize % DISPLAY_HEIGHT;
                if self.pixels[dy][dx] {
                    collision = true;
                    self.pixels[dy][dx] = false;
                } else {
                    self.pixels[dy][dx] = true;
                }
            }
        }
        collision
    }
}
