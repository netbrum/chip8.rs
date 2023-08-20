mod emulator;

use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;
use sdl2::{pixels::Color, Sdl};

use emulator::{display, Emulator};
use sdl2::event::Event;
use std::{
    env,
    fs::File,
    io::{self, Read},
    process,
};

const TICKS_PER_FRAME: usize = 10;

// 64x32 is __really__ small on a modern screen so we scale it up
const WINDOW_SCALE: u32 = 15;
const WINDOW_WIDTH: u32 = display::DISPLAY_WIDTH as u32 * WINDOW_SCALE;
const WINDOW_HEIGHT: u32 = display::DISPLAY_HEIGHT as u32 * WINDOW_SCALE;

fn main() -> Result<(), String> {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        println!("Usage: cargo run path/to/rom");
        process::exit(1);
    }

    let mut emulator = Emulator::new();

    let sdl_context = sdl2::init()?;
    let mut event_pump = sdl_context.event_pump()?;
    let mut canvas = init_canvas(&sdl_context)?;

    match read_rom(&args[1]) {
        Ok(rom) => emulator.load_rom(&rom),
        Err(err) => panic!("{err}"),
    }

    'main: loop {
        // Quit on ctrl-c
        for event in event_pump.poll_iter() {
            if let Event::Quit { .. } = event {
                break 'main;
            };
        }

        emulator.keyboard.poll(&event_pump);

        for _ in 0..TICKS_PER_FRAME {
            emulator.tick();
        }

        emulator.tick_timers();
        draw_screen(&emulator, &mut canvas);
    }

    Ok(())
}

fn init_canvas(sdl_context: &Sdl) -> Result<Canvas<Window>, String> {
    let video_subsystem = sdl_context.video()?;
    let window = video_subsystem
        .window("chip8.rs", WINDOW_WIDTH, WINDOW_HEIGHT)
        .position_centered()
        .opengl()
        .build()
        .expect("to build window");

    let mut canvas = window
        .into_canvas()
        .present_vsync()
        .build()
        .expect("to build canvas");

    canvas.present();

    Ok(canvas)
}

fn draw_screen(emulator: &Emulator, canvas: &mut Canvas<Window>) {
    canvas.set_draw_color(Color::BLACK);
    canvas.clear();

    let frame_buffer = emulator.display.screen;

    canvas.set_draw_color(Color::WHITE);

    for (y, row) in frame_buffer.iter().enumerate() {
        for (x, &col) in row.iter().enumerate() {
            if col {
                let x = (x as u32) * WINDOW_SCALE;
                let y = (y as u32) * WINDOW_SCALE;

                canvas
                    .fill_rect(Rect::new(x as i32, y as i32, WINDOW_SCALE, WINDOW_SCALE))
                    .expect("to draw rectangle on canvas");
            }
        }
    }

    canvas.present();
}

fn read_rom(file: &String) -> Result<Vec<u8>, io::Error> {
    let mut rom = File::open(file)?;
    let mut buffer = Vec::new();

    rom.read_to_end(&mut buffer)?;

    Ok(buffer)
}
