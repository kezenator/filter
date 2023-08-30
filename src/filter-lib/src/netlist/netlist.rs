use std::str::FromStr;
use std::collections::HashSet;

use super::{Device, NetlistParseError, NodeName};

#[derive(Debug, Clone)]
pub struct Netlist
{
    devices: Vec<Device>
}

impl Netlist
{
    pub fn nodes(&self) -> HashSet<NodeName>
    {
        self.devices.iter()
            .flat_map(|d| d.nodes())
            .collect::<HashSet<_>>()
    }

    pub fn devices(&self) -> &Vec<Device>
    {
        &self.devices
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