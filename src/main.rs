extern crate transitfeed;
extern crate chrono;
extern crate getopts;

use transitfeed::{GTFSIterator, Route, RouteType, Stop, StopTime, Trip, Calendar};
use chrono::DateTime;
use chrono::prelude::*;
use getopts::Options;
use std::env;
use std::collections::HashMap;
use std::path::Path;

#[derive(PartialEq)]
enum Step {
    Departure,
    Arrival
}

pub struct Service {
    pub departure: DateTime<Local>,
    pub arrival: DateTime<Local>,
    pub vehicule: String,
    pub short_name: String,
    pub long_name: String
}

pub struct Search {
    path: String,
    pub from: Option<String>,
    pub to: Option<String>,
    pub at: DateTime<Local>,
    pub results: Option<Vec<Service>>
}

impl Search {
    pub fn new(path: String, from: String, to: String, at: DateTime<Local>) -> Search {
        Search {
            path: path,
            from: Some(from),
            to: Some(to),
            at: at,
            results: None
        }
    }

    pub fn run(&mut self) {
        let debug = false;
        let departure_name = &self.from.clone().unwrap();
        let arrival_name = &self.to.clone().unwrap();

        let mut stop_ids = HashMap::new();
        self.from = None;
        self.to = None;

        let mut n = 0;
        let path = Path::new(&self.path).join("stops.txt");
        let iter: GTFSIterator<_, Stop> = GTFSIterator::from_path(path.to_str().unwrap()).unwrap();
        for result in iter {
            n += 1;
            if let Ok(entry) = result {
                let name = entry.stop_name.to_lowercase();

                if name.contains(departure_name) {
                    stop_ids.insert(entry.stop_id, Step::Departure);
                    self.from = Some(entry.stop_name);
                } else if name.contains(arrival_name) {
                    stop_ids.insert(entry.stop_id, Step::Arrival);
                    self.to = Some(entry.stop_name);
                }
            }
        }
        if debug {
            println!("Loaded {} stops", n);
        }

        let mut departure_stop_times = Vec::new();
        let mut arrival_stop_times = Vec::new();
        let mut n = 0;
        let path = Path::new(&self.path).join("stop_times.txt");
        let iter: GTFSIterator<_, StopTime> = GTFSIterator::from_path(path.to_str().unwrap()).unwrap();
        for result in iter {
            n += 1;
            if let Ok(entry) = result {
                match stop_ids.get(&entry.stop_id) {
                    Some(&Step::Departure) => departure_stop_times.push(entry),
                    Some(&Step::Arrival) => arrival_stop_times.push(entry),
                    _ => ()
                }
            }
        }
        if debug {
            println!("Loaded {} stop times", n);
        }

        let date = self.at.date().naive_local();
        let midnight = self.at.date().and_hms(0, 0, 0);

        let mut trip_ids = HashMap::new();
        for stop_time in departure_stop_times {
            let departure = midnight + stop_time.departure_time.duration();
            if departure > self.at {
                trip_ids.insert(stop_time.trip_id, departure);
            }
        }

        let mut trips = HashMap::new();
        let mut service_ids = HashMap::new();
        let mut route_ids = HashMap::new();
        let mut n = 0;
        let path = Path::new(&self.path).join("trips.txt");
        let iter: GTFSIterator<_, Trip> = GTFSIterator::from_path(path.to_str().unwrap()).unwrap();
        for result in iter {
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
        if debug {
            println!("Loaded {} trips", n);
        }

        let mut routes = HashMap::new();
        let mut n = 0;
        let path = Path::new(&self.path).join("routes.txt");
        let iter: GTFSIterator<_, Route> = GTFSIterator::from_path(path.to_str().unwrap()).unwrap();
        for result in iter {
            n += 1;
            if let Ok(entry) = result {
                let route_id = entry.route_id.clone();
                if route_ids.contains_key(&route_id) {
                    routes.insert(route_id, entry);
                }
            }
        }
        if debug {
            println!("Loaded {} routes", n);
        }

        let mut services = HashMap::new();
        let mut n = 0;
        let path = Path::new(&self.path).join("calendar.txt");
        let iter: GTFSIterator<_, Calendar> = GTFSIterator::from_path(path.to_str().unwrap()).unwrap();
        for result in iter {
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
        if debug {
            println!("Loaded {} calendars", n);
        }

        let mut res = Vec::new();
        for stop_time in arrival_stop_times {
            let trip_id = stop_time.trip_id;
            if let Some(departure) = trip_ids.get(&trip_id) {
                let arrival = midnight + stop_time.arrival_time.duration();
                let departure = *departure;
                if arrival > departure {
                    if let Some(trip) = trips.get(&trip_id) {
                        if !services.contains_key(&trip.service_id) {
                            continue;
                        }
                        if let Some(route) = routes.get(&trip.route_id) {
                            let vehicule = match &route.route_type {
                                &RouteType::LightRail => "Tram".into(),
                                &RouteType::Subway => "Metro".into(),
                                &RouteType::Rail => "Rail".into(),
                                &RouteType::Bus => "Bus".into(),
                                &RouteType::Ferry => "Ferry".into(),
                                &RouteType::CableCar => "Cable car".into(),
                                &RouteType::Gondola => "Gondola".into(),
                                &RouteType::Funicular => "Funicular".into()
                            };
                            let short_name = route.route_short_name.clone();
                            let long_name = route.route_long_name.clone();
                            let service = Service {
                                departure,
                                arrival,
                                vehicule,
                                short_name,
                                long_name
                            };
                            res.push(service);
                        }
                    }
                }
            }
        }

        if debug {
            println!("");
        }
        res.sort_by(|a, b| a.departure.cmp(&b.departure));
        self.results = if res.len() > 0 { Some(res) } else { None };
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
    opts.optopt("g",  "gtfs",    "gtfs path",   "PATH");
    opts.optopt("f",  "from",    "depart from", "NAME");
    opts.optopt("t",  "to",      "arrive to",   "NAME");
    opts.optopt("a",  "at",      "depart at",   "TIME");
    //opts.optflag("d", "debug",   "enable debug output");
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
    if matches.opt_present("g") {
        if let Some(s) = matches.opt_str("g") {
            path = s;
        }
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

    let mut search = Search::new(path, from, to, at);
    
    search.run();
    println!("From: {}", search.from.unwrap_or("not found".into()));
    println!("To: {}", search.to.unwrap_or("not found".into()));
    match search.results {
        None => {
            println!("Results: not found");
        },
        Some(results) => {
            println!("Results:");
            for service in results.iter().take(5) {
                println!(
                    "  {} -> {} ({} {} - {})",
                    service.departure.format("%H:%M"),
                    service.arrival.format("%H:%M"),
                    service.vehicule,
                    service.short_name,
                    service.long_name
                );
            }
        }
    }
}
