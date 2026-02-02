use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::time::Duration;
use sdl2::rect::{FRect, Rect};
use sdl2::render::{Canvas, WindowCanvas};
use log::debug;

const WIDTH: i32 = 800;
const HEIGHT: i32 = 600;
const POINT_SIZE: f32 = 10.0;
pub fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let points: Vec<P> = vec![
        P { x: 0.5, y: 0.5, z: 0.5},
        P { x: -0.5, y: 0.5, z: 0.5},
        P { x: 0.5, y: -0.5, z: 0.5},
        P { x: -0.5, y: -0.5, z: 0.5},

        P { x: 0.5, y: 0.5, z: -0.5},
        P { x: -0.5, y: 0.5, z: -0.5},
        P { x: 0.5, y: -0.5, z: -0.5},
        P { x: -0.5, y: -0.5, z: -0.5},
    ];

    let window = video_subsystem.window("rust-sdl2 demo", WIDTH as u32, HEIGHT as u32)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();

    let mut event_pump = sdl_context.event_pump().unwrap();
    let mut z = 0.0;
    'running: loop {
        z = z + 0.01;
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} |
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running
                },
                _ => {}
            }
        }

        clear(&mut canvas);

        for p in &points {
            point(&mut canvas, screen(project(translate(p.clone(), z))));
        }

        canvas.present();

        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
}

pub fn clear(canvas: &mut WindowCanvas) {
    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();
}

pub fn point (canvas: &mut WindowCanvas, p: P) {
    canvas.set_draw_color(Color::RGB(255, 210, 0));
    let fill_rect = FRect::new(p.x - POINT_SIZE/2.0, p.y - POINT_SIZE/2.0, POINT_SIZE, POINT_SIZE);
    canvas.fill_frect(fill_rect).unwrap();
}

pub fn screen(p: P) -> P {
    P {
        x: (p.x + 1.0)/2.0 * WIDTH as f32,
        y: (1.0 - (p.y + 1.0)/2.0) * HEIGHT as f32,
        z: p.z
    }
}

pub fn project(p: P) -> P {
    P {
        x: p.x / p.z,
        y: p.y / p.z,
        z: p.z
    }
}

pub fn translate(p: P, dz: f32) -> P {
    P {
        x: p.x,
        y: p.y,
        z: p.z + dz
    }
}

pub fn rotate_x(p: P, angle: f32) -> P {
    P {
        x: p.x * angle.cos() - p.y * angle.sin(),
        y: p.x * angle.sin() + p.y * angle.cos(),
        z: p.z
    }
}

#[derive(Debug, Clone, Copy)]
pub struct P {
    x: f32,
    y: f32,
    z: f32
}