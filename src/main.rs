//! hello!
//! reference to [testdoc](articles/testdoc.html)
extern crate serde_json;

use std::fs;
use std::process::Command;

use serde_json::{Map, Value};

fn get_package_name() -> String {
    let mut cargo = Command::new("cargo");
    cargo.arg("metadata");
    let stdout = String::from_utf8(cargo.output().expect("`cargo metadata` did not run").stdout)
                     .expect("invalid encoding");
    let value: Value = serde_json::from_str(&stdout).expect("invalid JSON metadata");
    let main: &Map<String, Value> = value.as_object().expect("top level value is not an object");
    String::from(main["name"].as_str().expect("invalid main string"))
}

///test
fn main() {
    let mut cargo = Command::new("cargo");
    cargo.arg("doc");
    let stdout = cargo.output().map(|o| o.stdout).expect("could not open cargo output");
    let result: String = String::from_utf8(stdout).expect("cargo command failed");
    println!("cargo result: {}", result);

    let crate_name = get_package_name();
    let custom_doc_path = String::from("./target/doc/") + &crate_name;
    // FIXME: should handle the cargo env variables that override target folder
    // or get the target folder from cargo
    fs::create_dir_all(&custom_doc_path).expect("could not create directory");

    let doc_files = fs::read_dir("./doc").expect("could not read directory content");
    for file in doc_files {
        if let Ok(entry) = file {
            println!("will generate doc from {:?}", entry);
            let mut test = Command::new("rustdoc");
            test.arg(entry.path());
            test.arg("--test");
            test.arg("-L");
            // FIXME: the debug folder has to be there, then :/
            test.arg("./target/debug/");
            let test_stdout = test.output()
                                  .map(|o| o.stdout)
                                  .expect("could not open rustdoc output");
            let test_result: String = String::from_utf8(test_stdout)
                                          .expect("rustdoc command failed");
            println!("rustdoc --test result: {}", test_result);

            let mut rustdoc = Command::new("rustdoc");
            rustdoc.arg(entry.path());
            rustdoc.arg("--crate-name");
            rustdoc.arg(&crate_name);
            rustdoc.arg("-o");
            rustdoc.arg(&custom_doc_path);
            rustdoc.arg("--markdown-css");
            rustdoc.arg("../rustdoc.css");
            rustdoc.arg("--markdown-css");
            rustdoc.arg("../main.css");
            rustdoc.arg("--html-before-content");
            // FIXME: generate the before.html file with a template in a tmp file, then get its path
            rustdoc.arg("./before.html");
            rustdoc.arg("--html-after-content");
            rustdoc.arg("./after.html");
            rustdoc.arg("-L");
            // FIXME: the debug folder has to be there, then :/
            rustdoc.arg("./target/debug/");
            rustdoc.arg("-v");
            let doc_stdout = rustdoc.output()
                                    .map(|o| o.stdout)
                                    .expect("could not open rustdoc output");
            let doc_result: String = String::from_utf8(doc_stdout).expect("rustdoc command failed");
            println!("rustdoc result: {}", doc_result);
        }
    }
}

/// Hi
pub fn hello() {}
