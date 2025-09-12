// src/main.rs

// Comments are double slashes. :)
use chrono::NaiveDate;
use polars::prelude::*;

fn main() -> PolarsResult<()> {
    // Build the birthdate column using chrono (good form: use the *_opt + unwrap).
    let birthdates: Vec<NaiveDate> = vec![
        NaiveDate::from_ymd_opt(1997, 1, 10).unwrap(),
        NaiveDate::from_ymd_opt(1985, 2, 15).unwrap(),
        NaiveDate::from_ymd_opt(1983, 3, 22).unwrap(),
        NaiveDate::from_ymd_opt(1981, 4, 30).unwrap(),
    ];

    // Build the DataFrame. `df!` returns a Result, so we use `?` and return PolarsResult.
    let df: DataFrame = df![
        "name"      => &["Alice Archer", "Ben Brown", "Chloe Cooper", "Daniel Donovan"],
        "birthdate" => &birthdates,                 // needs polars "chrono" + "dtype-date" features
        "weight"    => &[57.9_f64, 72.5, 53.6, 83.1],
        "height"    => &[1.56_f64, 1.77, 1.65, 1.75],
    ]?;

    // Pretty table print (requires "fmt" feature).
    println!("{df}");
    Ok(())
}
