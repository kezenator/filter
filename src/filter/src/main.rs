use std::collections::BTreeMap;
use std::fs::File;
use std::io::prelude::*;

use filter_lib::{netlist::{Netlist, NetlistParseError}, sim::transient::TransientSimulation};

const NETLIST_FILE: &str = r#"
V1 1 GND 1
R1 1 2 1000
R2 2 GND 1000
R3 2 GND 500"#;

fn main() -> Result<(), NetlistParseError>
{
    let netlist = NETLIST_FILE.parse::<Netlist>()?;
    println!("{:?}", netlist);

    let trans = TransientSimulation::new(&netlist);
    if let Some(solution) = trans.solve()
    {
        println!();
        println!("Solution:");
        for entry in BTreeMap::from_iter(solution.into_iter())
        {
            println!("   {:8} = {:8}", entry.0, entry.1);
        }
    }
    else
    {
        println!();
        println!("NO SOLUTION!");        
    }

    let values = (0..100)
        .map(|i| ((i as f64) * 0.1).sin())
        .collect::<Vec<_>>();

    let mut graph = filter_lib::graph::Graph::new();
    graph.add_trace(&values, 1.0);
    let svg = graph.to_svg();
    
    let mut file = File::create("results.svg").unwrap();
    file.write_all(svg.as_bytes()).unwrap();

    Ok(())
}
