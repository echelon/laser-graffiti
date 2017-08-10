use argparse::ArgumentParser;
use argparse::{Store, StoreTrue};
use lase::tools::ETHERDREAM_X_MAX;
use lase::tools::ETHERDREAM_X_MIN;
use lase::tools::ETHERDREAM_Y_MAX;
use lase::tools::ETHERDREAM_Y_MIN;
use lase::tools::ETHERDREAM_COLOR_MAX;

#[derive(Clone)]
pub struct Arguments {
  /// Whether or not to show the GUI.
  pub show_gui: bool,

  /// Which hardware webcam to use.
  pub webcam_index: u32,
  /// Width of the captured webcam image
  pub webcam_width: u32,
  /// Height of the captured webcam image
  pub webcam_height: u32,

  /// Maximum projected X coordinate
  pub x_max: i16,
  /// Minimum projected X coordinate
  pub x_min: i16,
  /// Maximum projected Y coordinate
  pub y_max: i16,
  /// Minimum projected Y coordinate
  pub y_min: i16,

  /// Projected red.
  pub red: u16,
  /// Projected blue.
  pub blue: u16,

  /// Projected x offset
  pub x_offset: i16,

  /// Projected y offset
  pub y_offset: i16,
}

impl Arguments {
  pub fn parse_args() -> Arguments {

    let mut args = Arguments {
      show_gui: false,
      webcam_index: 0,
      webcam_width: 640,
      webcam_height: 480,
      x_max: ETHERDREAM_X_MAX/4,
      x_min: ETHERDREAM_X_MIN/4,
      y_max: ETHERDREAM_Y_MAX/4,
      y_min: ETHERDREAM_Y_MIN/4,
      red: ETHERDREAM_COLOR_MAX,//4,
      blue: ETHERDREAM_COLOR_MAX,//4,
      x_offset: 0,
      y_offset: 0,
    };

    {
      // Limit scope of borrow.
      let mut parser = ArgumentParser::new();

      parser.set_description("Paint with lasers.");

      parser.refer(&mut args.show_gui)
          .add_option(&["-g", "--show-gui"], StoreTrue,
            "Show the GUI.");

      parser.refer(&mut args.webcam_index)
          .add_option(&["-n", "--webcam"], Store,
            "Numeric hardware index of webcam to use: /dev/video{N}");

      parser.refer(&mut args.webcam_width)
          .add_option(&["-w", "--width"], Store,
            "Width of the webcam capture.");

      parser.refer(&mut args.webcam_height)
          .add_option(&["-h", "--height"], Store,
            "Height of the webcam capture.");

      parser.refer(&mut args.x_max)
          .add_option(&["--x-max"], Store, "Maximum projected x coordinate");

      parser.refer(&mut args.x_min)
          .add_option(&["--x-min"], Store, "Minimum projected x coordinate");

      parser.refer(&mut args.y_max)
          .add_option(&["--y-max"], Store, "Maximum projected y coordinate");

      parser.refer(&mut args.y_min)
          .add_option(&["--y-min"], Store, "Minimum projected y coordinate");

      parser.refer(&mut args.red)
          .add_option(&["--red"], Store, "Projected red value");
      parser.refer(&mut args.blue)
          .add_option(&["--blue"], Store, "Projected blue value");

      parser.refer(&mut args.x_offset)
          .add_option(&["--x-offset"], Store, "X offset");

      parser.refer(&mut args.y_offset)
          .add_option(&["--y-offset"], Store, "Y offset");

      parser.parse_args_or_exit();
    }

    args
  }
}
