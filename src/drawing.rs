// Copyright (c) 2017 Brandon Thomas <bt@brand.io>
// Painting with Lasers

use error::PaintError;
use lase::Point;
use lase::tools::ETHERDREAM_X_MAX;
use lase::tools::ETHERDREAM_X_MIN;
use std::sync::Arc;
use std::sync::RwLock;
use std::time::Instant;

/// Position of the laser pointer in a webcam frame.
pub struct ImagePosition {
  pub x: u32,
  pub y: u32,
}

// Debug data from the camera.
pub struct CameraCapture {
  pub position: ImagePosition,
  pub time: Instant,
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
}

impl Canvas {
  /// CTOR.
  pub fn new(image_width: u32, image_height: u32) -> Canvas {
    Canvas {
      camera_data: RwLock::new(Vec::new()),
      laser_points: RwLock::new(Vec::new()),
      image_width: image_width,
      image_height: image_height,
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
    let mut laser_points = self.laser_points.write()?;

    let x = map_point(position.x, self.image_width);
    let y = map_point(position.y, self.image_height);

    laser_points.push(Point::xy_binary(x, y, true));

    Ok(())
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

