use std::fs::File;
use std::io::prelude::*;
use std::time::Instant;

use filter_lib::{netlist::{Netlist, ParseError}, sim::transient::TransientSimulation};

const NETLIST_FILE: &str = r#"
V1 1 GND 4*sin(1000+10000*t)+30*t
R1 1 2 1000
R2 2 GND 100000
C3 2 GND 0.000005
D1 2 GND
D2 GND 2
Rd 2 3 10000000
E1 4 GND 2 3 1000000
Rg1 4 3 3000
Rg2 3 GND 1000"#;

fn main() -> Result<(), ParseError>
{
    let netlist = NETLIST_FILE.parse::<Netlist>()?;

    let mut trans = TransientSimulation::new(&netlist);

    let mut graph = filter_lib::graph::Graph::new();

    let steps = 6000;

    let start = Instant::now();

    let results = trans.simulate(1.0/48000.0, steps);

    let duration = start.elapsed();
    println!("Solved {} steps in  {:?}", steps, duration);

    for var in ["V_1", "V_2", "V_3", "V_4"]
    {
        graph.add_trace(results.get(var).unwrap(), 5.0, var, "V");
    }

    let svg = graph.to_svg();
    
    let mut file = File::create("results.svg").unwrap();
    file.write_all(svg.as_bytes()).unwrap();

    Ok(())
}
