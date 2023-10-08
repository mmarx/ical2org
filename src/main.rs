mod datetime;

use std::iter::empty;

use clap::{builder::OsStr, command, value_parser, Arg, ArgAction, ArgGroup};

fn main() {
    env_logger::init();

    let matches = command!()
        .arg(
            Arg::new("print-timezones")
                .short('p')
                .long("print-timezones")
                .action(ArgAction::SetTrue)
                .help("Print acceptable timezone names and exit."),
        )
        .arg(
            Arg::new("email")
                .short('e')
                .long("email")
                .action(ArgAction::Append)
                .default_values(empty::<OsStr>())
                .help("User email address (used to deal with declined events)."),
        )
        .arg(
            Arg::new("days")
                .short('d')
                .long("days")
                .default_value("90")
                .value_parser(value_parser!(u32))
                .help(
                    "Window length in days ".to_owned()
                        + "(left & right from current time)."
                        + "Has to be weakly positive.",
                ),
        )
        .arg(
            Arg::new("timezone")
                .short('t')
                .long("timezone")
                .required(false)
                .help("Timezone to use (defaults to local timezone)."),
        )
        .arg(
            Arg::new("location")
                .long("location")
                .action(ArgAction::SetTrue)
                .required(false),
        )
        .arg(
            Arg::new("continue")
                .long("continue-on-error")
                .action(ArgAction::SetTrue)
                .help("Attempt to continue even if some events are not handled."),
        )
        .get_matches();

    println!("Hello, world!");
}
