use std::str::FromStr;
use std::collections::HashSet;
use super::{Device, Exp, ParseError, NodeName, Value};
use super::parser::{Parser, Token, TokenKind};

#[derive(Debug, Clone)]
pub struct Netlist
{
    devices: Vec<Device>
}

impl Netlist
{
    pub fn nodes(&self) -> HashSet<NodeName>
    {
        self.devices.iter()
            .flat_map(|d| d.nodes())
            .collect::<HashSet<_>>()
    }

    pub fn devices(&self) -> &Vec<Device>
    {
        &self.devices
    }
}

impl FromStr for Netlist
{
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err>
    {
        let mut parser = Parser::new(s.to_owned());
        let start_location = parser.cur_location();
        let mut devices = Vec::new();
        let mut device_names = HashSet::new();
        let mut node_names = HashSet::new();
        let gnd_node_name = NodeName::gnd();

        while parser.more_lines()
        {
            let line_location = parser.cur_location();

            match parser.peek().kind()
            {
                TokenKind::Newline =>
                {
                    // Empty line
                    let _ = parser.expect(TokenKind::Newline)?;
                    continue;
                },
                TokenKind::Ident =>
                {
                    let name = parser.expect_ident()?;

                    if !device_names.insert(name.clone())
                    {
                        return Err(line_location.into_error_named("Duplicate device name".to_owned()))
                    }
                    if node_names.contains(&name)
                    {
                        return Err(line_location.into_error_named("Device name conflicts with previous node name".to_owned()));
                    }

                    let char = name.chars().next().unwrap();
                    match char
                    {
                        'C' =>
                        {
                            let (plus, minus, capacitance) = parse_two_terminal(&mut parser, &mut device_names, &mut node_names)?;
                            devices.push(Device::Capacitor { name, plus, minus, capacitance });
                        },
                        'D' =>
                        {
                            let (plus, minus) = parse_two_terminal_basic(&mut parser, &mut device_names, &mut node_names)?;
                            devices.push(Device::Diode { name, plus, minus });
                        },
                        'E' =>
                        {
                            let plus = parse_node(&mut parser, &mut device_names, &mut node_names)?;
                            let minus = parse_node(&mut parser, &mut device_names, &mut node_names)?;
                            let control_plus = parse_node(&mut parser, &mut device_names, &mut node_names)?;
                            let control_minus = parse_node(&mut parser, &mut device_names, &mut node_names)?;
                            let gain = Value::new(parser.expect_value()?);

                            devices.push(Device::Vcvs { name, plus, minus, control_plus, control_minus, gain})
                        },
                        'R' =>
                        {
                            let (plus, minus, resistance) = parse_two_terminal(&mut parser, &mut device_names, &mut node_names)?;
                            devices.push(Device::Resistor { name, plus, minus, resistance });
                        },
                        'V' =>
                        {
                            let (plus, minus, voltage) = parse_two_terminal_exp(&mut parser, &mut device_names, &mut node_names)?;
                            devices.push(Device::Voltage { name, plus, minus, voltage });
                        },
                        _ =>
                        {
                            return Err(line_location.into_error_named(format!("Unknown device type '{}'", char)));
                        },
                    }

                    parser.expect(TokenKind::Newline)?;
                },
                _ =>
                {
                    // Failure
                    parser.expect(TokenKind::Ident)?;
                }
            }
        }

        // Final checks

        if devices.is_empty()
        {
            return Err(start_location.into_error_named("Must contain at least one device".to_owned()));
        }

        if !node_names.contains(gnd_node_name.name())
        {
            return Err(start_location.into_error_named(format!("Must contain reference node \"{}\"", gnd_node_name)));
        }

        Ok(Netlist{ devices })
    }
}

fn parse_two_terminal_basic(parser: &mut Parser, device_names: &mut HashSet<String>, node_names: &mut HashSet<String>) -> Result<(NodeName, NodeName), ParseError>
{
    let plus = parse_node(parser, device_names, node_names)?;
    let minus = parse_node(parser, device_names, node_names)?;

    Ok((plus, minus))
}

fn parse_two_terminal(parser: &mut Parser, device_names: &mut HashSet<String>, node_names: &mut HashSet<String>) -> Result<(NodeName, NodeName, Value), ParseError>
{
    let plus = parse_node(parser, device_names, node_names)?;
    let minus = parse_node(parser, device_names, node_names)?;
    let value = Value::new(parser.expect_value()?);

    Ok((plus, minus, value))
}

fn parse_two_terminal_exp(parser: &mut Parser, device_names: &mut HashSet<String>, node_names: &mut HashSet<String>) -> Result<(NodeName, NodeName, Exp), ParseError>
{
    let plus = parse_node(parser, device_names, node_names)?;
    let minus = parse_node(parser, device_names, node_names)?;
    let exp = Exp::parse(parser)?;

    Ok((plus, minus, exp))
}

fn parse_node(parser: &mut Parser, device_names: &mut HashSet<String>, node_names: &mut HashSet<String>) -> Result<NodeName, ParseError>
{
    let location = parser.cur_location();

    let name = match parser.peek().clone()
    {
        Token::Integer(int) =>
        {
            parser.expect(TokenKind::Integer)?;
            format!("{}", int)
        },
        _ => return Err(location.into_error_named("Expected node name".to_owned())),
    };

    node_names.insert(name.clone());

    if device_names.contains(&name)
    {
        return Err(location.into_error_named("Node name conflicts with previous device name".to_owned()));
    }

    Ok(NodeName::new(name))
}