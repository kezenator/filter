use std::collections::{BTreeMap, HashMap};
use nalgebra::{DMatrix, DVector};

#[derive(Clone, Copy)]
pub struct EquationIndex(usize);
#[derive(Clone, Copy)]
pub struct VariableIndex(usize);

impl EquationIndex
{
    pub fn from_index(i: usize) -> Self
    {
        EquationIndex(i)
    }
}

pub struct System
{
    variables_in_order: Vec<String>,
}

impl System
{
    pub fn new_solver(&self) -> Solver
    {
        Solver::new(self.variables_in_order.len())
    }

    pub fn dim(&self) -> usize
    {
        self.variables_in_order.len()
    }

    pub fn variables(&self) -> &Vec<String>
    {
        &self.variables_in_order
    }

    pub fn solution_to_named_vars(&self, solution: Option<Vec<f64>>) -> Option<BTreeMap<String, f64>>
    {
        match solution
        {
            Some(solution) =>
            {
                assert!(solution.len() == self.variables_in_order.len());
                Some(solution.iter().enumerate()
                    .map(|(i, val)| (self.variables_in_order[i].clone(), *val))
                    .collect())
            }
            None => None
        }
    }

    pub fn print(&self, solver: &Solver)
    {
        assert!(self.variables_in_order.len() == solver.dim());
        let dim = solver.dim();

        for row in 0..dim
        {
            print!("|");
            for col in 0..dim
            {
                print!(" {:8}", solver.a[(row, col)]);
            }
            print!(" | | {:8}", self.variables_in_order[row]);
            println!(" | = | {:8} |", solver.b[row]);
        }
    }
}

pub struct Solver
{
    a: DMatrix<f64>,
    b: DVector<f64>,
}

impl Solver
{
    pub fn new(dim: usize) -> Self
    {
        let a = DMatrix::zeros(dim, dim);
        let b = DVector::zeros(dim);
        Solver {a, b}
    }

    pub fn dim(&self) -> usize
    {
        self.b.len()
    }

    pub fn coef<'a>(&'a mut self, eq: EquationIndex, var: VariableIndex) -> &'a mut f64
    {
        &mut self.a[(eq.0, var.0)]
    }

    pub fn constant<'a>(&'a mut self, eq: EquationIndex) -> &'a mut f64
    {
        &mut self.b[eq.0]
    }

    pub fn solve(self) -> Option<Vec<f64>>
    {
        let lu = self.a.lu();
        lu.solve(&self.b).map(|mut m| m.as_mut_slice().into())
    }
}

pub struct Builder
{
    vars_to_index: HashMap<String, usize>,
    vars_in_order: Vec<String>,
    equation_count: usize,
}

impl Builder
{
    pub fn new() -> Self
    {
        Builder
        {
            vars_to_index: HashMap::new(),
            vars_in_order: Vec::new(),
            equation_count: 0,
        }
    }

    pub fn new_equation(&mut self) -> EquationIndex
    {
        let result = self.equation_count;
        self.equation_count += 1;
        EquationIndex(result)
    }

    pub fn find_var(&mut self, name: &str) -> VariableIndex
    {
        VariableIndex(*self.vars_to_index
            .entry(name.to_owned())
            .or_insert_with(|| 
            {
                let new_index = self.vars_in_order.len();
                self.vars_in_order.push(name.to_owned());
                new_index
            }))
    }

    pub fn build(self) -> System
    {
        System{ variables_in_order: self.vars_in_order }
    }
}
