use argparse::ArgumentParser;
use argparse::{Store, StoreTrue};

#[derive(Clone)]
pub struct Arguments {
  /// Whether or not to show the GUI.
  pub show_gui: bool,
  /// Which hardware webcam to use.
  pub webcam_index: u32,
}

impl Arguments {
  pub fn parse_args() -> Arguments {

    let mut args = Arguments {
      show_gui: false,
      webcam_index: 0,
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

      parser.parse_args_or_exit();
    }

    args
  }
}
