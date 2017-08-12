use argparse::ArgumentParser;
use argparse::{Store, StoreTrue};

pub struct Arguments {
  pub show_gui: bool,
}

impl Arguments {
  pub fn parse_args() -> Arguments {

    let mut args = Arguments {
      show_gui: false,
    };

    {
      // Limit scope of borrow.
      let mut parser = ArgumentParser::new();

      parser.set_description("Paint with lasers.");

      parser.refer(&mut args.show_gui)
          .add_option(&["-g", "--show-gui"], StoreTrue,
            "Show the GUI.");

      /*parser.refer(&mut frame_repeat_number)
          .add_option(&["-r", "--repeat-frames"], Store,
            "Number of times to repeat frames");*/

      parser.parse_args_or_exit();
    }

    args
  }
}
