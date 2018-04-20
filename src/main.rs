extern crate num;

use std::env;
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

fn main() {
    let owned_args: Vec<_> = env::args().skip(1).collect();
    let args: Vec<_> = owned_args.iter().map(|s| s.as_str()).collect();
    let (filename, output_type) = process_args(&args)
        .unwrap_or_else(|e| {
            eprintln!("{}", e);
            exit(2);
        });
    println!("{} -> {:?}", filename, output_type);
}
