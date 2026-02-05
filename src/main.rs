use std::f32::consts::PI;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::time::Duration;
use sdl2::rect::{FPoint, FRect, Rect};
use sdl2::render::{WindowCanvas};
use nalgebra::*;
use log::{debug, log};

const WIDTH: i32 = 800;
const HEIGHT: i32 = 600;
const POINT_SIZE: f32 = 10.0;
pub fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let (points, faces) = get_vertices().unwrap();

    let window = video_subsystem.window("rust-sdl2 demo", WIDTH as u32, HEIGHT as u32)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();

    let mut event_pump = sdl_context.event_pump().unwrap();
    let z = 5.0;
    let mut angle = 1.0;

    let mut translation_params = TranslationParams {
        dx: 0.0,
        dy: 0.0,
        dz: 10.0,
        angle_x: 0.0,
        angle_y: 0.0,
        angle_z: 0.0,
    };
    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} |
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running
                },
                Event::KeyDown { keycode: Some(Keycode::RIGHT), ..} => {
                    angle = angle + PI * (1.0/60.0);
                },
                Event::KeyDown { keycode: Some(Keycode::LEFT), ..} => {
                    angle = angle - PI * (1.0/60.0);
                },
                Event::KeyDown { keycode: Some(Keycode::W), ..} => {
                    translation_params.dz = translation_params.dz + 0.3;
                },
                Event::KeyDown { keycode: Some(Keycode::S), ..} => {
                    translation_params.dz = translation_params.dz - 0.3;
                }
                _ => {}
            }
        }

        clear(&mut canvas);

        // for p in &points {
        //     point(&mut canvas, screen(project(translate_model(&translation_params, p.clone()))));
        // }

        for face in &faces {
            for (index, _) in face.iter().enumerate() {
                let p1 = points[face[index] as usize];
                let p2 = points[face[(index+1)%face.len()] as usize];
                line(
                    &mut canvas,
                    screen(project(translate_model(&translation_params, p1.clone()))),
                    screen(project(translate_model(&translation_params, p2.clone()))),
                );
            }
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

pub fn line (canvas: &mut WindowCanvas, p1: P, p2: P) {
    canvas.set_draw_color(Color::RGB(255, 210, 0));
    let p = FPoint::new(p1.x, p1.y);
    canvas.draw_fline(FPoint::new(p1.x, p1.y), FPoint::new(p2.x, p2.y));
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
        x: p.x * angle.cos() - p.z * angle.sin(),
        z: p.x * angle.sin() + p.z * angle.cos(),
        y: p.y
    }
}

pub fn translate_model(params: &TranslationParams, p: P) -> P {
    let v = Vector4::new(p.x, p.y, p.z, 1.0);

    let translation = Translation3::new(params.dx, params.dy, params.dz).to_homogeneous();
    let rx = Rotation3::from_euler_angles(params.angle_x, 0.0, 0.0).to_homogeneous();
    let ry = Rotation3::from_euler_angles(0.0, params.angle_y, 0.0).to_homogeneous();
    let rz = Rotation3::from_euler_angles(0.0, 0.0, params.angle_z).to_homogeneous();

    // Combine them (Order: Translation * Rotation)
    let model_matrix = translation * rz * ry * rx;

    let result = model_matrix * v;

    P {
        x: result[0],
        y: result[1],
        z: result[2]
    }
}

pub fn get_vertices() -> io::Result<(Vec<P>, Vec<Vec<i32>>)> {
    let file = File::open("teapot.obj")?;
    let reader = BufReader::new(file);
    let mut vertices: Vec<P> = Vec::new();
    let mut faces: Vec<Vec<i32>> = Vec::new();

    for (_index, line) in reader.lines().enumerate() {
        let line = line?;
        let words: Vec<&str> = line.split_whitespace().collect();
        if words.len() > 0 && words[0] == "v" {
            vertices.push(P {
                x: words[1].parse().expect("Not a valid number"),
                y: words[2].parse().expect("Not a valid number"),
                z: words[3].parse().expect("Not a valid number")
            });
        }
        if words.len() > 0 && words[0] == "f" {
            faces.push(words.iter().skip(1).map(|x| x.parse::<i32>().expect("Not a valid number") - 1).collect())
        }
    }
    Ok((vertices, faces))
}

#[derive(Debug, Clone, Copy)]
pub struct P {
    x: f32,
    y: f32,
    z: f32
}

pub struct TranslationParams {
    dx: f32,
    dy: f32,
    dz: f32,
    angle_x: f32,
    angle_y: f32,
    angle_z: f32,

}