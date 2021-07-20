use yahoo::Quote;
use yahoo_finance_api as yahoo;
use std::{error::Error, ops::{Div, Mul, Sub}};
use chrono::prelude::*;
use clap::{Arg, App};
use std::io::{self, Write};

struct ClosingPriceData {
    min: f64,
    max: f64,
    avg: f64,
}

fn main() -> Result<(), Box<dyn Error>> {
    let date_str = get_user_input()?;    

    let date_split: Vec<i32> = 
            date_str                                                             // (1)
            .split('/').map(|s| s.trim())                 // (2)
            .filter(|s| !s.is_empty())                  // (3)
            .map(|s| s.parse().unwrap_or(1))      // (4)
            .collect();                                                             // (5)

    if date_split.len() != 3 {
        io::stdout().write_all(b"Date Format Incorrect")?;
        return Ok(());
    } else if date_split[0] < 2019 {
        io::stdout().write_all(b"Year should be greater than 2018")?;
        return Ok(());
    }
    
    let sym = ["MSFT", "GOOG", "AAPL", "UBER", "IBM"];

    
    let from = NaiveDate::parse_from_str(&date_str, "%Y/%m/%d")?;
    let from = from.and_time(NaiveTime::from_hms_milli(0, 0, 0, 0));


    io::stdout().write_all(b"period start,symbol,price,change %,min,max,30d avg\n")?;

    sym.iter().for_each(|s| {
        let raw_close_prices = &fetch_stock_data(s, &date_split).unwrap_or_default().iter().map(|q| q.adjclose).collect::<Vec<f64>>();
        let data = refine_quotes(raw_close_prices);
        let sma = n_window_sma(30, raw_close_prices).unwrap_or_default();
        let pd = price_diff(raw_close_prices).unwrap_or_default();

        let o = format!(
            "{}, {}, ${:.2}, {:.2}%, ${:.2}, ${:.2}, ${:.2}",
            from,
            s,
            raw_close_prices.last().unwrap_or(&0f64),
            pd.0,
            data.min,
            data.max,
            sma.last().unwrap_or(&0.0)
        ) + "\n";
        let _r = io::stdout().write_all(o.as_bytes());
    });

    Ok(())
}

fn get_user_input() -> Result<String, Box<dyn Error>> {
    let matches = App::new("My Test Program")
        .version("0.1.0")
        .author("Hackerman Jones <hckrmnjones@hack.gov>")
        .about("Teaches argument parsing")
        .arg(Arg::with_name("date")
                 .short("d")
                 .long("date")
                 .takes_value(true)
                 .help("Retrieve data since")
            )
        .get_matches();

    let date_str = matches.value_of("date").ok_or("Not a date")?;
    
    Ok(date_str.to_string())
}

fn fetch_stock_data(sym: &str, date_split: &Vec<i32>) -> Result<Vec<Quote>, Box<dyn Error>> {
    let provider = yahoo::YahooConnector::new();
    let start = Utc.ymd(date_split[0], date_split[1] as u32, date_split[2] as u32).and_hms_milli(0, 0, 0, 0);
    let end = Utc::now();

    let response = provider.get_quote_history(sym, start, end)?;
    let s = response.quotes()?;

    Ok(s)
}

fn refine_quotes(quotes: &[f64]) -> ClosingPriceData {
    let count = quotes.len() as f64;
    let mut cpd = quotes.iter().map(|q| q).fold(ClosingPriceData {min: f64::MAX, max: f64::MIN, avg: 0f64}, |mut a, b| {
        if a.min >= *b { a.min = *b; }
        if a.max <= *b { a.max = *b; }
        a.avg += b.div(count);

        a
    });

    cpd.min = round(cpd.min);
    cpd.max = round(cpd.max);
    cpd.avg = round(cpd.avg);

    cpd
}

fn n_window_sma(n: usize, series: &[f64]) -> Result<Vec<f64>, Box<dyn Error>> {
    let win = series.windows(n);

    let a = win.into_iter().map(|i| {
        round(i.iter().sum::<f64>().div(n as f64))
    }).collect::<Vec<_>>();

    Ok(a)
}

fn price_diff(series: &[f64]) -> Result<(f64, f64), Box<dyn Error>> {
    let f = series.first().ok_or("Invalid series")?;
    let l = series.last().ok_or("Invalid series")?;
    let del = round(l.sub(f));

    let n1 = round(del.mul(100f64).div(f));
    Ok((n1, del))
}

fn round(i: f64) -> f64 {
    i.mul(100f64).round().div(100f64)
}