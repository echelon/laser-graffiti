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

/// Number of points to blank from spiral's edge to center.
static BLANKING_POINTS : i32 = 10;

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
      //println!(">> LASER WANTS POINTS: {}", num_points);

      let num_points = num_points as usize;
      let mut buf = Vec::new();

      //println!("Wants points: {}", num_points);

      match drawing.read() {
        Err(_) => println!("Problem reading drawing"),
        Ok(drawing) => {
          let path_size = drawing.path.len();
          let total_points = path_size + BLANKING_POINTS as usize;

          if path_size == 0 {
            for _ in 0..num_points {
              buf.push(Point::xy_binary(0, 0, false))
            }
          } else {
            //println!("Draw Time!");
            //println!("Drawing path length: {}", path_size);

            while buf.len() < num_points {
              if current_point < path_size {
                let pt = drawing.path.get(current_point).unwrap();

                //println!("Current point: {}", current_point);
                buf.push(pt.clone());
              } else {
                let first_point = drawing.path.get(0).unwrap();
                let last_point = drawing.path.get(drawing.path.len() - 1).unwrap();

                let tracking_points = get_tracking_points(
                  first_point.x, first_point.y, last_point.x, last_point.y, 10);

                let mut index = current_point - path_size;

                /*println!("Path Size: {}, Total Points: {} ||| Current Point: {} ||| Tracking vec size: {}, index: {}",
                  path_size, total_points,
                  current_point,
                  tracking_points.len(), index);*/

                if let Some(pt) = tracking_points.get(index) {
                  buf.push(pt.clone());
                }
              }

              current_point = (current_point + 1) % total_points;
              //println!("next current point: {}, path_size: {}", current_point, path_size);
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

      if let Some(pos) = maybe_pos {
        println!("Found Point : {:?}", pos);

        match drawing.write() {
          Err(_) => println!("Error obtaining write lock"),
          Ok(mut drawing) => {

            let x = map_point(pos.x, WIDTH) * -1;
            let y = map_point(pos.y, HEIGHT) * -1;


            drawing.path.push(Point::xy_rgb(x, y, 0, 0, ETHERDREAM_COLOR_MAX / 4));
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


fn get_tracking_points(last_x: i16, last_y: i16, next_x: i16, next_y: i16, num_points: usize) -> Vec<Point> {
  /* Python tracking code:
  # Now, track to the next object.
  lastX = curObj.lastPt[0]
  lastY = curObj.lastPt[1]
  xDiff = curObj.lastPt[0] - nextObj.firstPt[0]
  yDiff = curObj.lastPt[1] - nextObj.firstPt[1]

  mv = TRACKING_SAMPLE_PTS
  for i in xrange(mv):
    percent = i/float(mv)
    xb = int(lastX - xDiff*percent)
    yb = int(lastY - yDiff*percent)
    # If we want to 'see' the tracking path (debug)
    if SHOW_TRACKING_PATH:
      yield (xb, yb, 0, CMAX, 0)
    else:
      yield (xb, yb, 0, 0, 0)*/

  let x_diff = last_x.saturating_sub(next_x) as f64;
  let y_diff = last_y.saturating_sub(next_y) as f64;

  let mut path = Vec::with_capacity(num_points);

  for i in 0 .. num_points {
    let percent = i as f64 / num_points as f64;
    let xb = last_x - (x_diff * percent) as i16;
    let yb = last_y - (y_diff * percent) as i16;

    path.push(Point::xy_rgb(xb, yb, ETHERDREAM_COLOR_MAX / 4, 0, 0));
  }

  path
}


