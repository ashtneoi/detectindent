extern crate neoilib;
extern crate num;

use neoilib::peek_while;
use num::Integer;
use std::env;
use std::fs::File;
use std::io::{self, BufRead};
use std::process::exit;

#[derive(Debug)]
enum OutputFormat {
    Generic,
    Vim,
}

fn exit_with_usage() -> ! {
    eprint!("\
        Usage:  indentdetect FILE FORMAT DEFTABWIDTH\n\
        \n\
        FORMAT: output format (`vim` or `generic`)\n\
        DEFTABWIDTH: default tab width\n\
    ");
    exit(2);
}

fn process_args<'a>(args: &[&'a str])
    -> Result<(&'a str, OutputFormat, u32), String>
{
    if args.len() == 0 {
        exit_with_usage();
    } else if args.len() < 3 {
        return Err("Too few arguments".to_string());
    } else if args.len() > 3 {
        return Err("Too many arguments".to_string());
    }

    let output_format = match args[1] {
        "generic" => OutputFormat::Generic,
        "vim" => OutputFormat::Vim,
        _ => { return Err("Invalid output format".to_string()); },
    };

    let def_tab_width = args[2].parse::<u32>()
        .map_err(|_| "Invalid default tab width".to_string())?;
    if def_tab_width == 0 {
        return Err("Default tab width can't be zero".to_string());
    }

    Ok((args[0], output_format, def_tab_width))
}

fn count_indents(filename: &str) -> io::Result<(bool, Vec<u32>)> {
    let file = File::open(filename)?;
    let mut lines = io::BufReader::new(file).lines();

    let mut tabs = false;
    let mut sp_counts = Vec::new();

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
        if peek_while(&mut chars, |&x| { x == '\t' }).count() > 0 {
            tabs = true;
        }
        let sp_count = peek_while(&mut chars, |&x| { x == ' ' })
            .count() as u32;
        // TODO: chars.peek() == Some(&'\t') is probably an error. (Not sure
        // how best to return it, though.)
        sp_counts.push(sp_count);
    }
    assert!(sp_counts.len() > 0); // TODO: not a bug

    Ok((tabs, sp_counts))
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

fn detect_indent(tabs: bool, sp_counts: &[u32], def_tab_width: u32)
    -> Result<(u32, u32), String>
{
    let mut sp_unit = 0; // 0 means infinity, I guess
    let mut max_sp = 0;
    for &sp_count in sp_counts {
        sp_unit = maybe_gcd(sp_unit, sp_count);
        max_sp = max_sp.max(sp_count);
    }

    let tab_width = match (tabs, sp_unit) {
        (true, 0) => def_tab_width,
        (true, _) => max_sp + sp_unit, // TODO: this is unreliable
        (false, _) => 0,
    };

    Ok((tab_width, sp_unit))
}

fn format_indent((tab_width, sp_unit): (u32, u32), output_format: OutputFormat)
    -> String
{
    assert!(!(tab_width == 0 && sp_unit == 0));
    match output_format {
        OutputFormat::Generic => {
            let (kind, count) = match (tab_width, sp_unit) {
                (t, 0) => ("tab", t.to_string()),
                (0, s) => ("space", s.to_string()),
                (t, s) => ("tab+space", format!("{} {}", t, s)),
            };
            format!("{} {}", kind, count)
        },
        OutputFormat::Vim => {
            let (expandtab, tabstop, shiftwidth) = match (tab_width, sp_unit) {
                (t, 0) => (false, t, t),
                (0, s) => (true, s, s),
                (t, s) => (false, t, s),
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
    let (filename, output_format, def_tab_width) = process_args(&args)
        .unwrap_or_else(|e| {
            eprintln!("error: {}", e);
            exit(2);
        });

    let (tabs, sp_counts) = count_indents(&filename)
        .unwrap_or_else(|e| {
            eprintln!("error: {}", e);
            exit(1);
        });

    let indent = detect_indent(tabs, &sp_counts, def_tab_width)
        .unwrap_or_else(|e| {
            eprintln!("error: {}", e);
            exit(1);
        });

    println!("{}", format_indent(indent, output_format));
}
