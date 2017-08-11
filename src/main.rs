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

use std::sync::Arc;
use lase::tools::ETHERDREAM_X_MAX;
use lase::tools::ETHERDREAM_X_MIN;
use lase::tools::ETHERDREAM_COLOR_MAX;
use std::sync::RwLock;
use lase::Point;
use image::Pixel;
use lase::tools::find_first_etherdream_dac;
use opencv::core;
use opencv::highgui;
use piston_window::{PistonWindow, Texture, WindowSettings, TextureSettings, clear};
use image::ImageBuffer;
use image::ConvertBuffer;
use server::start_http_server;
use rscam::Frame;

type ImageFrame = image::ImageBuffer<image::Rgb<u8>, Frame>;
type ImageFrameRgba = ImageBuffer<image::Rgba<u8>, Vec<u8>>;

const WIDTH: u32 = 640;
const HEIGHT: u32 = 480;

#[derive(Debug)]
struct ImagePosition {
  pub x: u32,
  pub y: u32,
}

struct Drawing {
  pub path: Vec<Point>,
}

impl Drawing {
  pub fn new() -> Drawing {
    Drawing { path: Vec::new() }
  }
}

fn main() {
  println!("TODO: Everything.");
  //start_http_server();

  let drawing = Arc::new(RwLock::new(Drawing::new()));
  let drawing2 = drawing.clone();


  let mut dac = find_first_etherdream_dac().expect("Unable to find DAC");

  std::thread::spawn(move || {
    let mut current_point = 0;

    dac.play_function(move |num_points: u16| {
      let num_points = num_points as usize;
      let mut buf = Vec::new();

      //println!("Wants points: {}", num_points);

      match drawing.read() {
        Err(_) => println!("Problem reading drawing"),
        Ok(drawing) => {
          let path_size = drawing.path.len();

          if path_size == 0 {
            for _ in 0..num_points {
              buf.push(Point::xy_binary(0, 0, false))
            }
          } else {
            //println!("Draw Time!");
            while buf.len() < num_points {
              let pt = drawing.path.get(current_point).unwrap();
              current_point = (current_point + 1) % path_size;
              //println!("Current point: {}", current_point);
              buf.push(Point::xy_rgb(pt.x, pt.y, ETHERDREAM_COLOR_MAX/4, 0, ETHERDREAM_COLOR_MAX/4))
            }
          }
        }
      }

      buf
    });
  });

  unused_webcam(drawing2);
}

fn to_grayscale(frame: ImageFrame) -> ImageFrameRgba {
  let (width, height) = frame.dimensions();
  let mut new_image : ImageFrameRgba = ImageBuffer::new(WIDTH, HEIGHT);

  for i in 0..width {
    for j in 0..height {
      let pix = frame.get_pixel(i, j);
      let rgba = pix.to_rgba();
      let mut pix2 = rgba.clone();
      pix2.apply(|pix: u8| {
        if pix > 180 {
          255
        } else {
          0
        }
      });

      new_image.put_pixel(i, j, pix2);
    }
  }

  new_image
}

fn find_laser_position(frame: ImageFrameRgba) -> Option<ImagePosition> {
  // FIXME: This crudely finds the first pixel that has a saturated green channel
  // We need to find the centroid of the largest saturation cluster.
  let (width, height) = frame.dimensions();

  for i in 0..width {
    for j in 0..height {
      let pix = frame.get_pixel(i, j);
      let g = pix.data[1]; // green channel
      if g == 255 {
        return Some(ImagePosition { x: i, y: j } )
      }
    }
  }
  None
}

fn unused_webcam(mut drawing: Arc<RwLock<Drawing>>) {
  /*let window: PistonWindow =
    WindowSettings::new("piston: image", [WIDTH, HEIGHT])
        .exit_on_esc(true)
        .build()
        .unwrap();

  let (sender, receiver) = std::sync::mpsc::channel();
  let mut tex: Option<Texture<_>> = None;*/

  let imgthread = std::thread::spawn(move || {
    let cam = camera_capture::create(1).unwrap()
        .fps(30.0)
        .unwrap()
        .resolution(WIDTH, HEIGHT)
        .unwrap()
        .start()
        .unwrap();

    for frame in cam {
      let grayscale = to_grayscale(frame);
      let converted : ImageBuffer<image::Rgba<u8>, Vec<u8>> = grayscale.convert();

      let maybe_pos = find_laser_position(converted);
      println!("Position: {:?}", maybe_pos);

      if let Some(pos) = maybe_pos {
        match drawing.write() {
          Err(_) => println!("Error obtaining write lock"),
          Ok(mut drawing) => {

            let x = map_point(pos.x, WIDTH);
            let y = map_point(pos.y, HEIGHT);


            drawing.path.push(Point::xy_binary(x, y, true));
          }
        }

      }

      /*if let Err(_) = sender.send(grayscale) {
        break;
      }*/
    }
  });

  /*for e in window {
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
  drop(receiver);*/
  imgthread.join().unwrap();
}

fn map_point(image_position: u32, image_scale: u32) -> i16 {
  let num = image_position as f64;
  let denom = image_scale as f64;
  let ratio = num / denom;
  let scale = ETHERDREAM_X_MAX as f64 - ETHERDREAM_X_MIN as f64;
  let result = ratio * scale;
  result as i16
}

/*fn webcam_x(laser_x: i16, image_width: u32) -> u32 {
  map_point(laser_x, image_width)
}

fn webcam_y(laser_y: i16, image_height : u32) -> u32 {
  let laser_y = laser_y * -1; // Inverted
  map_point(laser_y, image_height)
}*/

