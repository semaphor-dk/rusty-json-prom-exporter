use serde_json::Value;
use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::{thread, time, process};

const FIVE_SECONDS: time::Duration = time::Duration::from_secs(5);
const THIRTY_SECONDS: time::Duration = time::Duration::from_secs(30);
const MAX_RETRIES: usize = 5;

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut data;

    if args.len() != 3 {
        eprintln!("Usage: rusty-json-prom-exporter [url] [filename]");
        std::process::exit(1);
    }

    loop {
        match send_request_with_retry(&args[1], MAX_RETRIES) {
            Ok(response_data) => {
                data = response_data;
            }
            Err(err) =>  {
                eprintln!("Failed to send the request ({})", err);
                process::exit(1); // Is this bad?
            }
        }
        let parsed_data: Value = match serde_json::from_str(&data) {
            Ok(data) => data,
            Err(err) => {
                eprintln!("Could not parse JSON data ({})", err);
                std::process::exit(1);
            }
        };
        let mut file = File::create(&args[2]).expect("Could not create file {&args[2]}");
        unpack_dict(&parsed_data, "", &mut file);
        thread::sleep(FIVE_SECONDS);
    }
}


fn send_request_with_retry(url: &str, max_retries: usize) -> Result<String, reqwest::Error> {
    let mut retries = 0;
    loop {
        match reqwest::blocking::get(url) {
            Ok(response) => {
                return response.text();
            }
            Err(_) => {
                retries += 1;
                if retries >= max_retries {
                    eprintln!("Exceeded maximum retries - Exiting.");
                    process::exit(1);
                }
                eprintln!("Connection refused - Retrying in 30 seconds... (Attempt {} of {})", retries, MAX_RETRIES);
                thread::sleep(THIRTY_SECONDS);
            }
        }
    }
}


fn unpack_dict(data: &Value, path: &str, file: &mut File) {
    
    /* Help-text should be somewhat customizable in
     * the future, maybe by including a dictionary
     * that can substitue values based on metric name? */

    let formatted_path = str::replace(path, "-", "_");
    match data {
        Value::Number(num) => {
            let line = format!(
                "# HELP {}\n# TYPE gauge\n{} {}\n",
                formatted_path, formatted_path, num
            );
            file.write_all(line.as_bytes()).unwrap();
        }
        Value::Bool(boolean) => {
            let numeric_bool: i8 = if *boolean { 1 } else { 0 };
            let line: String = format!(
                "# HELP {}_bool\n# TYPE gauge\n{} {}\n",
                formatted_path, formatted_path, numeric_bool
            );
            file.write_all(line.as_bytes()).unwrap();
        }
        Value::Object(map) => {
            for (key, value) in map {
                if path == "" {
                    let new_path = format!("{}{}", formatted_path, key);
                    unpack_dict(value, &new_path, file);
                } else {
                    let new_path = format!("{}_{}", formatted_path, key);
                    unpack_dict(value, &new_path, file);
                }
            }
        }
        _ => (), // Do not include in output, if not Value::Number or Value::Bool.
    }
}
