use std::collections::BTreeMap;
use crate::netlist::{Device, Exp, Netlist, NodeName, Scalar};
use crate::la::{Builder, EquationIndex, Solver, System, VariableIndex};

const DIODE_MIN_CONDUCTANCE: Scalar = 1.0 / 100_000_000.0;
const DIODE_IS: Scalar = 1e-12;
const DIODE_1_ON_N_VT: Scalar = 1.0 / 1.5 / 0.025852;

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
        let gnd = NodeName::gnd();
        let mut builder = Builder::new();
        let mut equations = Vec::new();

        // First - reference V_GND as 0 Volts

        {
            let _ = builder.new_equation();
            let gnd = builder.find_var(&format!("V_{}", gnd.name()));
            equations.push(Equation::GndRef { gnd });
        }

        // The sum of currents into every node
        // must be zero. Skip the GND node
        // as it's a redundant equation

        for node in netlist.nodes()
        {
            if node != gnd
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
                    let voltage = voltage.clone();
                    equations.push(Equation::Voltage { voltage, plus, minus });
                },
                Device::Resistor { name, plus, minus, resistance } =>
                {
                    let current = builder.find_var(&format!("I_{}", name));
                    let plus = builder.find_var(&format!("V_{}", plus.name()));
                    let minus = builder.find_var(&format!("V_{}", minus.name()));
                    let conductance = 1.0 / resistance.value();
                    equations.push(Equation::Conductance { current, plus, minus, conductance });
                },
                Device::Capacitor { name, plus, minus, capacitance } =>
                {
                    let current = builder.find_var(&format!("I_{}", name));
                    let plus = builder.find_var(&format!("V_{}", plus.name()));
                    let minus = builder.find_var(&format!("V_{}", minus.name()));
                    let capacitance = capacitance.value();
                    let voltage = 0.0;
                    equations.push(Equation::Capacitor { current, plus, minus, capacitance, voltage });
                },
                Device::Diode { name, plus, minus} =>
                {
                    let current_var = builder.find_var(&format!("I_{}", name));
                    let plus_voltage_var = builder.find_var(&format!("V_{}", plus.name()));
                    let minus_voltage_var = builder.find_var(&format!("V_{}", minus.name()));
                    let conductance = DIODE_MIN_CONDUCTANCE;
                    let offset_voltage = 0.0;
                    equations.push(Equation::Diode { conductance, offset_voltage, plus_voltage_var, minus_voltage_var, current_var });
                },
                Device::Vcvs { plus, minus, control_plus, control_minus, gain, .. } =>
                {
                    let plus_voltage_var = builder.find_var(&format!("V_{}", plus.name()));
                    let minus_voltage_var = builder.find_var(&format!("V_{}", minus.name()));
                    let control_plus_voltage_var = builder.find_var(&format!("V_{}", control_plus.name()));
                    let control_minus_voltage_var = builder.find_var(&format!("V_{}", control_minus.name()));
                    let gain = gain.value();

                    equations.push(Equation::Vcvs{ plus_voltage_var, minus_voltage_var, control_plus_voltage_var, control_minus_voltage_var, gain});
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
                    for eq in self.equations.iter_mut()
                    {
                        eq.update(&solution, delta_t);
                    }

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
    Voltage{voltage: Exp, plus: VariableIndex, minus: VariableIndex},
    Conductance{conductance: Scalar, plus: VariableIndex, minus: VariableIndex, current: VariableIndex},
    Capacitor{capacitance: Scalar, plus: VariableIndex, minus: VariableIndex, current: VariableIndex, voltage: Scalar},
    Diode{conductance: Scalar, offset_voltage: Scalar, plus_voltage_var: VariableIndex, minus_voltage_var: VariableIndex, current_var: VariableIndex},
    Vcvs
    {
        gain: Scalar,
        plus_voltage_var: VariableIndex,
        minus_voltage_var: VariableIndex,
        control_plus_voltage_var: VariableIndex,
        control_minus_voltage_var: VariableIndex,
    },
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
                *solver.constant(eq) = voltage.calc(time);
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
            Equation::Capacitor { plus, minus, voltage, .. } =>
            {
                // Same as a voltage source
                // Stored voltage is updated when the new current is found

                *solver.coef(eq, *plus) = 1.0;
                *solver.coef(eq, *minus) = -1.0;
                *solver.constant(eq) = *voltage;
            },
            Equation::Diode { conductance, offset_voltage, plus_voltage_var, minus_voltage_var, current_var } =>
            {
                // Diode is modelled as an offset voltage, then a resistor
                // V+ - V- - Voff = I.R
                // I - V+.C + V-.C = Voff.C
                *solver.coef(eq, *current_var) = 1.0;
                *solver.coef(eq, *plus_voltage_var) = -conductance;
                *solver.coef(eq, *minus_voltage_var) = *conductance;
                *solver.constant(eq) = offset_voltage * conductance;
            },
            Equation::Vcvs { gain, plus_voltage_var, minus_voltage_var, control_plus_voltage_var, control_minus_voltage_var, ..} =>
            {
                // (V+ - V-) = G * (Vc+ - Vc-)
                // => V+ - V- - G*Vc+ + G*Vc- = 0
                *solver.coef(eq, *plus_voltage_var) = 1.0;
                *solver.coef(eq, *minus_voltage_var) = -1.0;
                *solver.coef(eq, *control_plus_voltage_var) = -gain;
                *solver.coef(eq, *control_minus_voltage_var) = *gain;
            }
        }
    }

    pub fn update(&mut self, solution: &Vec<Scalar>, delta_t: Scalar)
    {
        match self
        {
            Equation::Capacitor { capacitance, voltage, current, .. } =>
            {
                // I = C . dV/dt
                // => dV = I * dt / C
                *voltage += solution[current.into_index()] * delta_t / *capacitance;
            },
            Equation::Diode { conductance, offset_voltage, plus_voltage_var, minus_voltage_var, .. } =>
            {
                // First, work out the final voltage across the diode
                let new_voltage = solution[plus_voltage_var.into_index()] - solution[minus_voltage_var.into_index()];

                // Work out the new voltage offset and conductance
                if new_voltage <= 0.0
                {
                    // Negative biased:
                    // Just model as a very large resistor
                    // to simulate some leakage current

                    *conductance = DIODE_MIN_CONDUCTANCE;
                    *offset_voltage = 0.0;
                }
                else
                {
                    // Positive biased:
                    // Model the Shockley Diode Equation
                    // piecewise - as a voltage offset + resistor

                    // First: Solve the Schokley Diode Equation
                    // to find the current operating point
                    // Id = Is * (exp(Vd/n.Vt) - 1)

                    let id = DIODE_IS * ((new_voltage * DIODE_1_ON_N_VT).exp() - 1.0);

                    // Now - dI/dV will be the resistance
                    // as the current operating point
                    // dI/dV = Is * exp(Vd/n.Vt) / n.Vt

                    let didv = (DIODE_IS * DIODE_1_ON_N_VT) * (new_voltage * DIODE_1_ON_N_VT).exp();

                    // Now - solve for the new offset voltage
                    // and new conductanc
                    //
                    // C = dI/dV;
                    //
                    // dI/dV = I/V => Vr = I / (dI/dV)
                    // Voff = Vd - Vr
                    //      = Vd - (I / dI/dV)

                    *conductance = didv;
                    *offset_voltage = new_voltage - (id / didv);
                }
            },
            _ => (),
        }
    }
}