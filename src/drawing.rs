// Copyright (c) 2017 Brandon Thomas <bt@brand.io>
// Painting with Lasers

use arguments::Arguments;
use error::PaintError;
use lase::Point;
use lase::tools::ETHERDREAM_COLOR_MAX;
use std::sync::Mutex;
use std::sync::RwLock;
use std::time::Duration;
use std::time::Instant;

/// Position of the laser pointer in a webcam frame.
#[derive(Debug)]
pub struct ImagePosition {
  pub x: u32,
  pub y: u32,
}

/// A payload to draw to the laser.
pub struct DrawPayload {
  pub points: Vec<Point>,
  pub next_cursor: usize,
}

/// Canvas for drawing.
pub struct Canvas {
  /// Points to draw
  laser_points: RwLock<Vec<Point>>,

  /// Time the last point was added
  // NB: Technically doesn't need to be behind a lock, but is for exterior
  // immutability ergonomics.
  last_added_time: Mutex<Option<Instant>>,

  /// Camera dimensions.
  image_width: u32,
  image_height: u32,

  /// Points to track back to source.
  tracking_points: usize,

  /// Dimension maxima/minima
  x_max: i16,
  x_min: i16,
  y_max: i16,
  y_min: i16,

  /// Projected colors
  red: u16,
  blue: u16,

  /// Projected offset
  x_offset: i16,
  y_offset: i16,
}

impl Canvas {
  /// CTOR.
  pub fn new(tracking_points: usize, args: &Arguments) -> Canvas {
    Canvas {
      laser_points: RwLock::new(Vec::new()),
      last_added_time: Mutex::new(None),
      image_width: args.webcam_width,
      image_height: args.webcam_height,
      tracking_points: tracking_points,
      x_max: args.x_max,
      x_min: args.x_min,
      y_max: args.y_max,
      y_min: args.y_min,
      red: args.red,
      blue: args.blue,
      x_offset: args.x_offset,
      y_offset: args.y_offset,
    }
  }

  /// Clear the canvas.
  pub fn reset(&self) -> Result<(), PaintError> {
    {
      let mut time = self.last_added_time.lock()?;
      *time = None;
    }

    {
      let mut points = self.laser_points.write()?;
      *points = Vec::new();
    }

    Ok(())
  }

  /// Add a point to the canvas.
  pub fn add_point(&self, position: ImagePosition, time: Instant)
      -> Result<(), PaintError> {
    { // Scope for write lock
      let mut laser_points = self.laser_points.write()?;

      let x = self.map_x_point(position.x, self.image_width);
      let y = self.map_y_point(position.y, self.image_height);

      println!("New point xy: {}, {}", x, y);

      let mut interpolation = Vec::new();

      let mut is_blank = false;

      // FIXME: Inefficient locking.
      {
        let mut t = self.last_added_time.lock()?;
        if t.is_some() {
          let last_time = t.unwrap();
          let duration = time.duration_since(last_time);
          if duration > Duration::from_secs(1) {
            is_blank = true;
            println!("New Blank Point");
          }
        }
      }

      match laser_points.last() {
        None => {},
        Some(last_point) => {
          println!("Last xy: {}, {}", last_point.x, last_point.y);

          let last_x = last_point.x;
          let last_y = last_point.y;
          let x_diff = last_x.saturating_sub(x) as f64;
          let y_diff = last_y.saturating_sub(y) as f64;

          const interpolate_pts : usize = 20;
          for i in 0 .. interpolate_pts {
            let percent = i as f64 / interpolate_pts as f64;
            let xb = last_x.saturating_sub((x_diff * percent) as i16);
            let yb = last_y.saturating_sub((y_diff * percent) as i16);

            let point = if is_blank {
              Point::xy_blank(xb, yb)
            } else {
              Point::xy_rgb(xb, yb, self.red, 0, self.blue) // No green!
            };

            interpolation.push(point);
          }
        },
      }

      if interpolation.len() > 0 {
        laser_points.extend(interpolation);
      }

      laser_points.push(Point::xy_rgb(
        x,
        y,
        self.red,
        0, // Cannot have green
        self.blue,
      ));
    }

    let mut g = self.last_added_time.lock()?;
    *g = Some(time);

    Ok(())
  }

  /// Get a point at the given offset.
  pub fn get_points(&self, index: usize, num_points: usize)
      -> Result<DrawPayload, PaintError> {

    let mut buf = Vec::new();
    let laser_points = self.laser_points.read()?;

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

        let interpolation_pts = self.interpolate_points(
          last_pt, first_pt, self.tracking_points, true);

        let j = (i - laser_points.len()) % total_len;

        let pt = interpolation_pts.get(i).unwrap(); // TODO: shouldn't this be 'j'?
        buf.push(pt.clone());
      }

      i = (i + 1) % laser_points.len();
    }

    Ok(DrawPayload {
      points: buf,
      next_cursor: i,
    })
  }

  fn map_x_point(&self, image_position: u32, image_scale: u32) -> i16 {
    let num = image_position as f64;
    let denom = image_scale as f64;
    let ratio = num / denom;
    let scale = self.x_max.saturating_sub(self.x_min) as f64;
    let result = ratio * scale * -1.0;
    let result = result as i16;

    result.saturating_add(self.x_offset)
  }

  fn map_y_point(&self, image_position: u32, image_scale: u32) -> i16 {
    let num = image_position as f64;
    let denom = image_scale as f64;
    let ratio = num / denom;
    let scale = self.y_max.saturating_sub(self.y_min) as f64;
    let result = ratio * scale * -1.0;
    let result = result as i16;

    result.saturating_add(self.y_offset)
  }

  fn blank_points(num_points: usize) -> Vec<Point> {
    let mut points = Vec::with_capacity(num_points);
    for _i in 0 .. num_points {
      points.push(Point::xy_blank(0, 0));
    }
    points
  }

  fn interpolate_points(&self, last_point: &Point, next_point: &Point,
                        num_interpolation_points: usize, blank: bool)
                        -> Vec<Point> {
    let mut buf = Vec::with_capacity(num_interpolation_points);

    let last_x = last_point.x;
    let last_y = last_point.y;
    let x_diff = last_x.saturating_sub(next_point.x) as f64;
    let y_diff = last_y.saturating_sub(next_point.y) as f64;

    for i in 0 .. num_interpolation_points {
      let percent = i as f64 / num_interpolation_points as f64;
      let xb = last_x.saturating_sub((x_diff * percent) as i16);
      let yb = last_y.saturating_sub((y_diff * percent) as i16);

      let point = if blank {
        Point::xy_blank(xb, yb)
      } else {
        Point::xy_rgb(xb, yb, self.red, 0, self.blue)
      };

      buf.push(point);
    }

    buf
  }
}

