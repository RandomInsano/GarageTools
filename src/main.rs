#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate rocket;
use rocket::State;
use std::collections::HashMap;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::io::SeekFrom;
use std::str::FromStr;

static SECURITY_CODE: &'static str = "APPLESAUCE-LOVIN";

static ON: &[u8] = b"1";
static OFF: &[u8] = b"0";

struct Relays {
    gpio_map: HashMap<String, File>,
}

impl Relays {
    fn new() -> Relays {
        let mut map = HashMap::new();
        for i in 0..7 {
            let mut file = OpenOptions::new()
                .read(true)
                .write(true)
                .create(true) // TODO: Remove once GPIO is used
                .open(i.to_string()).unwrap();

            file.write(OFF).unwrap();

            file.seek(SeekFrom::Start(0)).unwrap();    // TODO: Remove this too

            map.insert(i.to_string(), file);
        }

        Relays {
            gpio_map: map
        }
    }

    fn get(&self, index: u8) -> bool {
        let mut buffer = String::new();
        let index = index.to_string();
        self.gpio_map.get(&index).unwrap().read_to_string(&mut buffer).unwrap();

        FromStr::from_str(&buffer).unwrap()
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
fn index(relays: State<Relays>, endpoint: u8, value: bool) -> String {
    let mut out = String::new();
    out.push_str(SECURITY_CODE);
    out.push_str("-");
    out.push_str(&endpoint.to_string());

    relays.set(endpoint, value);

    // Dump current values
    for key in 0 .. 7 {
        let value = relays.get(key);

        out.push_str("\n GPIO: ");
        out.push_str(&key.to_string());
        out.push_str(": ");
        out.push_str(value.to_string().as_str());
    }

    out
}

fn main() {
    rocket::ignite()
        .mount("/", routes![index])
        .manage(Relays::new())
        .launch();
}
