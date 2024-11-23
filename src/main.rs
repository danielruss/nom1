use nom1::{parse_module, ModuleItem};
use regex::Regex;
use std::time::Instant;
pub mod connect_module;
use connect_module::get_connect_module;

fn main() {
    let markdown = get_connect_module("module1.txt").expect("Could not load the file");
    // strip out the comments...
    let re = Regex::new(r"//.*").unwrap();
    let markdown = re.replace_all(&markdown, "");

    // min,mean,max
    let mut timings: (u128, f64, u128) = (0, 0., 0);
    let mut sum: u128 = 0;

    if false {
        for i in 0..1000 {
            let now = Instant::now();
            let _ = parse_module(&markdown).unwrap();
            let elapsed_time = now.elapsed().as_millis();
            sum = elapsed_time + sum;
            if i == 0 {
                timings = (elapsed_time, sum as f64, elapsed_time);
            } else {
                timings = (
                    elapsed_time.min(timings.0),
                    (sum as f64) / (i as f64),
                    elapsed_time.max(timings.2),
                );
            }
        }
    }

    if true {
        let (remaining_text, module) = parse_module(&markdown).unwrap();
        println!("Preamble:\n{:?}", module.preamble.trim());
        for (indx, mi) in module.items.iter().enumerate() {
            if indx < 1000 {
                match mi {
                    ModuleItem::Question(_q) => {
                        //println!("{:?}", q.header)
                    }
                    ModuleItem::Loop(l) => println!("====> LOOP:\n{:?}", l),
                    ModuleItem::Grid(g) => println!("{:?}", g),
                }
                //println!("{}: {:?}", indx + 1, mi)
            }
        }
        println!("anything additional???:\n{}", remaining_text);
    }

    println!("Time to parse: {:?}", timings);
}
