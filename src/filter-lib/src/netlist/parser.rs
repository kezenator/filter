use super::{NodeName, Value};

#[derive(Debug)]
pub struct NetlistParseError
{
    err: String,
}

impl<T> From<T> for NetlistParseError
    where T: Into<String>
{
    fn from(err: T) -> Self
    {
        NetlistParseError { err: err.into() }
    }
}

pub struct LineParser
{
    parts: Vec<String>,
    index: usize,
}

impl LineParser
{
    pub fn new(line: String) -> Self
    {
        LineParser
        {
            parts: line.split_whitespace().map(String::from).collect(),
            index: 0,
        }
    }

    pub fn read_string(&mut self) -> Result<String, NetlistParseError>
    {
        if self.index < self.parts.len()
        {
            self.index += 1;
            Ok(self.parts[self.index - 1].clone())
        }
        else
        {
            Err("No more parts".into())
        }
    }

    pub fn read_node_name(&mut self) -> Result<NodeName, NetlistParseError>
    {
        Ok(NodeName::new(self.read_string()?))
    }

    pub fn read_value(&mut self) -> Result<Value, NetlistParseError>
    {
        match self.read_string()?.parse()
        {
            Ok(val) => Ok(Value::new(val)),
            Err(_) => Err("Invalid value".into()),
        }
    }

    pub fn completed(&mut self) -> Result<(), NetlistParseError>
    {
        if self.index < self.parts.len()
        {
            Err("Remaining parts".into())
        }
        else
        {
            Ok(())
        }
    }
}
