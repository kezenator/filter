use nalgebra::{DMatrix, DVector, ComplexField};
use std::collections::HashMap;
use std::fmt::Display;
use std::hash::Hash;

use super::builder::EquationBuilder;

struct Entry<Scalar>
where
    Scalar: ComplexField,
{
    row: usize,
    col: usize,
    value: Scalar,
}

pub struct LinearSystem<Variable, Scalar>
where
    Variable: Eq + Hash + Display + Clone,
    Scalar: ComplexField,
{
    variables: Vec<Variable>,
    var_to_index: HashMap<Variable, usize>,
    vector: Vec<Scalar>,
    coefs: Vec<Entry<Scalar>>,
}

impl<Variable, Scalar> LinearSystem<Variable, Scalar>
where
    Variable: Eq + Hash + Display + Clone,
    Scalar: ComplexField,
{
    pub fn new() -> Self {
        LinearSystem {
            variables: Vec::new(),
            var_to_index: HashMap::new(),
            vector: Vec::new(),
            coefs: Vec::new(),
        }
    }

    pub fn add_equation(&mut self, builder: EquationBuilder<Variable, Scalar>)
    {
        let row = self.vector.len();

        self.vector.push(builder.constant());

        for entry in builder.variables() {
            let col = match self.var_to_index.get(&entry.0) {
                Some(index) => *index,
                None => {
                    let index = self.variables.len();
                    self.variables.push(entry.0.clone());
                    self.var_to_index.insert(entry.0, index);
                    index
                }
            };

            self.coefs.push(Entry {
                row,
                col,
                value: entry.1,
            });
        }
    }

    pub fn solve(&self) -> Option<HashMap<Variable, Scalar>>
    {
        assert!(self.variables.len() == self.vector.len());

        let dim = self.variables.len();

        let mut matrix = DMatrix::<Scalar>::zeros(dim, dim);

        for entry in self.coefs.iter()
        {
            matrix[(entry.row, entry.col)] = entry.value.clone();
        }

        let b = DVector::from_vec(self.vector.clone());

        let lu = matrix.lu();

        if let Some(x) = lu.solve(&b)
        {
            let mut result = HashMap::new();
            for i in 0..dim
            {
                result.insert(self.variables[i].clone(), x[i].clone());
            }
            return Some(result);
        }

        None
    }

    pub fn print(&self)
    {
        println!(
            "   {} vars, {} equations",
            self.variables.len(),
            self.vector.len()
        );
        for row in 0..self.vector.len()
        {
            print!("|");
            for col in 0..self.variables.len() {
                print!(" {:8}", self.get_coef(row, col));
            }
            if row < self.variables.len()
            {
                print!(" | | {:8}", self.variables[row]);
            };
            println!(" | = | {:8} |", self.vector[row]);
        }
    }

    fn get_coef(&self, row: usize, col: usize) -> Scalar
    {
        self.coefs
            .iter()
            .filter(|c| c.row == row && c.col == col)
            .next()
            .map(|c| c.value.clone())
            .unwrap_or(Scalar::zero())
    }
}
