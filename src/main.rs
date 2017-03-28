#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate rocket;

use rocket::request::FromParam;
use rocket::State;
use std::collections::HashMap;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::io::SeekFrom;
use std::str::FromStr;
use std::{thread, time};

/// Lookup table built from this page:
/// http://www.chip-community.org/index.php/GPIO_Info
///
/// We're physically using CSID0-7 to map to the relay ports 0-7, and these are starting at
/// the sysfs export point of 132.

//const GPIO_PATH: &str = "/sys/class/gpio/gpio{0}/value"; // Re-typed below because Rust is dumb
static GPIO: &'static [&'static str] = &[
    "132",
    "133",
    "134",
    "135",
    "136",
    "137",
    "138",
    "139",
];

static SECURITY_CODE: &'static str = "APPLESAUCE-LOVIN";
static ON: &[u8] = b"1";
static OFF: &[u8] = b"0";
static SLEEP_TIME: u64 = 100;

enum RelayCommand {
    SETSTATE(bool),
    CYCLE   // Tells us to cycle the relay on and off
}

struct Relays {
    gpio_map: HashMap<String, File>,
}

impl<'r> FromParam<'r> for RelayCommand {
    type Error = &'r str;

    fn from_param(param: &'r str) -> Result<RelayCommand, &'static str> {
        match param {
            "cycle" => Ok(RelayCommand::CYCLE),
            "true" => Ok(RelayCommand::SETSTATE(true)),
            "false" => Ok(RelayCommand::SETSTATE(false)),
            _ => Err("Unable to parse")
        }
    }
}

impl Relays {
    fn new() -> Relays {
        let mut map = HashMap::new();
        for i in 0..8 {
            let filename = format!("/sys/class/gpio/gpio{0}/value", GPIO[i]);
            let mut file = OpenOptions::new()
                .read(true)
                .write(true)
                .create(false)
                .open(&filename).unwrap();

            file.write(OFF).unwrap();

            file.seek(SeekFrom::Start(0)).unwrap();    // TODO: Remove this too

            map.insert(i.to_string(), file);

            println!("Opened {} for write", filename);
        }

        Relays {
            gpio_map: map
        }
    }

    fn get(&self, index: u8) -> bool {
        let mut buffer = String::new();
        let index = index.to_string();
        self.gpio_map
		.get(&index).unwrap()
		.read_to_string(&mut buffer).unwrap();

        //FromStr::from_str(&buffer).unwrap()
	false
    }

    fn set(&self, index: u8, value: bool) {
        let index = index.to_string();

        if let Some(mut x) = self.gpio_map.get(&index) {
            if value {
                x.write(ON).unwrap();
            } else {
                x.write(OFF).unwrap();
            }

            x.seek(SeekFrom::Start(0)).unwrap();    // TODO: Remove this too
        }
    }
}


#[get("/relays/<endpoint>/<value>")]
fn relays(relays: State<Relays>, endpoint: u8, value: RelayCommand) -> String {
    let mut out = String::new();
    out.push_str(SECURITY_CODE);
    out.push_str("-");
    out.push_str(&endpoint.to_string());

    match value {
        RelayCommand::SETSTATE(x) => relays.set(endpoint, x),
        RelayCommand::CYCLE => {
            relays.set(endpoint, true);
            thread::sleep(time::Duration::from_millis(SLEEP_TIME));
            relays.set(endpoint, false);
        }
    }

    out.push_str("\n Value:");
    out.push_str(&relays.get(endpoint).to_string());

    out
}

fn main() {
    rocket::ignite()
        .mount("/", routes![relays])
        .manage(Relays::new())
        .launch();
}
