use super::{NodeName, Value};

#[derive(Debug, Clone)]
pub enum Device
{
    Voltage{name: String, plus: NodeName, minus: NodeName, voltage: Value},
    Resistor{name: String, plus: NodeName, minus: NodeName, resistance: Value},
    Capacitor{name: String, plus: NodeName, minus: NodeName, capacitance: Value},
}

impl Device
{
    pub fn name<'a>(&'a self) -> &'a str
    {
        match self
        {
            Self::Voltage { name, ..} => name,
            Self::Resistor { name, ..} => name,
            Self::Capacitor { name, ..} => name,
        }
    }

    pub fn nodes(&self) -> Vec<NodeName>
    {
        match self
        {
            Self::Voltage { plus, minus, .. }
                => vec![plus.clone(), minus.clone()],
            Self::Resistor { plus, minus, .. }
                => vec![minus.clone(), plus.clone()],
            Self::Capacitor { plus, minus, .. }
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
