use std::env;
use std::fs::File;
use std::io::{self, BufRead};
use std::iter::Peekable;
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

struct PeekWhile<'a, I, P>
where
    I: 'a + Iterator,
    P: FnMut(&I::Item) -> bool,
{
    iter: &'a mut Peekable<I>,
    predicate: P,
}

impl<'a, I, P> Iterator for PeekWhile<'a, I, P>
where
    I: Iterator,
    P: FnMut(&I::Item) -> bool,
{
    type Item = I::Item;

    fn next(&mut self) -> Option<I::Item> {
        let take = match self.iter.peek() {
            Some(ref x) => Some((self.predicate)(x)),
            None => None,
        };
        match take {
            Some(true) => Some(self.iter.next().unwrap()),
            _ => None,
        }
    }
}

fn peek_while<'a, I, P>(iter: &'a mut Peekable<I>, predicate: P)
    -> PeekWhile<I, P>
where
    I: Iterator,
    P: FnMut(&I::Item) -> bool,
{
    PeekWhile { iter: iter, predicate }
}

fn count_indentation(filename: &str) -> io::Result<Vec<(u32, u32)>> {
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
        counts.push((tab_count, sp_count));
    }

    Ok(counts)
}

fn main() {
    let owned_args: Vec<_> = env::args().skip(1).collect();
    let args: Vec<_> = owned_args.iter().map(|s| s.as_str()).collect();
    let (filename, output_type) = process_args(&args)
        .unwrap_or_else(|e| {
            eprintln!("error: {}", e);
            exit(2);
        });

    let counts = count_indentation(&filename)
        .unwrap_or_else(|e| {
            eprintln!("error: {}", e);
            exit(2);
        });
    println!("{:?}", counts);
}
