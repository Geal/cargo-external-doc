extern crate handlebars;
extern crate serde_json;
extern crate tempfile;

use std::fs;
use std::io::Write;
use std::collections::HashMap;
use std::process::Command;

use handlebars::Handlebars;
use serde_json::{Map, Value};
use tempfile::NamedTempFile;

fn get_package_name() -> String {
    let mut cargo = Command::new("cargo");
    cargo.arg("read-manifest");
    let stdout = String::from_utf8(cargo.output().expect("`cargo metadata` did not run").stdout)
                     .expect("invalid encoding");
    let value: Value = serde_json::from_str(&stdout).expect("invalid JSON metadata");
    let main: &Map<String, Value> = value.as_object().expect("top level value is not an object");
    String::from(main["name"].as_str().expect("invalid main string"))
}

fn generate_wrapper(name: &str) -> (NamedTempFile,NamedTempFile) {
  let handlebars = Handlebars::new();
  let before = include_str!("../templates/before.html");
  let after  = include_str!("../templates/after.html");

  let mut hash = HashMap::new();
  hash.insert(String::from("name"), String::from(name));

  let before_html = handlebars.template_render(before, &hash).expect("could not render the HTML prefix file");
  let after_html  = handlebars.template_render(after, &hash).expect("could not render the HTML suffix file");

  let mut before_file = NamedTempFile::new().expect("could not create temporary file");
  let mut after_file  = NamedTempFile::new().expect("could not create temporary file");

  before_file.write_all(before_html.as_bytes()).expect("could not write the HTML prefix file");
  before_file.flush().expect("could not write the HTML prefix file");
  after_file.write_all(after_html.as_bytes()).expect("could not write the HTML prefix file");
  after_file.flush().expect("could not write the HTML suffix file");

  (before_file, after_file)
}

///test
fn main() {
    let mut cargo = Command::new("cargo");
    cargo.arg("doc");
    let stdout = cargo.output().map(|o| o.stdout).expect("could not open cargo output");
    let result: String = String::from_utf8(stdout).expect("cargo command failed");
    //println!("cargo result: {}", result);

    let crate_name = get_package_name();
    let custom_doc_path = String::from("./target/doc/") + &crate_name;
    // FIXME: should handle the cargo env variables that override target folder
    // or get the target folder from cargo
    fs::create_dir_all(&custom_doc_path).expect("could not create directory");

    let (before_html, after_html) = generate_wrapper(&crate_name);
    let before_path = before_html.path();
    let after_path  = after_html.path();
    //println!("generating temporary HTML files at {:?} and {:?}", before_path, after_path);

    let doc_files = fs::read_dir("./doc").expect("could not read directory content");
    for file in doc_files {
        if let Ok(entry) = file {
            println!("generating doc from {:?}", entry.path());
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
            //println!("rustdoc --test result: {}", test_result);

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
            rustdoc.arg(before_path);
            rustdoc.arg("--html-after-content");
            rustdoc.arg(after_path);
            rustdoc.arg("-L");
            // FIXME: the debug folder has to be there, then :/
            rustdoc.arg("./target/debug/");
            rustdoc.arg("-v");
            let doc_stdout = rustdoc.output()
                                    .map(|o| o.stdout)
                                    .expect("could not open rustdoc output");
            let doc_result: String = String::from_utf8(doc_stdout).expect("rustdoc command failed");
            //println!("rustdoc result: {}", doc_result);
        }
    }
}

