use anyhow::Result;
use getopts::Options;
use plotly::layout::AxisType;
use regex::Regex;
use rust_stemmers::{Algorithm, Stemmer};
use std::borrow::Cow;
use std::collections::{HashMap, HashSet};

struct Config {
    file: String,
    remove_punc: bool,
    lower: bool,
    stem: bool,
}

fn main() {
    let config = parse_args().unwrap();
    let data = std::fs::read_to_string(config.file).unwrap();
    let re = Regex::new(r"[^\w\s\d']").unwrap();
    let stop: HashSet<_> = stop_words::get(stop_words::LANGUAGE::English)
        .into_iter()
        .map(|x| re.replace_all(&x, "").to_string()) // For some reason they are literally quoted
        .collect();
    dbg!(stop.len());

    // Options that the user can control should include: lowercasing,
    // either stemming or lemmatization, stopword removal, and at least one additional
    // option you added.
    let mut data = if config.remove_punc {
        match re.replace_all(&data, "") {
            Cow::Borrowed(x) => x.to_string(),
            Cow::Owned(x) => x,
        }
    } else {
        todo!()
    };

    if config.lower {
        data.make_ascii_lowercase();
    }
    let stemmer = Stemmer::create(Algorithm::English);
    let tokens = data.split_whitespace().filter(|&word| !stop.contains(word));
    let tokens: Vec<_> = if config.stem {
        tokens.map(|x| stemmer.stem(x)).collect()
    } else {
        tokens.map(|x| Cow::Borrowed(x)).collect() // Keep types the same
    };
    let mut counts: HashMap<_, u32> = HashMap::new();
    for token in tokens.iter() {
        *counts.entry(token).or_default() += 1;
    }
    let mut output: Vec<_> = counts.iter().map(|(k, v)| (*v, k)).collect();
    output.sort_by(|a, b| b.cmp(a));
    for x in output.iter().skip(16).take(16) {
        dbg!(x);
    }
    let counts = output.iter().map(|(x, _)| x).copied().collect();
    plot(counts);
}

fn plot(output: Vec<u32>) {
    use plotly::common::Title;
    use plotly::layout::{Axis, Layout};
    use plotly::{Plot, Scatter};
    let layout = Layout::new()
        .title(Title::new("Token Analysis"))
        .x_axis(Axis::new().title("Rank".into()))
        .y_axis(Axis::new().type_(AxisType::Log).title("Token Count".into()));
    let mut plot = Plot::new();
    let trace = Scatter::new((1..=output.len()).collect(), output);
    plot.add_trace(trace);
    plot.set_layout(layout);
    plot.write_html("plot.html");
}

fn parse_args() -> Result<Config> {
    let args: Vec<_> = std::env::args().collect();
    let program = args[0].clone();
    let mut opts = Options::new();
    opts.optflag("p", "punctuation", "Retains punctuation information");
    opts.optflag("l", "lower", "Lowercases the input text");
    opts.optflag("s", "stem", "Stems the input text");
    opts.optflag("h", "help", "print this help menu");
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
    let file = matches.free[0].clone();
    Ok(Config {
        file,
        remove_punc: !matches.opt_present("p"),
        lower: matches.opt_present("l"),
        stem: matches.opt_present("s"),
    })
}

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} FILE [options]", program);
    print!("{}", opts.usage(&brief));
}
