// Copyright (c) 2017 Brandon Thomas <bt@brand.io>
// Painting with Lasers

extern crate camera_capture;
extern crate image;
extern crate iron;
extern crate lase;
extern crate opencv;
extern crate piston_window;
extern crate router;
extern crate rscam;
extern crate serde;
extern crate serde_json;
extern crate texture;

#[macro_use] extern crate serde_derive;

mod laser;
mod server;

use opencv::core;
use opencv::highgui;
use piston_window::{PistonWindow, Texture, WindowSettings, TextureSettings, clear};
use image::ConvertBuffer;
use server::start_http_server;
use rscam::Frame;

type Image = image::ImageBuffer<image::Rgb<u8>, Frame>;

const WIDTH: u32 = 1280;
const HEIGHT: u32 = 720;

fn main() {
  println!("TODO: Everything.");
  unused_webcam();
  //start_http_server();
}

fn unused_webcam() {
  let window: PistonWindow =
    WindowSettings::new("piston: image", [WIDTH, HEIGHT])
        .exit_on_esc(true)
        .build()
        .unwrap();

  let mut tex: Option<Texture<_>> = None;
  let (sender, receiver) = std::sync::mpsc::channel();
  let imgthread = std::thread::spawn(move || {
    let cam = camera_capture::create(0).unwrap()
        .fps(30.0)
        .unwrap()
        .resolution(WIDTH, HEIGHT)
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

