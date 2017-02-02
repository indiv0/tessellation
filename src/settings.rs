use rustc_serialize::Encodable;
use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Write};
use toml::{Decoder, Encoder, Parser, Value};
use rustc_serialize::Decodable;

const SETTINGS_FILENAME: &'static str = ".xplicit";

pub fn show_settings_dialog() {
    SettingsData::new();
}

#[derive(RustcDecodable, RustcEncodable)]
pub struct SettingsData {
    tessellation_resolution: f64,
}

fn join<S: ToString>(l: Vec<S>, sep: &str) -> String {
    l.iter().fold("".to_string(), |a, b| {
        if a.len() > 0 {
            a + sep
        } else {
            a
        }
    } + &b.to_string())
}

#[derive(Debug)]
enum SettingsError {
    Io(::std::io::Error),
    Dec(::toml::DecodeError),
    Enc(::toml::Error)
}

impl SettingsData {
    fn path() -> Result<::std::path::PathBuf, SettingsError> {
        let mut path = match ::std::env::home_dir() {
            Some(p) => p,
            None => try!(::std::env::current_dir().map_err(SettingsError::Io))
        };
        path.push(SETTINGS_FILENAME);
        Ok(path)
    }
    fn get_toml() -> Result<Self, SettingsError> {
        let path = try!(SettingsData::path());
        let f = try!(File::open(path).map_err(SettingsError::Io));
        let mut reader = BufReader::new(f);
        let mut buffer = String::new();
        let _ = try!(reader.read_to_string(&mut buffer).map_err(SettingsError::Io));
        let mut parser = Parser::new(&buffer);
        match parser.parse() {
            Some(value) => {
                let mut decoder = Decoder::new(Value::Table(value));
                let settings = try!(Decodable::decode(&mut decoder).map_err(SettingsError::Dec));
                Ok(settings)
            },
            None => {
                Err(SettingsError::Io(::std::io::Error::new(::std::io::ErrorKind::InvalidInput,
                                          join(parser.errors
                                                     .iter()
                                                     .map(|e| format!("{}", e))
                                                     .collect(),
                                               ", "))))
            }
        }
    }
    pub fn new() -> SettingsData {
        match SettingsData::get_toml() {
            Ok(c) => c,
            Err(e) => {
                println!("error reading settings: {:?}", e);
                SettingsData { tessellation_resolution: 0.12 }
            }
        }
    }

    fn put_toml(&mut self) -> Result<(), SettingsError> {
        let mut e = Encoder::new();
        try!(self.encode(&mut e).map_err(SettingsError::Enc));
        let path = try!(SettingsData::path());
        let file = try!(File::create(path).map_err(SettingsError::Io));
        let mut writer = BufWriter::new(file);
        try!(writer.write(format!("{}", Value::Table(e.toml)).as_bytes()).map_err(SettingsError::Io));
        Ok(())
    }
}

impl ::std::ops::Drop for SettingsData {
    fn drop(&mut self) {
        match self.put_toml() {
            Ok(_) => {},
            Err(e) => println!("error writing settings: {:?}", e),
        }
    }
}