use std::collections::BTreeMap;
use crate::netlist::{Device, Netlist, Scalar};
use crate::la::{Builder, EquationIndex, Solver, System, VariableIndex};

pub struct TransientSimulation
{
    system: System,
    equations: Vec<Equation>,
    time: Scalar
}

impl TransientSimulation
{
    pub fn new(netlist: &Netlist) -> Self
    {
        let mut builder = Builder::new();
        let mut equations = Vec::new();

        // First - reference V_GND as 0 Volts

        {
            let _ = builder.new_equation();
            let gnd = builder.find_var("V_GND");
            equations.push(Equation::GndRef { gnd });
        }

        // The sum of currents into every node
        // must be zero. Skip the GND node
        // as it's a redundant equation

        for node in netlist.nodes()
        {
            if node.name() != "GND"
            {
                let mut currents = Vec::new();
                for device in netlist.devices()
                {
                    if let Some(factor) = device.flow_into_node(&node)
                    {
                        let var = builder.find_var(&format!("I_{}", device.name()));
                        currents.push((var, factor));
                    }
                }
                equations.push(Equation::NodeCurrents { currents });
            }
        }

        // Finally - every device generates an equation

        for device in netlist.devices()
        {
            match device
            {
                Device::Voltage { plus, minus, voltage, .. } =>
                {
                    let plus = builder.find_var(&format!("V_{}", plus.name()));
                    let minus = builder.find_var(&format!("V_{}", minus.name()));
                    let voltage = voltage.value();
                    equations.push(Equation::Voltage { voltage, plus, minus });
                },
                Device::Resitor { name, plus, minus, resistance } =>
                {
                    let current = builder.find_var(&format!("I_{}", name));
                    let plus = builder.find_var(&format!("V_{}", plus.name()));
                    let minus = builder.find_var(&format!("V_{}", minus.name()));
                    let conductance = 1.0 / resistance.value();
                    equations.push(Equation::Conductance { current, plus, minus, conductance });
                },
            }
        }

        let system = builder.build();
        let time = 0.0;

        TransientSimulation{ system, equations, time }
    }

    pub fn simulate(&mut self, delta_t: Scalar, steps: usize) -> BTreeMap<String, Vec<Scalar>>
    {
        let mut results = vec![vec![0.0;steps];self.system.dim()];

        for step in 0..steps
        {
            let time = (step as Scalar) * delta_t + self.time;

            let mut solver = self.system.new_solver();
            for (i, eq) in self.equations.iter().enumerate()
            {
                eq.fill(&mut solver, EquationIndex::from_index(i), time);
            }

            match solver.solve()
            {
                Some(solution) =>
                {
                    for (var_index, var_solution) in solution.iter().enumerate()
                    {
                        results[var_index][step] = *var_solution;
                    }
                },
                None =>
                {
                    println!("Solution failed at time={}", time);
                    let mut solver = self.system.new_solver();
                    for (i, eq) in self.equations.iter().enumerate()
                    {
                        eq.fill(&mut solver, EquationIndex::from_index(i), time);
                    }
                    self.system.print(&solver);
                    panic!();
                },
            }
        }
        self.time += (steps as Scalar) * delta_t;

        results.into_iter().enumerate()
            .map(|(var_index, var_results)| (self.system.variables()[var_index].clone(), var_results))
            .collect()
    }
}

pub enum Equation
{
    GndRef{gnd: VariableIndex},
    NodeCurrents{currents: Vec<(VariableIndex, Scalar)>},
    Voltage{voltage: Scalar, plus: VariableIndex, minus: VariableIndex},
    Conductance{conductance: Scalar, plus: VariableIndex, minus: VariableIndex, current: VariableIndex},
}

impl Equation
{
    pub fn fill(&self, solver: &mut Solver, eq: EquationIndex, time: Scalar)
    {
        match self
        {
            Equation::GndRef { gnd } =>
            {
                // V_GND = 0
                *solver.coef(eq, *gnd) = 1.0;
            },
            Equation::NodeCurrents { currents } =>
            {
                // +/- I_1 +/- I_2 +/- ... = 0
                for current in currents.iter()
                {
                    *solver.coef(eq, current.0) = current.1;
                }
            },
            Equation::Voltage { voltage, plus, minus } =>
            {
                // V+ - V- = voltage
                *solver.coef(eq, *plus) = 1.0;
                *solver.coef(eq, *minus) = -1.0;
                *solver.constant(eq) = *voltage + (time * 0.5 * std::f64::consts::FRAC_1_PI * 440.0).sin();
            },
            Equation::Conductance { conductance, plus, minus, current } =>
            {
                // V = IR => I = V / R
                // => I - V/R = 0
                // => I - (V+ - V-) / R = 0
                // => I + V-/R - V+/R = 0
                // => I + V-.C - V+.C = 0
                *solver.coef(eq, *current) = 1.0;
                *solver.coef(eq, *minus) = *conductance;
                *solver.coef(eq, *plus) = -conductance;
            },
        }
    }
}