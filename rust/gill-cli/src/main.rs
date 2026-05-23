use gill_cli::{run_file, run_string};
use std::env;
use std::path::Path;

fn main() {
    let args: Vec<String> = env::args().collect();
    let banner = "Gill CLI v0.1.0 - A hybrid C/Rust/Lisp-inspired language";
    println!("{}", banner);

    match args.as_slice() {
        [_, path] => {
            let path = Path::new(path);
            match run_file(path) {
                Ok(_) => println!("Execution completed."),
                Err(err) => eprintln!("Gill runtime error: {err}"),
            }
        }
        [_] | [] => {
            let sample = r#"NAME=\"hello\"
            DESC=\"Basic Gill sample\"

            func main() {
              print.console(\"Hello from Gill!\");
            }
            "#;
            if let Err(err) = run_string(sample) {
                eprintln!("Gill runtime error: {err}");
            }
        }
        _ => {
            eprintln!("Usage: gill-cli <script.gill>");
        }
    }
}
