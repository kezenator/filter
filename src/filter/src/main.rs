use std::collections::BTreeMap;
use std::fs::File;
use std::io::prelude::*;
use std::time::Instant;

use filter_lib::{netlist::{Netlist, NetlistParseError}, sim::transient::TransientSimulation};

const NETLIST_FILE: &str = r#"
V1 1 GND 1
R1 1 2 1000
R2 2 GND 1000
C3 2 GND 0.00005"#;

fn main() -> Result<(), NetlistParseError>
{
    let netlist = NETLIST_FILE.parse::<Netlist>()?;

    let mut trans = TransientSimulation::new(&netlist);

    let mut graph = filter_lib::graph::Graph::new();

    let steps = 6000;

    let start = Instant::now();

    let results = trans.simulate(1.0/48000.0, steps);

    let duration = start.elapsed();
    println!("Solved {} steps in  {:?}", steps, duration);

    for var in ["V_1", "V_2"]
    {
        graph.add_trace(results.get(var).unwrap(), 2.0, var, "V");
    }

    let svg = graph.to_svg();
    
    let mut file = File::create("results.svg").unwrap();
    file.write_all(svg.as_bytes()).unwrap();

    Ok(())
}
