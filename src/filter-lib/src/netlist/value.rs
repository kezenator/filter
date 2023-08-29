use  std::fmt::*;

#[derive(Debug, Clone)]
pub struct Value
{
    val: f64,
}

impl Value
{
    pub fn new(val: f64) -> Self
    {
        Value { val }
    }

    pub fn value(&self) -> f64
    {
        self.val
    }
}

impl Display for Value
{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result
    {
        write!{f, "{}", self.val}
    }
}
