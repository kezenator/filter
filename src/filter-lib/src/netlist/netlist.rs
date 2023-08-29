use std::str::FromStr;
use std::collections::HashSet;
use crate::eqn::{EquationBuilder, LinearSystem};

use super::{Device, NetlistParseError, IVCurve};

#[derive(Debug, Clone)]
pub struct Netlist
{
    devices: Vec<Device>
}

impl Netlist
{
    pub fn get_equations(&self) -> LinearSystem<String, f64>
    {
        let mut result = LinearSystem::<String, f64>::new();

        // Reference GND to zero volts

        {
            let mut b1 = EquationBuilder::<String, f64>::new();
            b1.add_variable("V_GND".to_owned(), 1.0);
            result.add_equation(b1);
        }

        // Sum of currents into each node is zero

        {
            let nodes = self.devices.iter()
                .flat_map(|d| d.nodes())
                .collect::<HashSet<_>>();

            for node in nodes.into_iter()
            {
                if (node.name() != "GND")
                {
                    let mut builder = EquationBuilder::<String, f64>::new();

                    for dev in self.devices.iter()
                    {
                        if let Some(factor) = dev.direction_into_node(&node)
                        {
                            let var = format!("I_{}", dev.name());
                            builder.add_variable(var, factor);
                        }
                    }

                    result.add_equation(builder);
                }
            }
        }

        // Calculate voltages across each component

        for dev in self.devices.iter()
        {
            let mut builder = EquationBuilder::<String, f64>::new();

            match dev.get_ivcurve()
            {
                IVCurve::FixedVoltage(voltage) =>
                {
                    // V(+) - V(-) = voltage
                    let nodes = dev.nodes();
                    builder.add_variable(format!("V_{}", nodes[0]), 1.0);
                    builder.add_variable(format!("V_{}", nodes[1]), -1.0);
                    builder.add_constant(voltage);
                },
                IVCurve::Resistance(resistance) =>
                {
                    // V=IR
                    // => I=V/R
                    // => I = (V+ - V-) / R
                    // => I -1/R * V+ + 1/R + V- = 0
                    let nodes = dev.nodes();
                    builder.add_variable(format!("I_{}", dev.name()), 1.0);
                    builder.add_variable(format!("V_{}", nodes[0]), -1.0 / resistance);
                    builder.add_variable(format!("V_{}", nodes[1]), 1.0 / resistance);
                },
            }

            result.add_equation(builder);
        }
        
        result
    }
}

impl FromStr for Netlist
{
    type Err = NetlistParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err>
    {
        let devices = s
                .lines()
                .filter(|l| !l.is_empty())
                .map(|l| l.parse::<Device>())
                .collect::<Result<Vec<Device>, Self::Err>>()?;

        check_netlist(&devices)?;

        Ok(Netlist{ devices })
    }
}

fn check_netlist(devices: &Vec<Device>) -> std::result::Result<(), NetlistParseError>
{
    // Get all nodes

    let nodes = devices.iter()
        .flat_map(|d| d.nodes())
        .map(|nn| nn.name().to_owned())
        .collect::<HashSet<_>>();

    // Get all device names

    let device_names = devices.iter()
        .map(|d| d.name().to_owned())
        .collect::<HashSet<_>>();

    // Check there is a GND node

    if !nodes.contains(&"GND".to_owned())
    {
        return Err("No GND node found".into());
    }

    // Check the node and device names
    // dont intersect

    if device_names.intersection(&nodes).next().is_some()
    {
        return Err("Node and device names intersect".into());
    }

    // Check the device names are unique

    if device_names.len() != devices.len()
    {
        return Err("Non-unique device name".into())
    }

    // All looks OK

    Ok(())
}