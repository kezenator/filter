use std::fmt::*;
use std::str::FromStr;
use super::{NodeName, Value};
use super::parser::{LineParser, NetlistParseError};

#[derive(Debug, Clone)]
pub enum Device
{
    Voltage{name: String, plus: NodeName, minus: NodeName, voltage: Value},
    Resitor{name: String, plus: NodeName, minus: NodeName, resistance: Value},
}

impl Device
{
    pub fn name<'a>(&'a self) -> &'a str
    {
        match self
        {
            Self::Voltage { name, ..} => name,
            Self::Resitor { name, ..} => name,
        }
    }

    pub fn nodes(&self) -> Vec<NodeName>
    {
        match self
        {
            Self::Voltage { plus, minus, .. }
                => vec![plus.clone(), minus.clone()],
            Self::Resitor { plus, minus, .. }
                => vec![minus.clone(), plus.clone()],
        }
    }

    pub fn flow_into_node(&self, node: &NodeName) -> Option<f64>
    {
        let nodes = self.nodes();
        if *node == nodes[0] { return Some(1.0); }
        if *node == nodes[1] { return Some(-1.0); }
        None
    }
}

impl FromStr for Device
{
    type Err = NetlistParseError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err>
    {
        let mut parser = LineParser::new(s.to_string());
        let name = parser.read_string()?;
        let char = name.chars().next().unwrap();
        let plus = parser.read_node_name()?;
        let minus = parser.read_node_name()?;
        let val = parser.read_value()?;
        parser.completed()?;

        match char
        {
            'V' => Ok(Device::Voltage
                {
                    name, plus, minus,
                    voltage: val
                }),
            'R' => Ok(Device::Resitor
                {
                    name, plus, minus,
                    resistance: val
                }),
            _ => Err("Unknown device type".into()),
        }
    }
}
