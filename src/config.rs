use std::{fs::File, io::Read, path::Path};

use serde::Deserialize;

#[derive(Deserialize)]
pub struct Config {
    pub input_path: String,
    pub output_path: String,
    pub delimiter: char,
    pub row_filter: String
}

impl Config {
    pub fn load<P: AsRef<Path>>(filename: P) -> Config {
        let mut infile = File::open(filename)
            .expect("Nem talalhato a konfiguracios fajl (config.toml)");

        let mut buf: Vec<u8> = vec![];
        infile.read_to_end(&mut buf)
            .expect("Nem sikerult beolvasni a konfiguracios fajlt");

        toml::from_slice(&buf)
            .expect("Nem megfelelo formatumu konfiguracios fajl")
    }
}
