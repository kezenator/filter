use  std::fmt::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NodeName
{
    name: String,
}

impl NodeName
{
    pub fn gnd() -> Self
    {
        Self::new("0".to_owned())
    }

    pub fn new(name: String) -> NodeName
    {
        NodeName { name }
    }

    pub fn name<'a>(&'a self) -> &'a str
    {
        &self.name
    }
}

impl Display for NodeName
{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result
    {
        write!{f, "{}", self.name}
    }
}
