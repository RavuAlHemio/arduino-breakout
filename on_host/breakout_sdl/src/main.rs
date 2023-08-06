use std::thread::sleep;
use std::time::Duration;

use breakout_common::playfield::{
    BYTES_PER_PIXEL, DISPLAY_BYTES, DISPLAY_HEIGHT, DISPLAY_WIDTH, Playfield,
};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::{Color, PixelFormatEnum};
use sdl2::rect::Rect;
use sdl2::render::{Canvas, TextureAccess};
use sdl2::video::Window;


fn render_playfield(playfield: &Playfield, canvas: &mut Canvas<Window>) {
    let mut buf = [0u8; DISPLAY_BYTES];
    playfield.draw(&mut buf);

    let texture_creator = canvas.texture_creator();
    let mut texture = texture_creator.create_texture(
        Some(PixelFormatEnum::RGB565),
        TextureAccess::Static,
        DISPLAY_WIDTH.try_into().unwrap(),
        DISPLAY_HEIGHT.try_into().unwrap(),
    )
        .expect("failed to create texture");
    texture.update(
        Some(Rect::new(
            0,
            0,
            u32::try_from(DISPLAY_WIDTH).unwrap(),
            u32::try_from(DISPLAY_HEIGHT).unwrap(),
        )),
        &buf,
        DISPLAY_WIDTH*BYTES_PER_PIXEL,
    )
        .expect("failed to render into texture");
    canvas.copy(&texture, None, None)
        .expect("failed to copy texture");
}


fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem.window("breakout", 800, 600)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();

    canvas.set_draw_color(Color::RGB(0, 0, 0));

    let mut playfield = Playfield::new();

    'main_loop: loop {
        playfield.advance();

        canvas.clear();
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} |
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'main_loop;
                },
                _ => {}
            }
        }

        render_playfield(&playfield, &mut canvas);

        canvas.present();
        sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
}
