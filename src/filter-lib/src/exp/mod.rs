pub enum Exp
{
    Value(f64),
    Sum(Vec<Box<Exp>>),
    Product(Vec<Box<Exp>>),
    Sin(f64),
}

impl Exp
{
    pub fn calc(&self, time: f64) -> f64
    {
        match self
        {
            Exp::Value(value) => *value,
            Exp::Sum(terms) => terms.iter().map(|t| t.calc(time)).sum(),
            Exp::Product(factors) => factors.iter().map(|f| f.calc(time)).product(),
            Exp::Sin(freq) => (time * freq * 0.5 * std::f64::consts::FRAC_1_PI).sin()
        }
    }
}