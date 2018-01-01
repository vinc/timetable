extern crate transitfeed;
extern crate chrono;
extern crate colored;
extern crate getopts;
extern crate zip;

use chrono::DateTime;
use chrono::prelude::*;
use colored::Colorize;
use getopts::Options;
use std::collections::HashMap;
use std::env;

mod gtfs;
use gtfs::GTFS;

#[derive(PartialEq)]
enum Step {
    Departure,
    Arrival
}

pub struct Service {
    pub departure: DateTime<Local>,
    pub arrival: DateTime<Local>,
    pub short_name: String,
    pub long_name: String
}

#[derive(PartialEq, PartialOrd, Eq, Ord)]
pub struct Station {
    pub name: String
}

pub struct Search {
    gtfs: GTFS,
    debug: bool
}

impl Search {
    pub fn new(path: String) -> Search {
        let debug = false;
        let gtfs = GTFS::from_path(path);
        Search { gtfs, debug }
    }

    pub fn stations(&self) -> Vec<Station> {
        let mut results = Vec::new();
        for result in self.gtfs.stops() {
            if let Ok(entry) = result {
                let name = entry.stop_name;
                let station = Station { name };
                results.push(station);
            }
        }
        results.sort();
        results.dedup();
        results
    }

    pub fn timetable(&self, from: &str, to: &str, at: DateTime<Local>) -> Vec<Service> {
        let mut stop_ids = HashMap::new();
        let mut n = 0;
        let mut origins = Vec::new();
        let mut destinations = Vec::new();
        for result in self.gtfs.stops() {
            n += 1;
            if let Ok(entry) = result {
                let name = entry.stop_name.to_lowercase();

                if name.contains(from) {
                    stop_ids.insert(entry.stop_id, Step::Departure);
                    origins.push(entry.stop_name);
                } else if name.contains(to) {
                    stop_ids.insert(entry.stop_id, Step::Arrival);
                    destinations.push(entry.stop_name);
                }
            }
        }
        if self.debug {
            origins.sort();
            origins.dedup();
            destinations.sort();
            destinations.dedup();
            println!("{}: origins: {}", "Debug".cyan(), origins.join(", "));
            println!("{}: destinations: {}", "Debug".cyan(), destinations.join(", "));
            println!("{}: loaded {} stops ({} retained)", "Debug".cyan(), n, stop_ids.len());
        }

        let mut departure_stop_times = Vec::new();
        let mut arrival_stop_times = Vec::new();
        let mut n = 0;
        for result in self.gtfs.stop_times() {
            n += 1;
            if let Ok(entry) = result {
                match stop_ids.get(&entry.stop_id) {
                    Some(&Step::Departure) => departure_stop_times.push(entry),
                    Some(&Step::Arrival) => arrival_stop_times.push(entry),
                    _ => ()
                }
            }
        }
        if self.debug {
            let total = departure_stop_times.len() + arrival_stop_times.len();
            println!("{}: loaded {} stop times ({} retained)", "Debug".cyan(), n, total);
        }

        let date = at.date().naive_local();
        let midnight = at.date().and_hms(0, 0, 0);

        let mut trip_ids = HashMap::new();
        for stop_time in departure_stop_times {
            let departure = midnight + stop_time.departure_time.duration();
            if departure > at {
                trip_ids.insert(stop_time.trip_id, departure);
            }
        }

        let mut trips = HashMap::new();
        let mut service_ids = HashMap::new();
        let mut route_ids = HashMap::new();
        let mut n = 0;
        for result in self.gtfs.trips() {
            n += 1;
            if let Ok(entry) = result {
                let trip_id = entry.trip_id.clone();
                let route_id = entry.route_id.clone();
                let service_id = entry.service_id.clone();
                if trip_ids.contains_key(&trip_id) {
                    service_ids.insert(service_id, true);
                    route_ids.insert(route_id, true);
                    trips.insert(trip_id, entry);
                }
            }
        }
        if self.debug {
            println!("{}: loaded {} trips ({} retained)", "Debug".cyan(), n, trips.len());
        }

        let mut routes = HashMap::new();
        let mut n = 0;
        for result in self.gtfs.routes() {
            n += 1;
            if let Ok(entry) = result {
                let route_id = entry.route_id.clone();
                if route_ids.contains_key(&route_id) {
                    routes.insert(route_id, entry);
                }
            }
        }
        if self.debug {
            println!("{}: loaded {} routes ({} retained)", "Debug".cyan(), n, routes.len());
        }

        let mut services = HashMap::new();
        let mut n = 0;
        for result in self.gtfs.calendar() {
            n += 1;
            if let Ok(entry) = result {
                let service_id = entry.service_id;
                if service_ids.contains_key(&service_id) {
                    let is_running = match date.weekday() {
                        Weekday::Mon => entry.monday,
                        Weekday::Tue => entry.tuesday,
                        Weekday::Wed => entry.wednesday,
                        Weekday::Thu => entry.thursday,
                        Weekday::Fri => entry.friday,
                        Weekday::Sat => entry.saturday,
                        Weekday::Sun => entry.sunday
                    };
                    if !is_running {
                        continue;
                    }
                    if entry.start_date <= date && date <= entry.end_date {
                        services.insert(service_id, true);
                    }
                }
            }
        }
        if self.debug {
            println!("{}: loaded {} services ({} retained)", "Debug".cyan(), n, services.len());
        }

        let mut results = Vec::new();
        for stop_time in arrival_stop_times {
            let trip_id = stop_time.trip_id;
            if let Some(departure) = trip_ids.get(&trip_id) {
                let arrival = midnight + stop_time.arrival_time.duration();
                let departure = *departure;
                if arrival > departure {
                    if let Some(trip) = trips.get(&trip_id) {
                        if !services.is_empty() && !services.contains_key(&trip.service_id) {
                            continue;
                        }
                        if let Some(route) = routes.get(&trip.route_id) {
                            let short_name = route.route_short_name.clone();
                            let long_name = route.route_long_name.clone();
                            let service = Service {
                                departure,
                                arrival,
                                short_name,
                                long_name
                            };
                            results.push(service);
                        }
                    }
                }
            }
        }

        if self.debug {
            println!("");
        }
        results.sort_by(|a, b| a.departure.cmp(&b.departure));

        results
    }
}

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
    opts.optopt("z",  "zip",     "sync from zip", "ZIP");
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

    if matches.opt_present("z") {
        if let Some(archive) = matches.opt_str("z") {
            if matches.opt_present("d") {
                println!("{}: extracting '{}' to '{}'", "Debug".cyan(), archive, path);
            }
            gtfs::unzip_to_path(archive, path.clone());
        }
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
            if let Ok(naive_dt) = NaiveDateTime::parse_from_str(&s, "%Y-%m-%d %H:%M:%S") {
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
            let short = service.short_name.clone();
            let long = service.long_name.clone();
            println!(
                "{:13}{:11}{}",
                service.departure.format("%H:%M").to_string(),
                service.arrival.format("%H:%M").to_string(),
                vec![short, long].join(" - ")
            );
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
    }
}
