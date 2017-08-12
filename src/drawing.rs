// Copyright (c) 2017 Brandon Thomas <bt@brand.io>
// Painting with Lasers

use error::PaintError;
use lase::Point;
use lase::tools::ETHERDREAM_X_MAX;
use lase::tools::ETHERDREAM_X_MIN;
use lase::tools::ETHERDREAM_COLOR_MAX;
use std::sync::Arc;
use std::sync::RwLock;
use std::time::Instant;

/// Position of the laser pointer in a webcam frame.
#[derive(Debug)]
pub struct ImagePosition {
  pub x: u32,
  pub y: u32,
}

/// Debug data from the camera.
#[derive(Debug)]
pub struct CameraCapture {
  pub position: ImagePosition,
  pub time: Instant,
}

/// A payload to draw to the laser.
pub struct DrawPayload {
  pub points: Vec<Point>,
  pub next_cursor: usize,
}

/// Canvas for drawing.
pub struct Canvas {
  /// Debug data
  camera_data: RwLock<Vec<CameraCapture>>,
  /// Points to draw
  laser_points: RwLock<Vec<Point>>,

  /// Camera dimensions.
  image_width: u32,
  image_height: u32,

  // Points to track back to source.
  tracking_points: usize,
}

impl Canvas {
  /// CTOR.
  pub fn new(image_width: u32, image_height: u32, tracking_points: usize)
      -> Canvas {
    Canvas {
      camera_data: RwLock::new(Vec::new()),
      laser_points: RwLock::new(Vec::new()),
      image_width: image_width,
      image_height: image_height,
      tracking_points: tracking_points,
    }
  }

  /// Clear the canvas.
  pub fn reset(&self) -> Result<(), PaintError> {
    //self.camera_data.clear();

    Ok(())
  }

  /// Add a point to the canvas.
  pub fn add_point(&self, position: ImagePosition, time: Instant)
      -> Result<(), PaintError> {
    { // Scope for write lock
      let mut laser_points = self.laser_points.write()?;

      let mut x = map_point(position.x, self.image_width);
      let mut y = map_point(position.y, self.image_height);

      println!("New point xy: {}, {}", x, y);

      let mut interpolation = Vec::new();

      match laser_points.last() {
        None => {},
        Some(last_point) => {
          println!("Last xy: {}, {}", last_point.x, last_point.y);

          // TODO DEBUG ONLY
          //x = last_point.x.saturating_add(1000);
          //y = last_point.y.saturating_add(1000);

          let last_x = last_point.x;
          let last_y = last_point.y;
          let x_diff = last_x.saturating_sub(x) as f64;
          let y_diff = last_y.saturating_sub(y) as f64;

          const interpolate_pts : usize = 200;
          for i in 0 .. interpolate_pts {
            let percent = i as f64 / interpolate_pts as f64;
            let xb = last_x.saturating_sub((x_diff * percent) as i16);
            let yb = last_y.saturating_sub((y_diff * percent) as i16);

            interpolation.push(Point::xy_rgb(
              xb,
              yb,
              ETHERDREAM_COLOR_MAX/4,
              0,
              0,
            ));
          }
        },
      }

      if interpolation.len() > 0 {
        laser_points.extend(interpolation);
      }

      laser_points.push(Point::xy_rgb(
        x,
        y,
        ETHERDREAM_COLOR_MAX/4,
        0, // Cannot have green
        0,
      ));
    }

    Ok(())
  }

  /// Get a point at the given offset.
  pub fn get_points(&self, index: usize, num_points: usize)
      -> Result<DrawPayload, PaintError> {

    let mut buf = Vec::new();
    let mut laser_points = self.laser_points.read()?;

    // No points case
    if laser_points.len() < 1 {
      return Ok(DrawPayload {
        points: Self::blank_points(num_points),
        next_cursor: 0, // Doesn't matter. Infinite stream of blanks.
      })
    }

    let total_len = laser_points.len() + self.tracking_points;

    let mut i = index;

    while buf.len() < num_points {
      if i < laser_points.len() {
        let pt = laser_points.get(index).unwrap();
        buf.push(pt.clone());
      } else {
        let first_pt = laser_points.get(0).unwrap();
        let last_pt = laser_points.last().unwrap();

        let interpolation_pts = Self::interpolate_points(
          last_pt, first_pt, self.tracking_points);

        let j = (i - laser_points.len()) % total_len;

        let pt = interpolation_pts.get(i).unwrap();
        buf.push(pt.clone());
      }

      i = (i + 1) % laser_points.len();
    }

    Ok(DrawPayload {
      points: buf,
      next_cursor: i, // TODO
    })
  }

  fn blank_points(num_points: usize) -> Vec<Point> {
    let mut points = Vec::with_capacity(num_points);
    for _i in 0 .. num_points {
      points.push(Point::xy_blank(0, 0));
    }
    points
  }

  fn interpolate_points(last_point: &Point, next_point: &Point,
                        num_interpolation_points: usize) -> Vec<Point> {
    let mut buf = Vec::with_capacity(num_interpolation_points);

    let last_x = last_point.x;
    let last_y = last_point.y;
    let x_diff = last_x.saturating_sub(next_point.x) as f64;
    let y_diff = last_y.saturating_sub(next_point.y) as f64;

    for i in 0 .. num_interpolation_points {
      let percent = i as f64 / num_interpolation_points as f64;
      let xb = last_x.saturating_sub((x_diff * percent) as i16);
      let yb = last_y.saturating_sub((y_diff * percent) as i16);

      buf.push(Point::xy_rgb(
        xb,
        yb,
        ETHERDREAM_COLOR_MAX/4,
        0,
        0,
      ));
    }

    buf
  }
}

fn map_point(image_position: u32, image_scale: u32) -> i16 {
  let num = image_position as f64;
  let denom = image_scale as f64;
  let ratio = num / denom;
  let scale = ETHERDREAM_X_MAX as f64 - ETHERDREAM_X_MIN as f64;
  let result = ratio * scale;
  result as i16
}

