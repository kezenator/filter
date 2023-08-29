use std::collections::HashMap;
use std::hash::Hash;
use std::fmt::Display;
use nalgebra::ComplexField;
use num::traits::Zero;

pub struct EquationBuilder<Variable, Scalar> where
    Variable: Eq + Hash + Display + Clone,
    Scalar: ComplexField
{
    constant: Scalar,
    variables: HashMap<Variable, Scalar>,
}

impl<Variable, Scalar> EquationBuilder<Variable, Scalar> where
    Variable: Eq + Hash + Display + Clone,
    Scalar: ComplexField
{
    pub fn new() -> Self
    {
        EquationBuilder { constant: Scalar::zero(), variables: HashMap::new() }
    }

    pub fn constant(&self) -> Scalar
    {
        self.constant.clone()
    }

    pub fn variables(&self) -> HashMap<Variable, Scalar>
    {
        self.variables.clone()
    }

    pub fn add_constant(&mut self, constant: Scalar)
    {
        self.constant += constant;
    }

    pub fn add_variable(&mut self, var: Variable, coeff: Scalar)
    {
        *self.variables.entry(var).or_insert(Scalar::zero()) += coeff;
    }

    pub fn to_string(&self) -> String
    {
        let mut result = String::new();
        let mut prefix = "";
        for (var, coeff) in self.variables.iter()
        {
            result += &format!("{}{}*{}", prefix, coeff, var);
            prefix = " + ";
        }
        result += &format!(" = {}", self.constant);
        return result;
    }
}
