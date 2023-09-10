use super::{Exp, NodeName, Value};

#[derive(Debug, Clone)]
pub enum Device
{
    Voltage{name: String, plus: NodeName, minus: NodeName, voltage: Exp},
    Resistor{name: String, plus: NodeName, minus: NodeName, resistance: Value},
    Capacitor{name: String, plus: NodeName, minus: NodeName, capacitance: Value},
    Diode{name: String, plus: NodeName, minus: NodeName},
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
            Self::Diode { name, ..} => name,
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
            Self::Diode { plus, minus, .. }
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
