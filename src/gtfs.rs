use std::error::Error;
use std::io::prelude::*;
use std::fs::File;
use std::path::Path;
use transitfeed::{GTFSIterator, Calendar, Route, Stop, StopTime, Trip};
use zip::ZipArchive;

pub fn unzip_to_path(archive: String, path: String) {
    let reader = File::open(&archive).unwrap();
    let mut zip = ZipArchive::new(reader).unwrap();
    for i in 0..zip.len() {
    let mut file = zip.by_index(i).unwrap();
        println!("{}", file.name());
        let filename = Path::new(&path).join(file.name());
        let mut f = File::create(&filename).unwrap();
        let mut content = Vec::new();
        if let Err(why) = file.read_to_end(&mut content) {
            panic!("Error: could not read '{}' in '{}': {}", file.name(), archive, why.description());
        }
        if let Err(why) = f.write_all(&content) {
            panic!("Error: could not write '{}': {}", filename.display(), why.description());
        }
    }
}

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
