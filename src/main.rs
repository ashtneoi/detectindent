extern crate neoilib;
extern crate num;

use neoilib::peek_while;
use num::Integer;
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

fn count_indents(filename: &str) -> io::Result<Vec<(u32, u32)>> {
    let file = File::open(filename)?;
    let mut lines = io::BufReader::new(file).lines();

    let mut counts = Vec::new();

    for _ in 0..100 {
        let maybe_line = (&mut lines).skip_while(|x| {
            match x {
                Ok(line) => !(
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

        let mut chars = line.chars().peekable();
        let tab_count = peek_while(&mut chars, |&x| { x == '\t' })
            .count() as u32;
        let sp_count = peek_while(&mut chars, |&x| { x == ' ' })
            .count() as u32;
        // TODO: chars.peek() == Some(&'\t') is probably an error. (Not sure
        // how best to return it, though)
        counts.push((tab_count, sp_count));
    }
    assert!(counts.len() > 0); // TODO: not a bug

    Ok(counts)
}

fn maybe_gcd(x: u32, y: u32) -> u32 {
    match x {
        0 => match y {
            0 => 0,
            yy => yy,
        },
        xx => match y {
            0 => xx,
            yy => xx.gcd(&yy),
        },
    }
}

fn detect_indent(counts: &Vec<(u32, u32)>) -> (u32, u32) {
    let mut tab_gcd = 0; // 0 means infinity, I guess
    let mut sp_gcd = 0;
    for &(tab_count, sp_count) in counts {
        tab_gcd = maybe_gcd(tab_gcd, tab_count);
        sp_gcd = maybe_gcd(sp_gcd, sp_count);
    }
    (tab_gcd, sp_gcd)
}

fn format_indent((tab_ind, sp_ind): (u32, u32), output_type: OutputType)
    -> String
{
    assert!(!(tab_ind == 0 && sp_ind == 0));
    // TODO: Figure out indent unit at runtime.
    // TODO: Don't assume tab+space tab width is 2*unit.
    let unit = 4;
    assert!(sp_ind.is_multiple_of(&unit)); // TODO: not a bug
    match output_type {
        OutputType::Generic => {
            let (kind, count) = match (tab_ind, sp_ind) {
                (ti, 0) => ("tab", (unit*ti).to_string()),
                (0, si) => ("space", si.to_string()),
                (ti, si) => ("tab+space", format!("{}+{}", 2*unit*ti, si)),
            };
            format!("kind={} count={}", kind, count)
        },
        OutputType::Vim => {
            let (expandtab, tabstop, shiftwidth) = match (tab_ind, sp_ind) {
                (ti, 0) => (false, unit*ti, unit*ti),
                (0, si) => (true, si, si),
                (ti, si) => (false, 2*unit*ti, si),
            };
            format!(
                "set {}expandtab tabstop={} shiftwidth={}",
                if expandtab { "" } else { "no" },
                tabstop,
                shiftwidth,
            )
        },
    }
}

fn main() {
    let owned_args: Vec<_> = env::args().skip(1).collect();
    let args: Vec<_> = owned_args.iter().map(|s| s.as_str()).collect();
    let (filename, output_type) = process_args(&args)
        .unwrap_or_else(|e| {
            eprintln!("error: {}", e);
            exit(2);
        });

    let counts = count_indents(&filename)
        .unwrap_or_else(|e| {
            eprintln!("error: {}", e);
            exit(2);
        });

    let indent = detect_indent(&counts);

    println!("{}", format_indent(indent, output_type));
}
