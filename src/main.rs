// Copyright (c) 2017 Brandon Thomas <bt@brand.io>
// Painting with Lasers

extern crate camera_capture;
extern crate image;
extern crate opencv;
extern crate piston_window;
extern crate texture;

use opencv::core;
use opencv::highgui;
use piston_window::{PistonWindow, Texture, WindowSettings, TextureSettings, clear};
use image::ConvertBuffer;

fn main() {
  println!("TODO: Everything.");
  let window: PistonWindow =
    WindowSettings::new("piston: image", [300, 300])
        .exit_on_esc(true)
        .build()
        .unwrap();

  let mut tex: Option<Texture<_>> = None;
  let (sender, receiver) = std::sync::mpsc::channel();
  let imgthread = std::thread::spawn(move || {
    let cam = camera_capture::create(0).unwrap()
        .fps(30.0)
        .unwrap()
        .start()
        .unwrap();
    for frame in cam {
      if let Err(_) = sender.send(frame.convert()) {
        break;
      }
    }
  });

  for e in window {
    if let Ok(frame) = receiver.try_recv() {
      if let Some(mut t) = tex {
        t.update(&mut *e.encoder.borrow_mut(), &frame).unwrap();
        tex = Some(t);
      } else {
        tex = Texture::from_image(&mut *e.factory.borrow_mut(), &frame, &TextureSettings::new()).ok();
      }
    }
    e.draw_2d(|c, g| {
      clear([1.0; 4], g);
      if let Some(ref t) = tex {
        piston_window::image(t, c.transform, g);
      }
    });
  }
  drop(receiver);
  imgthread.join().unwrap();
}

