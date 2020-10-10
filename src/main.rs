use clap::{App, Arg};
use notify::{watcher, RecursiveMode, Watcher};
use std::fs;
use std::path::Path;
use std::time::Duration;
use std::{thread, time};
use wkhtmltopdf::*;

mod markdown;
use markdown::*;

fn compile(path: &str, output: &str, header: &String, footer: &String, pdf: bool) {
    let path = Path::new(path);

    println!("{:?}", path.file_name().unwrap());

    if !path.exists() {
        panic!("Invalid input file");
    }

    let html = if path.is_dir() {
        let mut parsed = String::new();

        let mut paths: Vec<_> = fs::read_dir(path).unwrap().map(|r| r.unwrap()).collect();
        paths.sort_by_key(|dir| dir.path());

        for entry in paths {
            let entry_path = entry.path();

            if entry_path.is_dir() {
                continue;
            }

            if entry_path.extension().unwrap() == "md" {
                println!("\t{:?}", entry_path.file_name().unwrap());
                let markdown = fs::read_to_string(entry_path).unwrap();
                let mut parser = Parser::new(&markdown);
                parsed += &parser.to_html();
            }
        }

        parsed
    } else {
        let markdown = fs::read_to_string(path).unwrap();
        let mut parser = Parser::new(&markdown);
        parser.to_html()
    };

    let mut result = String::from(header);
    result.push_str(html.as_str());
    result.push_str(&footer);

    if pdf {
        let mut pdf_app = PdfApplication::new().expect("Failed to init PDF application");
        let mut pdfout = pdf_app
            .builder()
            .orientation(Orientation::Portrait)
            .margin(Size::Millimeters(25))
            .title("Rabbit Output")
            .build_from_html(&result)
            .expect("Failed to build pdf");
        pdfout.save(output).expect("Failed to save pdf file");
    } else {
        fs::write(output, result).unwrap();
    }
}

fn main() {
    let matches = App::new("Markdown Parser")
        .version("1.0")
        .author("Hector Peeters")
        .about("Convert Markdown files into HTML!")
        .arg(Arg::with_name("input").required(true).index(1))
        .arg(Arg::with_name("output").short("o").takes_value(true))
        .arg(Arg::with_name("pdf").short("p").takes_value(false))
        .arg(Arg::with_name("header").short("h").takes_value(true))
        .arg(Arg::with_name("footer").short("f").takes_value(true))
        .arg(Arg::with_name("watcher").short("w").takes_value(false))
        .get_matches();

    let header: String = match matches.value_of("header") {
        Some(x) => fs::read_to_string(x).expect("Failed to read header file"),
        None => String::from(include_str!("header.html")),
    };
    let footer: String = match matches.value_of("footer") {
        Some(x) => fs::read_to_string(x).expect("Failed to read footer file"),
        None => String::from(include_str!("footer.html")),
    };

    let input_file = matches.value_of("input").unwrap();
    let output_file = matches.value_of("output").unwrap_or("index.html");

    compile(
        input_file,
        output_file,
        &header,
        &footer,
        matches.is_present("pdf"),
    );

    if matches.is_present("watcher") {
        let (tx, rx) = std::sync::mpsc::channel();

        let mut watcher = watcher(tx, Duration::from_secs(10)).unwrap();

        loop {
            watcher
                .watch(input_file, RecursiveMode::NonRecursive)
                .unwrap();

            match rx.recv() {
                Ok(_) => {
                    thread::sleep(time::Duration::from_millis(100));
                    compile(
                        input_file,
                        output_file,
                        &header,
                        &footer,
                        matches.is_present("pdf"),
                    );
                    println!("Recompiled {}", input_file);
                }
                Err(err) => println!("watch error: {:?}", err),
            }
        }
    }
}
