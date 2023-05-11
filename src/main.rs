#![windows_subsystem = "windows"]
#![allow(clippy::many_single_char_names)]
#![allow(clippy::manual_range_contains)]
use fltk::{app::*, button::*, dialog::*, frame::*, group::*, input::*, text::*, window::*, prelude::*};
use rand::Rng;
use std::cmp::Ordering;
use std::f64;

#[derive(Clone, Debug)]
// Define a struct for the form fields
struct Parameters {
    data_a: TextEditor,
    output: TextDisplay,
    zthresh: FloatInput,
}

#[derive(Clone, Debug)]
// Define a struct for our Z Score Counts
struct Zscoreresults {
    pluscountA: usize,
    minuscountA: usize,
    pluspercentA: f64,
    minuspercentA: f64,
    pluscountB: usize,
    minuscountB: usize,
    pluspercentB: f64,
    minuspercentB: f64,
}

fn main() {
    let app = App::default();

    // Main Window
    let mut wind = Window::new(100, 100, 737, 530, "Z Score Percentage Calculator v1.00");

    // Fill the form structure
    let mut parameters = Parameters {
        data_a: TextEditor::new(16, 30, 204, 404, ""),
        zthresh: FloatInput::new(558, 172, 54, 22, "Z Thresh"),
        output: TextDisplay::new(480, 200, 230, 300, ""),
    };

    // Text buffers for our inputs and output
    let buf_a = TextBuffer::default();
    let buf_out = TextBuffer::default();

    // Data Labels for Main Input windows
    Frame::new(16, 10, 51, 17, "Data A");

    // Format and initialize our main input windows
    parameters.data_a.set_scrollbar_size(15);
    parameters.data_a.set_cursor_style(Cursor::Simple);
    parameters.data_a.set_buffer(Some(buf_a));
    parameters.data_a.set_tab_nav(true);

    // Set output buffer
    parameters.output.set_buffer(Some(buf_out));


    // Set intial values for the form
    parameters.zthresh.set_value("3.0");

    // Clone the parameters to use for the clear function
    let mut p2 = parameters.clone();

    // Calculate button
    let mut calculate_button = Button::new(130, 450, 200, 57, "Calculate");
    calculate_button.set_callback(move |_| calculate(&mut parameters));

    // clear button
    let mut clear_button = Button::new(350, 450, 100, 57, "Clear");
    clear_button.set_callback(move |_| clear(&mut p2));

    // Show the window
    wind.end();
    wind.show();

    // Enter main loop
    app.run().unwrap();
}

fn clear(p: &mut Parameters) {
    p.output.buffer().unwrap().set_text("");
    p.data_a.buffer().unwrap().set_text("");
}

// Handle Calculate button
fn calculate(p: &mut Parameters) {

    let window: usize = 100;

    // Output String
    let mut out: String = String::from("");

    // Get the CSV data out of the two data fields
    let a_v: Vec<f64> = csv_split(&p.data_a.buffer().unwrap().text());

    if a_v.is_empty() {
        return;
    }

    // Get our Z Score Threshold
    let zthresh: f64 = match p.zthresh.value().parse::<f64>() {
        Ok(v) => v,
        Err(_) => {
            alert(368, 265, "Z Threshold Error");
            return;
        }
    };

    let zc = zcount(&a_v, zthresh, 100, 300);

    out.push_str(&format!(
        "\nZC + A:    \t{}\n",
        zc.pluscountA 
    ));

    out.push_str(&format!(
        "\nZC - A:    \t{}\n",
        zc.minuscountA 
    ));

    out.push_str(&format!(
        "\nZC + B:    \t{}\n",
        zc.pluscountB 
    ));

    out.push_str(&format!(
        "\nZC - B:    \t{}\n",
        zc.minuscountB 
    ));

    out.push_str(&format!(
        "\n\nZP + A:    \t{}\n",
        &science_pretty_format(zc.pluspercentA ,2)
    ));

    out.push_str(&format!(
        "\nZP - A:    \t{}\n",
        &science_pretty_format(zc.minuspercentA ,2)
    ));

    out.push_str(&format!(
        "\nZP + B:    \t{}\n",
        &science_pretty_format(zc.pluspercentB ,2)
    ));

    out.push_str(&format!(
        "\nZP - B:    \t{}\n",
        &science_pretty_format(zc.minuspercentB ,2)
    ));


    for y in 0..a_v.len() - window {
        let x = &a_v[y..y+window];
        let mean = mean(x);
        let sd = sd_pop(&x, &mean);
        out.push_str(&format!(
            "\n{},{},{}",
            y + window / 2,
            &science_pretty_format(mean , 4),
            &science_pretty_format(sd , 4)
        ));
    }
        


    p.output.buffer().unwrap().set_text(&out);
}

// Convert CSV from the main windows to arrays of floats, also clean up stray whitespace
fn csv_split(inp: &str) -> Vec<f64> {
    let mut values: Vec<f64> = Vec::new();

    let clean_inp: String = inp
        .replace("\n", ",")
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect();

    let fields = clean_inp.split(',');

    for f in fields {
        match f.parse::<f64>() {
            Ok(v) => values.push(v),
            Err(_) => continue,
        };
    }

    values
}

// Calculate mean
fn mean(vec: &[f64]) -> f64 {
    let sum: f64 = Iterator::sum(vec.iter());
    sum / vec.len() as f64
}

// Calculate Percent difference
fn per_change(f: &f64, s: &f64) -> f64 {
    (s - f) / f.abs() * 100.0
}

// Calculate SD of a sample
fn sd_pop(x: &[f64], mean: &f64) -> f64 {
    let mut sd: f64 = 0.0;

    for v in x.iter() {
        sd += (v - mean).powf(2.0);
    }
    (sd / x.len() as f64).sqrt()
}

fn zcount(x: &Vec<f64>, zth: f64, start: usize, end: usize) -> Zscoreresults {
    let mut count: usize = 0;
    let sublen = end - start;
    let mean = mean(&x);
    let sd = sd_pop(&x, &mean);
    let mut zresults: Zscoreresults = Zscoreresults { pluscountA: 0, minuscountA: 0, pluspercentA: 0.0, minuspercentA: 0.0, pluscountB: 0, minuscountB: 0, pluspercentB: 0.0, minuspercentB: 0.0 }; 

    if sublen < 1 {
        return zresults;
    }

    if end > x.len() || start > x.len() || end < 1 || start < 1 {
        return zresults;
    }

    for v in x {
        let z = (v - mean) / sd;

        if z >= zth.abs() {
            zresults.pluscountA += 1;
        }
        if z <= -zth.abs() {
            zresults.minuscountA += 1;
        }
        if count >= start && count <= end {
            if z >= zth.abs() {
                zresults.pluscountB += 1;
            }
            if z <= -zth.abs() {
                zresults.minuscountB += 1;
            }
        }
        count += 1;
    };

    zresults.pluspercentA = (zresults.pluscountA as f64 / x.len() as f64) * 100.0;
    zresults.minuspercentA = (zresults.minuscountA as f64 / x.len() as f64) * 100.0;

    zresults.pluspercentB = (zresults.pluscountB as f64 / sublen as f64) * 100.0;
    zresults.minuspercentB = (zresults.minuscountB as f64 / sublen as f64) * 100.0;

    zresults
}

// Pretty Format Scientific Numbers
fn science_pretty_format(value: f64, digits: usize) -> String {
    if value.abs() == 0.0 {
        "0".to_string();
    }
    if value.abs() >= 10000.0 || value.abs() < 0.001 {
        format!("{:.*e}", digits, value);
    }
    format!("{:.*}", digits, value)
        .trim_end_matches(|c| c == '0')
        .trim_end_matches(|c| c == '.')
        .to_string()
}
