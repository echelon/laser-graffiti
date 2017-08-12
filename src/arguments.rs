use argparse::ArgumentParser;
use argparse::{Store, StoreTrue};
use lase::tools::ETHERDREAM_X_MAX;
use lase::tools::ETHERDREAM_X_MIN;
use lase::tools::ETHERDREAM_Y_MAX;
use lase::tools::ETHERDREAM_Y_MIN;

#[derive(Clone)]
pub struct Arguments {
  /// Whether or not to show the GUI.
  pub show_gui: bool,
  /// Which hardware webcam to use.
  pub webcam_index: u32,

  /// Maximum projected X coordinate
  pub x_max: i16,
  /// Minimum projected X coordinate
  pub x_min: i16,
  /// Maximum projected Y coordinate
  pub y_max: i16,
  /// Minimum projected Y coordinate
  pub y_min: i16,
}

impl Arguments {
  pub fn parse_args() -> Arguments {

    let mut args = Arguments {
      show_gui: false,
      webcam_index: 0,
      x_max: ETHERDREAM_X_MAX/4,
      x_min: ETHERDREAM_X_MIN/4,
      y_max: ETHERDREAM_Y_MAX/4,
      y_min: ETHERDREAM_Y_MIN/4,
    };

    {
      // Limit scope of borrow.
      let mut parser = ArgumentParser::new();

      parser.set_description("Paint with lasers.");

      parser.refer(&mut args.show_gui)
          .add_option(&["-g", "--show-gui"], StoreTrue,
            "Show the GUI.");

      parser.refer(&mut args.webcam_index)
          .add_option(&["-w", "--webcam"], Store,
            "Numeric hardware index of webcam to use: /dev/video{N}");

      parser.refer(&mut args.x_max)
          .add_option(&["--x-max"], Store, "Maximum projected x coordinate");

      parser.refer(&mut args.x_min)
          .add_option(&["--x-min"], Store, "Minimum projected x coordinate");

      parser.refer(&mut args.y_max)
          .add_option(&["--y-max"], Store, "Maximum projected y coordinate");

      parser.refer(&mut args.y_min)
          .add_option(&["--y-min"], Store, "Minimum projected y coordinate");

      parser.parse_args_or_exit();
    }

    args
  }
}
