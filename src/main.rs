extern crate chrono;
extern crate colored;
extern crate geodate;
extern crate getopts;
extern crate reqwest;
extern crate transitfeed;
extern crate zip;
extern crate regex;

use chrono::prelude::*;
use colored::Colorize;
use geodate::geodate::get_formatted_date;
use getopts::Options;
use regex::RegexSet;
use std::env;

mod gtfs;
mod search;

use search::Search;

fn print_usage(opts: Options) {
    let brief = format!("Usage: timetable [options]");
    print!("{}", opts.usage(&brief));
}

fn print_version() {
    let version = String::from("v") + env!("CARGO_PKG_VERSION");
    println!("timetable {}", version)
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let mut opts = Options::new();
    opts.optopt("p",  "path",    "gtfs path",     "GTFS");
    opts.optopt("f",  "from",    "depart from",   "NAME");
    opts.optopt("t",  "to",      "arrive to",     "NAME");
    opts.optopt("a",  "at",      "depart at",     "TIME");
    opts.optopt("u",  "url",     "sync from url", "URL");
    opts.optopt("z",  "zip",     "sync from zip", "ZIP");
    opts.optflag("g", "geodate", "use geodate format");
    opts.optflag("d", "debug",   "enable debug output");
    opts.optflag("h", "help",    "print this message");
    opts.optflag("v", "version", "print version");

    let matches = match opts.parse(&args) {
        Ok(m) => { m }
        Err(f) => { panic!(f.to_string()) }
    };

    if matches.opt_present("h") {
        print_usage(opts);
        return;
    }

    if matches.opt_present("v") {
        print_version();
        return;
    }

    let mut path = String::from(".");
    if matches.opt_present("p") {
        if let Some(s) = matches.opt_str("p") {
            path = s;
        }
    }

    if matches.opt_present("u") {
        if let Some(url) = matches.opt_str("u") {
            if matches.opt_present("d") {
                println!("{}: downloading '{}' to '{}'", "Debug".cyan(), url, path);
            }
            gtfs::download_to_path(url, path.clone());
            let archive = format!("{}/gtfs.zip", path);
            if matches.opt_present("d") {
                println!("{}: extracting '{}' to '{}'", "Debug".cyan(), archive, path);
            }
            gtfs::unzip_to_path(archive, path.clone());
        }
    }

    if matches.opt_present("z") {
        if let Some(archive) = matches.opt_str("z") {
            if matches.opt_present("d") {
                println!("{}: extracting '{}' to '{}'", "Debug".cyan(), archive, path);
            }
            gtfs::unzip_to_path(archive, path.clone());
        }
    }

    let mut use_geodate_format = false;
    if matches.opt_present("g") {
        use_geodate_format = true;
    }

    let mut search = Search::new(path);

    if matches.opt_present("d") {
        search.debug = true;
    }

    let mut from = String::new();
    if matches.opt_present("f") {
        if let Some(s) = matches.opt_str("f") {
            from = s;
        }
    }

    let mut to = String::new();
    if matches.opt_present("t") {
        if let Some(s) = matches.opt_str("t") {
            to = s;
        }
    }

    let mut at = Local::now();
    if matches.opt_present("a") {
        if let Some(s) = matches.opt_str("a") {
            let set = RegexSet::new(&[
                r"\d+-\d+-\d+ \d+:\d+:\d+",
                r"\d+:\d+:\d+",
                r"\d+:\d+",
            ]).unwrap();
            let matches = set.matches(&s);
            let format = if matches.matched(0) {
                "%Y-%m-%d %H:%M:%S"
            } else if matches.matched(1) {
                "%H:%M:%S"
            } else if matches.matched(2) {
                "%H:%M"
            } else {
                "%s"
            };
            if let Ok(naive_dt) = NaiveDateTime::parse_from_str(&s, format) {
                if let Some(dt) = at.timezone().from_local_datetime(&naive_dt).earliest() {
                    at = dt;
                }
            }
        }
    }

    if from.len() > 0 && to.len() > 0 {
        let results = search.timetable(&from, &to, at);
        println!("{:13}{:11}{}", "Departures".bold(), "Arrivals".bold(), "Routes".bold());
        for service in results.iter().take(5) {

            let mut departure = service.departure_time.format("%H:%M").to_string();
            let mut arrival = service.arrival_time.format("%H:%M").to_string();

            if use_geodate_format {
                let format = "%c:%b";
                let longitude = service.arrival_longitude;
                departure = get_formatted_date(format, service.departure_time.timestamp(), longitude);
                arrival = get_formatted_date(format, service.arrival_time.timestamp(), longitude);
            }

            println!("{} ......... {}   {}", departure, arrival, service.name());
        }
    } else if from.len() > 0 || to.len() > 0 {
        let results = search.stations();
        println!("{}", "Stations".bold());
        for station in results {
            let name = station.name.to_lowercase();

            if from.len() > 0 && name.contains(&from) {
                println!("{}", station.name);
            } else if to.len() > 0 && to.contains(&from) {
                println!("{}", station.name);
            }
        }
    } else if !matches.opt_present("z") && !matches.opt_present("u") {
        print_usage(opts);
    }
}
