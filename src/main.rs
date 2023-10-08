//! ical2org -- convert Ical calendars to org agendas
#![deny(missing_docs)]

mod converter;
mod datetime;

use std::{
    fs::File,
    io::{self, BufRead, BufReader, BufWriter, Write},
    path::PathBuf,
};

use clap::Parser;

use converter::Converter;

#[derive(Parser, Debug)]
#[command(version, author, about, long_about = None)]
struct Args {
    /// Print acceptable timezone names and exit.
    #[arg(short, long)]
    print_timezones: bool,

    /// User email address (used to deal with declined events).
    #[arg(short, long)]
    email: Vec<String>,

    /// Window length in days (left & right from current time). Has to be weakly positive.
    #[arg(short, long, default_value_t = 90)]
    days: u32,

    /// Timezone to use (defaults to local timezone).
    #[arg(short, long)]
    timezone: Option<chrono_tz::Tz>,

    /// Include location in titles.
    #[arg(long, default_value_t = false)]
    location: bool,

    /// Attempt to continue even if some events are not handled.
    #[arg(long, default_value_t = false)]
    continue_on_error: bool,

    /// Icalendar file to convert.
    #[arg(required = true)]
    ics_file: PathBuf,

    /// Org agenda file to write to.
    #[arg(required = true)]
    org_file: PathBuf,
}

fn main() {
    env_logger::init();

    let matches = Args::parse();
    let ics_file: Box<dyn BufRead> = if matches.ics_file.to_string_lossy() == "-" {
        Box::new(BufReader::new(io::stdin()))
    } else {
        let file = File::open(matches.ics_file.clone());

        match file {
            Err(err) => {
                log::error!(
                    "Failed to open icalendar file `{}' for reading: {err:?}",
                    matches.ics_file.to_string_lossy()
                );

                return;
            }
            Ok(file) => Box::new(BufReader::new(file)),
        }
    };

    let mut org_file: Box<dyn Write> = if matches.org_file.to_string_lossy() == "-" {
        Box::new(BufWriter::new(io::stdout()))
    } else {
        let file = File::create(matches.org_file.clone());

        match file {
            Err(err) => {
                log::error!(
                    "Failed to open org agenda file `{}' for writing: {err:?}",
                    matches.org_file.to_string_lossy()
                );

                return;
            }
            Ok(file) => Box::new(BufWriter::new(file)),
        }
    };

    let converter = Converter::new(
        matches.days,
        matches.email,
        matches.timezone,
        matches.location,
        matches.continue_on_error,
    );

    if let Err(err) = converter.convert(ics_file, &mut org_file) {
        log::error!(
            "Failed to convert ical calendar file `{}' to org agenda file `{}': {err:?}",
            matches.ics_file.to_string_lossy(),
            matches.org_file.to_string_lossy()
        );
    }
}
