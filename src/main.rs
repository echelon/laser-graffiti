// Copyright (c) 2017 Brandon Thomas <bt@brand.io>
// Painting with Lasers

extern crate argparse;
extern crate beam;
extern crate camera_capture;
extern crate image;
extern crate lase;
extern crate piston;
extern crate piston_window;
extern crate router;
extern crate rscam;
extern crate texture;

mod arguments;
mod drawing;
mod error;

use arguments::Arguments;
use drawing::Canvas;
use drawing::ImagePosition;
use image::ConvertBuffer;
use image::ImageBuffer;
use image::Pixel;
use lase::tools::find_first_etherdream_dac;
use piston_window::Button;
use piston_window::PressEvent;
use piston_window::{PistonWindow, Texture, WindowSettings, TextureSettings, clear};
use rscam::Frame;
use std::sync::Arc;
use std::time::Instant;

type ImageFrame = image::ImageBuffer<image::Rgb<u8>, Frame>;
type ImageFrameRgba = ImageBuffer<image::Rgba<u8>, Vec<u8>>;

const THRESHOLD: u8 = 180;
const TRACKING_POINTS : i32 = 5; // Num of points to blank.

fn main() {
  let args = Arguments::parse_args();

  let canvas = Arc::new(Canvas::new(TRACKING_POINTS as usize, &args));
  let canvas2 = canvas.clone();

  let mut dac = find_first_etherdream_dac().expect("Unable to find DAC");

  std::thread::spawn(move || {
    let mut current_point = 0;

    dac.play_function(move |num_points: u16| {
      let num_points = num_points as usize;

      let payload = canvas.get_points(current_point, num_points)
          .expect("Failure to get points!");

      current_point = payload.next_cursor;

      payload.points
    }).expect("Projection failed.");
  });

  unused_webcam(canvas2, &args);
}

fn to_grayscale(frame: ImageFrame, args: &Arguments) -> ImageFrameRgba {
  let (width, height) = frame.dimensions();
  let mut new_image : ImageFrameRgba =
      ImageBuffer::new(args.webcam_width, args.webcam_height);

  for i in 0..width {
    for j in 0..height {
      let pix = frame.get_pixel(i, j);
      let rgba = pix.to_rgba();
      let mut pix2 = rgba.clone();
      pix2.apply(|pix: u8| {
        if pix > THRESHOLD {
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

fn unused_webcam(canvas: Arc<Canvas>, args: &Arguments) {

  let (sender, receiver) = std::sync::mpsc::channel();
  let mut tex: Option<Texture<_>> = None;
  let mut window: Option<PistonWindow> = None;

  if args.show_gui {
    window = Some(WindowSettings::new("Webcam capture",
        [args.webcam_width, args.webcam_height])
        .exit_on_esc(true)
        .build()
        .unwrap());
  }

  let args2 = (*args).clone();
  let canvas2 = canvas.clone();

  let imgthread = std::thread::spawn(move || {
    let cam = camera_capture::create(args2.webcam_index).unwrap()
        .fps(30.0)
        .unwrap()
        .resolution(args2.webcam_width, args2.webcam_height)
        .unwrap()
        .start()
        .unwrap();

    for frame in cam {
      let grayscale = to_grayscale(frame, &args2);
      let converted: ImageFrameRgba = grayscale.convert();

      let maybe_pos = find_laser_position(converted);
      if let Some(pos) = maybe_pos {
        println!("Found Point : {:?}", pos);
        canvas.add_point(pos, Instant::now())
            .expect("Could not add points");
      }

      if let Err(_) = sender.send(grayscale) {
        break;
      }
    }
  });

  if args.show_gui {
    for e in window.unwrap() {
      if let Ok(frame) = receiver.try_recv() {
        if let Some(mut t) = tex {
          t.update(&mut *e.encoder.borrow_mut(), &frame).unwrap();
          tex = Some(t);
        } else {
          tex = Texture::from_image(
            &mut *e.factory.borrow_mut(),
            &frame,
            &TextureSettings::new()
          ).ok();
        }
      }
      e.draw_2d(|c, g| {
        clear([1.0; 4], g);
        if let Some(ref t) = tex {
          piston_window::image(t, c.transform, g);
        }
      });

      if let Some(button) = e.press_args() {
        println!("Pressed button: {:?}", button);
        canvas2.reset();
      }
    }
  }

  imgthread.join().unwrap();
}
