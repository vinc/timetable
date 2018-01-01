Timetable
=========

Construct timetables from General Transit Feed Specification (GTFS)


Installation
------------

First you need to install Rust:

    $ curl https://sh.rustup.rs -sSf | sh

Then you can install the latest stable version with cargo:

    $ cargo install timetable

Or the development version by fetching the git repository:

    $ git clone git://github.com/vinc/timetable.git
    $ cd timetable
    $ cargo install


Usage
-----

Download and extract a gtfs zip file:

    $ timetable \
      --path ~/tmp/gtfs/transilien-sncf \
      --url http://transitfeeds.com/p/transilien-sncf/207/latest/download

Search a station:

    $ timetable \
      --path ~/tmp/gtfs/transilien-sncf
      --from "font" \
    Stations
    FERRIERES FONTENAY
    FONTAINE LE PORT
    FONTAINE MICHALON
    FONTAINEBLEAU AVON
    FONTENAY AUX ROSES
    FONTENAY LE FLEURY
    FONTENAY SOUS BOIS
    PORCHEFONTAINE
    VAL DE FONTENAY

Print timetable:

    $ timetable \
      --path ~/tmp/gtfs/transilien-sncf
      --from "fontainebleau" \
      --to "gare de lyon" \
      --at "2017-12-21 08:00:00" \
    Departures   Arrivals   Routes
    08:13 ......... 08:54   RER R - Montargis / Gare de Lyon
    09:06 ......... 09:47   RER R - Montargis / Gare de Lyon
    10:03 ......... 10:44   RER R - Montargis / Gare de Lyon
    11:03 ......... 11:44   RER R - Montargis / Gare de Lyon
    12:03 ......... 12:44   RER R - Montargis / Gare de Lyon


License
-------

Copyright (c) 2018 Vincent Ollivier. Released under MIT.
