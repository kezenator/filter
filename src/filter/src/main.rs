use std::collections::BTreeMap;

use filter_lib::netlist::{Netlist, NetlistParseError};
const NETLIST_FILE: &str = r#"
V1 1 GND 1
R1 1 2 1000
R2 2 GND 1000
R3 2 GND 500"#;

fn main() -> Result<(), NetlistParseError>
{
    let netlist = NETLIST_FILE.parse::<Netlist>()?;
    println!("{:?}", netlist);

    let eqns = netlist.get_equations();

    eqns.print();

    if let Some(solution) = eqns.solve()
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

    Ok(())
}
