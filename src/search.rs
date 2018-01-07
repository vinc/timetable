use colored::Colorize;
use chrono::prelude::*;
use std::collections::HashMap;
use gtfs::GTFS;
use transitfeed;

#[derive(PartialEq)]
enum Step {
    Departure,
    Arrival
}

pub struct Search {
    gtfs: GTFS,

    origins: Vec<String>,
    destinations: Vec<String>,
    stop_ids: HashMap<String, Step>,
    departure_stop_times: Vec<transitfeed::StopTime>,
    arrival_stop_times: Vec<transitfeed::StopTime>,

    route_ids: HashMap<String, bool>,
    routes: HashMap<String, transitfeed::Route>,
    service_ids: HashMap<String, bool>,
    services: HashMap<String, bool>,
    trip_ids: HashMap<String, DateTime<Local>>,
    trips: HashMap<String, transitfeed::Trip>,

    pub debug: bool
}

impl Search {
    pub fn new(path: String) -> Search {
        Search {
            gtfs: GTFS::from_path(path),

            origins: Vec::new(),
            destinations: Vec::new(),
            departure_stop_times: Vec::new(),
            arrival_stop_times: Vec::new(),

            route_ids: HashMap::new(),
            routes: HashMap::new(),
            service_ids: HashMap::new(),
            services: HashMap::new(),
            stop_ids: HashMap::new(),
            trip_ids: HashMap::new(),
            trips: HashMap::new(),

            debug: false
        }
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

    pub fn timetable(&mut self, from: &str, to: &str, at: DateTime<Local>) -> Vec<Service> {
        self.search_stops(from, to);
        self.search_stop_times();
        self.search_departures(at);
        self.search_trips();
        self.search_routes();
        self.search_calendar(at);
        self.search_services(at)
    }

    // Search stop candidates for origin and destination
    fn search_stops(&mut self, from: &str, to: &str) {
        let mut n = 0;
        for result in self.gtfs.stops() {
            n += 1;
            if let Ok(entry) = result {
                let name = entry.stop_name.to_lowercase();

                if name.contains(from) {
                    self.stop_ids.insert(entry.stop_id, Step::Departure);
                    self.origins.push(entry.stop_name);
                } else if name.contains(to) {
                    self.stop_ids.insert(entry.stop_id, Step::Arrival);
                    self.destinations.push(entry.stop_name);
                }
            }
        }
        if self.debug {
            self.origins.sort();
            self.origins.dedup();
            self.destinations.sort();
            self.destinations.dedup();
            println!("{}: origins: {}", "Debug".cyan(), self.origins.join(", "));
            println!("{}: destinations: {}", "Debug".cyan(), self.destinations.join(", "));
            println!("{}: loaded {} stops ({} retained)", "Debug".cyan(), n, self.stop_ids.len());
        }
    }

    // Get stop times
    fn search_stop_times(&mut self) {
        let mut n = 0;
        for result in self.gtfs.stop_times() {
            n += 1;
            if let Ok(entry) = result {
                match self.stop_ids.get(&entry.stop_id) {
                    Some(&Step::Departure) => self.departure_stop_times.push(entry),
                    Some(&Step::Arrival) => self.arrival_stop_times.push(entry),
                    _ => ()
                }
            }
        }
        if self.debug {
            let total = self.departure_stop_times.len() + self.arrival_stop_times.len();
            println!("{}: loaded {} stop times ({} retained)", "Debug".cyan(), n, total);
        }
    }

    // Get trips from origin stop candidates starting after the given time
    fn search_departures(&mut self, at: DateTime<Local>) {
        let midnight = at.date().and_hms(0, 0, 0);

        for stop_time in self.departure_stop_times.iter() {
            let departure = midnight + stop_time.departure_time.duration();
            if departure > at {
                let trip_id = stop_time.trip_id.clone();
                self.trip_ids.insert(trip_id, departure);
            }
        }
    }

    // Get routes and services from trips
    fn search_trips(&mut self) {
        let mut n = 0;
        for result in self.gtfs.trips() {
            n += 1;
            if let Ok(entry) = result {
                let trip_id = entry.trip_id.clone();
                let route_id = entry.route_id.clone();
                let service_id = entry.service_id.clone();
                if self.trip_ids.contains_key(&trip_id) {
                    self.service_ids.insert(service_id, true);
                    self.route_ids.insert(route_id, true);
                    self.trips.insert(trip_id, entry);
                }
            }
        }
        if self.debug {
            println!("{}: loaded {} trips ({} retained)", "Debug".cyan(), n, self.trips.len());
        }
    }

    // Get routes from their ids
    fn search_routes(&mut self) {
        let mut n = 0;
        for result in self.gtfs.routes() {
            n += 1;
            if let Ok(entry) = result {
                let route_id = entry.route_id.clone();
                if self.route_ids.contains_key(&route_id) {
                    self.routes.insert(route_id, entry);
                }
            }
        }
        if self.debug {
            println!("{}: loaded {} routes ({} retained)", "Debug".cyan(), n, self.routes.len());
        }
    }

    // Get services from their ids that are running on the given date
    fn search_calendar(&mut self, at: DateTime<Local>) {
        let date = at.date().naive_local();

        let mut n = 0;
        for result in self.gtfs.calendar() {
            n += 1;
            if let Ok(entry) = result {
                let service_id = entry.service_id;
                if self.service_ids.contains_key(&service_id) {
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
                        self.services.insert(service_id, true);
                    }
                }
            }
        }
        if self.debug {
            println!("{}: loaded {} services ({} retained)", "Debug".cyan(), n, self.services.len());
        }
    }

    // Filter trips based on remaining services and routes, that also connect
    // origin with destination stop candidates.
    fn search_services(&mut self, at: DateTime<Local>) -> Vec<Service> {
        let midnight = at.date().and_hms(0, 0, 0);
        let mut results = Vec::new();
        for stop_time in self.arrival_stop_times.iter() {
            let trip_id = stop_time.trip_id.clone();
            if let Some(departure) = self.trip_ids.get(&trip_id) {
                let arrival = midnight + stop_time.arrival_time.duration();
                let departure = *departure;
                if arrival > departure {
                    if let Some(trip) = self.trips.get(&trip_id) {
                        if !self.services.is_empty() && !self.services.contains_key(&trip.service_id) {
                            continue;
                        }
                        if let Some(route) = self.routes.get(&trip.route_id) {
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

#[derive(PartialEq, PartialOrd, Eq, Ord)]
pub struct Station {
    pub name: String
}

pub struct Service {
    pub departure: DateTime<Local>,
    pub arrival: DateTime<Local>,
    pub short_name: String,
    pub long_name: String
}

impl Service {
    pub fn name(&self) -> String {
        let short = self.short_name.clone();
        let long = self.long_name.clone();
        let mut parts = vec![short, long];
        parts.retain(|s| !s.is_empty());
        parts.join(" - ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timetable() {
        let time = Local::now();
        let naive_time = NaiveDateTime::parse_from_str("2017-12-21 07:30:00", "%Y-%m-%d %H:%M:%S").unwrap();
        let time = time.timezone().from_local_datetime(&naive_time).earliest().unwrap();
        let mut search = Search::new("examples/data/good_feed".into());
        let results = search.timetable("airport", "bullfrog", time);

        assert_eq!(results.len(), 1);
        for service in results.iter() {
            assert_eq!(service.departure.format("%H:%M").to_string(), String::from("08:00"));
            assert_eq!(service.arrival.format("%H:%M").to_string(), String::from("08:10"));
            assert_eq!(service.name(), "Airport ⇒ Bullfrog");
        }
    }
}
