use anyhow::Result;
use getopts::Options;
use regex::Regex;
use rust_stemmers::{Algorithm, Stemmer};
use std::borrow::Cow;
use std::collections::{HashMap, HashSet};
use std::fs::OpenOptions;
use std::io::{BufWriter, Write};

struct Config {
    file: String,
    lower: bool,
    stem: bool,
    proper: bool,
    stop_word: bool,
    output: Box<dyn Write>,
}

fn main() {
    let mut config = parse_args().unwrap();
    let data = std::fs::read_to_string(config.file).unwrap();
    let re = Regex::new(r"[^\w\s\d']|[_]").unwrap();

    // Stopwords
    let stop = if config.stop_word {
        stop_words::get(stop_words::LANGUAGE::English)
            .into_iter()
            .map(|x| re.replace_all(&x, "").to_string()) // For some reason they are literally quoted
            .collect()
    } else {
        HashSet::new()
    };

    // Replace punctuation, miscellaneous symbols
    let mut data = match re.replace_all(&data, "") {
        Cow::Borrowed(x) => x.to_string(),
        Cow::Owned(x) => x,
    };
    let proven_lower: HashSet<_> = data
        .split_whitespace()
        .filter(|x| {
            x.chars()
                .nth(0)
                .map(|x| x.is_ascii_lowercase() || x.is_ascii_digit())
                .unwrap_or(true)
        })
        .map(|x| x.to_string())
        .collect();
    // Lowercase
    if config.lower {
        data.make_ascii_lowercase();
    }
    let stemmer = Stemmer::create(Algorithm::English);
    let tokens = data.split_whitespace().filter(|&word| !stop.contains(word));
    // Stemming
    let tokens: Vec<_> = if config.stem {
        tokens.map(|x| stemmer.stem(x)).collect()
    } else {
        tokens.map(|x| Cow::Borrowed(x)).collect() // Keep types the same
    };
    println!("Token count: {}", tokens.len());
    let mut counts: HashMap<_, u32> = HashMap::new();
    for token in tokens.iter() {
        *counts.entry(token).or_default() += 1;
    }
    let mut output: Vec<_> = counts.iter().map(|(k, v)| (*v, *k)).collect();
    output.sort_by(|a, b| b.cmp(a));

    // Filter only proper nouns
    if config.proper {
        output.retain(|(c, word)| *c > 1 && !proven_lower.contains(word.as_ref()));
    }
    for (c, word) in output.iter() {
        writeln!(&mut config.output, "{} {}", word, c).expect("Failed to write to output");
    }
    let counts = output.iter().map(|(x, _)| x).copied().collect();
    plot(counts);
}

fn plot(output: Vec<u32>) {
    use plotly::common::Title;
    use plotly::layout::{Axis, AxisType, Layout};
    use plotly::{Plot, Scatter};
    let layout = Layout::new()
        .title(Title::new("Token Analysis"))
        .x_axis(Axis::new().title("Rank".into()).type_(AxisType::Log))
        .y_axis(Axis::new().title("Token Count".into()).type_(AxisType::Log));
    let mut plot = Plot::new();
    let xs: Vec<usize> = (1..=output.len()).collect();
    let max = output[0];
    let trace = Scatter::new(xs.clone(), output).name("Word Frequency");
    let zipf = (1..=xs.len()).map(|x| (max as f64) / (x as f64)).collect();
    let zipf_trace = Scatter::new(xs, zipf).name("Zipf's Law");
    plot.add_traces(vec![trace, zipf_trace]);
    plot.set_layout(layout);
    plot.write_html("plot.html");
}

fn parse_args() -> Result<Config> {
    let args: Vec<_> = std::env::args().collect();
    let program = args[0].clone();
    let mut opts = Options::new();
    opts.optflag("h", "help", "Print this help menu");
    opts.optflag("l", "lower", "Lowercases the input text");
    opts.optflag("s", "stem", "Stems the input text");
    opts.optflag("t", "stop", "Filters out stop words");
    opts.optflag("p", "proper", "Filters for suspected proper nouns");
    opts.optopt(
        "o",
        "output",
        "File path to output to. If empty, write to stdout",
        "-o FILE",
    );
    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => {
            panic!("{}", f);
        }
    };
    if matches.opt_present("h") {
        print_usage(&program, opts);
        std::process::exit(0);
    }
    anyhow::ensure!(
        !matches.free.is_empty(),
        "Unable to locate input file in arguments"
    );
    let out_writer: Box<dyn Write> = match matches.opt_str("o") {
        Some(x) => {
            let path = std::path::Path::new(&x);
            let writer = BufWriter::new(
                OpenOptions::new()
                    .write(true)
                    .create(true)
                    .open(path)
                    .expect("Unable to create out file"),
            );
            Box::new(writer)
        }
        _ => Box::new(std::io::stdout()),
    };
    let file = matches.free[0].clone();
    let proper = matches.opt_present("p");
    let stop_word = matches.opt_present("t");
    Ok(Config {
        file,
        proper,
        stop_word,
        output: out_writer,
        lower: matches.opt_present("l"),
        stem: matches.opt_present("s"),
    })
}

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} FILE [options]", program);
    print!("{}", opts.usage(&brief));
}
