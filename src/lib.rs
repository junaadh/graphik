pub mod constants;
pub mod graphik_circle;
pub mod graphik_line;
pub mod graphik_rect;
pub mod graphik_triangle;

use graphik_circle::GraphikCircle;
use graphik_line::GraphikLine;
use graphik_rect::GraphikRect;
use graphik_triangle::GraphikTriangle;

use std::{
    cell::RefCell,
    fs::{File, OpenOptions},
    io::Write,
    rc::Rc,
};

#[derive(Debug)]
pub enum Error {
    FileOpenError,
    FileWriteError,
}

#[derive(Debug, Clone)]
pub struct GraphikBuffer {
    pub width: usize,
    pub height: usize,
    pub buffer: Vec<u32>,
}

impl GraphikBuffer {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            buffer: vec![0; width * height],
        }
    }
}

pub fn get_center(canvas: usize, object: usize) -> i32 {
    ((canvas - object) / 2) as i32
}

pub fn lerpf(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

#[derive(Debug)]
pub struct GraphikBuilder {
    pub buffer: Rc<RefCell<GraphikBuffer>>,
}

impl GraphikBuilder {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            buffer: Rc::new(RefCell::new(GraphikBuffer::new(width, height))),
        }
    }

    pub fn fill(&mut self, color: u32) {
        self.buffer
            .borrow_mut()
            .buffer
            .iter_mut()
            .for_each(|pixel| {
                *pixel = color;
            });
    }

    pub fn rect_fill(&mut self, rect: &mut GraphikRect) {
        let mut buffer = self.buffer.borrow_mut();
        if rect.center {
            let x0 = get_center(buffer.width, rect.width);
            let y0 = get_center(buffer.height, rect.height);
            rect.origin(x0, y0);
        }

        for dy in 0..rect.height {
            let y = rect.y0 as usize + dy;
            if y < buffer.height {
                for dx in 0..rect.width {
                    let x = rect.x0 as usize + dx;
                    if x < buffer.width {
                        let bufwid = buffer.width;
                        buffer.buffer[y * bufwid + x] = rect.color;
                    }
                }
            }
        }
    }

    pub fn circle_fill(&mut self, circle: &mut GraphikCircle) {
        let mut buffer = self.buffer.borrow_mut();
        if circle.center {
            let x0 = (buffer.width / 2) as i32;
            let y0 = (buffer.height / 2) as i32;
            circle.origin(x0, y0);
        }

        let x1 = circle.x0 - circle.radius as i32;
        let y1 = circle.y0 - circle.radius as i32;
        let x2 = circle.x0 + circle.radius as i32;
        let y2 = circle.y0 + circle.radius as i32;
        for y in y1..y2 {
            if 0 <= y && y < buffer.height as i32 {
                for x in x1..x2 {
                    if 0 <= x && x < buffer.width as i32 {
                        let dx = x - circle.x0;
                        let dy = y - circle.y0;
                        if (dx * dx + dy * dy) <= (circle.radius * circle.radius) as i32 {
                            let bufwid = buffer.width;
                            buffer.buffer[y as usize * bufwid + x as usize] = circle.color;
                        }
                    }
                }
            }
        }
    }

    pub fn triangle_fill(&mut self, triangle: &mut GraphikTriangle) {
        let mut buffer = self.buffer.borrow_mut();

        let dx12 = triangle.x2 - triangle.x1;
        let dy12 = triangle.y2 - triangle.y1;
        let dx13 = triangle.x3 - triangle.x1;
        let dy13 = triangle.y3 - triangle.y1;

        for y in triangle.y1..=triangle.y2 {
            if 0 <= y && y < buffer.height as i32 {
                let s1 = if dy12 != 0 {
                    (y - triangle.y1) * dx12 / dy12 + triangle.x1
                } else {
                    triangle.x1
                };
                let s2 = if dy13 != 0 {
                    (y - triangle.y1) * dx13 / dy13 + triangle.x1
                } else {
                    triangle.x1
                };
                for x in s1..=s2 {
                    let width = buffer.width as i32;
                    if 0 <= x && x < width {
                        buffer.buffer[(y * width + x) as usize] = triangle.color;
                    }
                }
            }
        }

        let dx32 = triangle.x2 - triangle.x3;
        let dy32 = triangle.y2 - triangle.y3;
        let dx31 = triangle.x1 - triangle.x3;
        let dy31 = triangle.y1 - triangle.y3;

        for y in triangle.y2..=triangle.y3 {
            if 0 <= y && y < buffer.height as i32 {
                let s1 = if dy12 != 0 {
                    (y - triangle.y3) * dx32 / dy32 + triangle.x3
                } else {
                    triangle.x3
                };
                let s2 = if dy31 != 0 {
                    (y - triangle.y3) * dx31 / dy31 + triangle.x3
                } else {
                    triangle.x3
                };
                for x in s1..=s2 {
                    let width = buffer.width as i32;
                    if 0 <= x && x < width {
                        buffer.buffer[(y * width + x) as usize] = triangle.color;
                    }
                }
            }
        }
    }

    pub fn line_draw(&mut self, line: &mut GraphikLine) {
        let mut buffer = self.buffer.borrow_mut();
        self.process_line_vertices(line, buffer.width, buffer.height);

        let mut x0 = line.x0;
        let mut y0 = line.y0;
        let x1 = line.x1;
        let y1 = line.y1;
        let dx = (x1 - x0).abs();
        let dy = (y1 - y0).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut err = dx - dy;

        while x0 != x1 || y0 != y1 {
            if 0 <= x0 && x0 < buffer.width as i32 && 0 <= y0 && y0 < buffer.height as i32 {
                let bufwid = buffer.width;
                buffer.buffer[y0 as usize * bufwid + x0 as usize] = line.color;
            }
            let e2 = 2 * err;
            if e2 > -dy {
                err -= dy;
                x0 += sx;
            }
            if e2 < dx {
                err += dx;
                y0 += sy;
            }
        }
    }

    pub fn save_as_ppm(&self, file_path: &str) -> Result<(), Error> {
        let buffer = self.buffer.borrow();
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open(file_path)
            .map_err(|err| {
                eprintln!("Failed to open file {}: {}", &file_path, err);
                Error::FileOpenError
            })?;
        self.write_header(&mut file, buffer.width, buffer.height)?;

        for pixel in buffer.buffer.iter() {
            let bytes = [
                (*pixel & 0xff) as u8,
                ((*pixel >> 8) & 0xff) as u8,
                ((*pixel >> 16) & 0xff) as u8,
            ];
            file.write_all(&bytes).map_err(|_| Error::FileWriteError)?;
        }
        Ok(())
    }

    fn write_header(&self, file: &mut File, width: usize, height: usize) -> Result<(), Error> {
        let header = format!("P6\n{} {} 255\n", width, height);
        file.write_all(header.as_bytes())
            .map_err(|_| Error::FileWriteError)?;
        Ok(())
    }

    fn process_line_vertices(&self, line: &mut GraphikLine, width: usize, height: usize) {
        if line.center {
            if line.vertical {
                let center_x = (width / 2) as i32;
                line.vertical(center_x, line.y0, line.y1);
            } else if line.horizontal {
                let center_y = (height / 2) as i32;
                line.horizontal(center_y, line.x0, line.x1);
            }
        }
        // let x1 = self.width as i32 - line.x1;
        // let y1 = self.height as i32 - line.y1;
        // line.end(x1, y1);
    }
}
