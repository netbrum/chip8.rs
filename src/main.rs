mod emulator;

use emulator::{display, Emulator};
use sdl2::event::Event;
use std::{
    env,
    fs::File,
    io::{self, Read},
    process, thread,
    time::Duration,
};

// 64x32 is __really__ small on a modern screen so we scale it up
const WINDOW_SCALE: u32 = 15;
const WINDOW_WIDTH: u32 = display::DISPLAY_WIDTH as u32 * WINDOW_SCALE;
const WINDOW_HEIGHT: u32 = display::DISPLAY_HEIGHT as u32 * WINDOW_SCALE;

fn main() -> Result<(), String> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("Missing path/to/rom");
        process::exit(1);
    }

    let mut emulator = Emulator::new();

    match get_rom(args) {
        Ok(rom) => emulator.load_rom(&rom),
        Err(err) => panic!("{err}"),
    }

    let sdl_context = sdl2::init()?;
    let mut event_pump = sdl_context.event_pump()?;
    let video_subsystem = sdl_context.video()?;
    let window = video_subsystem
        .window("august", WINDOW_WIDTH, WINDOW_HEIGHT)
        .position_centered()
        .opengl()
        .build()
        .expect("to build window");

    let mut canvas = window
        .into_canvas()
        .present_vsync()
        .build()
        .expect("to build canvas");
    canvas.clear();
    canvas.present();

    'main: loop {
        // Quit on ctrl-c
        for event in event_pump.poll_iter() {
            if let Event::Quit { .. } = event {
                break 'main;
            };
        }

        emulator.keyboard.poll(&event_pump);

        thread::sleep(Duration::from_millis(60));
    }

    Ok(())
}

fn get_rom(args: Vec<String>) -> Result<Vec<u8>, io::Error> {
    let mut rom = File::open(&args[1])?;
    let mut buffer = Vec::new();

    rom.read_to_end(&mut buffer)?;

    Ok(buffer)
}
