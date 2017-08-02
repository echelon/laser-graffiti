
use lase::Dac;
use lase::Point;
use lase::tools::ETHERDREAM_COLOR_MAX;
use lase::tools::ETHERDREAM_X_MAX;
use lase::tools::ETHERDREAM_X_MIN;
use lase::tools::ETHERDREAM_Y_MAX;
use lase::tools::ETHERDREAM_Y_MIN;
use lase::tools::find_first_etherdream_dac;
use std::f64::consts::PI;
use std::f64;
use std::sync::Arc;
use std::sync::RwLock;
use std::thread;

fn test() {
  let mut dac = find_first_etherdream_dac().expect("Couldn't find DAC.");

  dac.play_function(|num_points: u16| {
    let mut points = Vec::new();
    points
  });
}