use std::path::Path;
use std::fs::File;
use transitfeed::{GTFSIterator, Calendar, Route, Stop, StopTime, Trip};

pub struct GTFS {
    path: String
}

impl GTFS {
    pub fn from_path(path: String) -> GTFS {
        GTFS { path }
    }

    pub fn calendar(&self) -> GTFSIterator<File, Calendar> {
        let path = Path::new(&self.path).join("calendar.txt");
        GTFSIterator::from_path(path.to_str().unwrap()).unwrap()
    }

    pub fn routes(&self) -> GTFSIterator<File, Route> {
        let path = Path::new(&self.path).join("routes.txt");
        GTFSIterator::from_path(path.to_str().unwrap()).unwrap()
    }

    pub fn stops(&self) -> GTFSIterator<File, Stop> {
        let path = Path::new(&self.path).join("stops.txt");
        GTFSIterator::from_path(path.to_str().unwrap()).unwrap()
    }

    pub fn stop_times(&self) -> GTFSIterator<File, StopTime> {
        let path = Path::new(&self.path).join("stop_times.txt");
        GTFSIterator::from_path(path.to_str().unwrap()).unwrap()
    }

    pub fn trips(&self) -> GTFSIterator<File, Trip> {
        let path = Path::new(&self.path).join("trips.txt");
        GTFSIterator::from_path(path.to_str().unwrap()).unwrap()
    }
}
