use std::io::Stdout;

use tui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{
        canvas::{Canvas, Points},
        Block, Borders,
    },
    Terminal,
};

const DISPLAY_WIDTH: usize = 64;
const DISPLAY_HEIGHT: usize = 32;

pub struct Display<'a> {
    terminal: &'a mut Terminal<CrosstermBackend<Stdout>>,
    pixels: [[bool; DISPLAY_WIDTH]; DISPLAY_HEIGHT],
}

impl Display<'_> {
    pub fn new(terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> Display {
        Display {
            terminal,
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
        self.terminal
            .draw(|f| {
                let chunks = Layout::default()
                    .constraints([Constraint::Length(DISPLAY_HEIGHT as u16)])
                    .split(f.size());

                let row_chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints(vec![Constraint::Length(1); DISPLAY_WIDTH])
                    .split(chunks[0]);

                for y in 0..DISPLAY_HEIGHT {
                    for x in 0..DISPLAY_WIDTH {
                        if self.pixels[y as usize][x as usize] {
                            let block = Block::default().style(Style::default().bg(Color::Red));
                            f.render_widget(
                                block,
                                Rect {
                                    x: row_chunks[x].x,
                                    y: chunks[0].y + y as u16,
                                    width: 1,
                                    height: 1,
                                },
                            );
                        }
                    }
                }
            })
            .unwrap();
    }

    pub fn draw(&mut self, x: u8, y: u8, sprite: u8) -> bool {
        let mut collision = false;
        for i in 0..8 {
            if sprite.checked_shr(8 - i).unwrap_or(0) & 0x1 == 0x1 {
                let dx = (x as usize + i as usize) % DISPLAY_WIDTH;
                let dy = y as usize % DISPLAY_HEIGHT;
                if self.pixels[dy][dx] {
                    collision = true;
                    self.pixels[dy][dx] = false;
                } else {
                    self.pixels[dy][dx] = true;
                    // println!("PIXEL");
                }
            }
        }
        collision
    }
}
