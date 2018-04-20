use std::env;
use std::fs::File;
use std::io::{self, BufRead};
use std::process::exit;

#[derive(Debug)]
enum OutputType {
    Generic,
    Vim,
}

fn exit_with_usage() -> ! {
    eprint!("\
        Usage:  indentdetect FILE OUTPUT-TYPE\n\
        \n\
        OUTPUT-TYPE: output type (`vim` or `generic`)\n\
    ");
    exit(2);
}

fn process_args<'a>(args: &[&'a str])
    -> Result<(&'a str, OutputType), String>
{
    if args.len() == 0 {
        exit_with_usage();
    } else if args.len() < 2 {
        return Err("Too few arguments".to_string());
    } else if args.len() > 2 {
        return Err("Too many arguments".to_string());
    }

    let output_type = match args[1] {
        "generic" => OutputType::Generic,
        "vim" => OutputType::Vim,
        _ => { return Err("Invalid output type".to_string()); },
    };

    Ok((args[0], output_type))
}

fn read_initial_ws(filename: &str) -> io::Result<Vec<(i32, i32)>> {
    let file = File::open(filename)?;
    let mut lines = io::BufReader::new(file).lines();

    for _ in 0..100 {
        let maybe_line = (&mut lines).skip_while(|x| {
            match x {
                Ok(ref line) => !(
                    line.starts_with('\t') || line.starts_with(' ')
                ),
                Err(_) => false,
            }
        }).next();
        let line = match maybe_line {
            Some(Ok(line)) => line,
            Some(Err(e)) => return Err(e),
            None => break,
        };
        println!("{}", &line);
    }

    Ok(vec![])
}

fn main() {
    let owned_args: Vec<_> = env::args().skip(1).collect();
    let args: Vec<_> = owned_args.iter().map(|s| s.as_str()).collect();
    let (filename, output_type) = process_args(&args)
        .unwrap_or_else(|e| {
            eprintln!("error: {}", e);
            exit(2);
        });
    println!("{} -> {:?}", filename, output_type);

    let initial_ws = read_initial_ws(&filename)
        .unwrap_or_else(|e| {
            eprintln!("error: {}", e);
            exit(2);
        });
}
