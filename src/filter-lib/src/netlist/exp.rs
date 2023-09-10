use super::parser::{Parser, ParseError, Token, TokenKind};

#[derive(Clone, Debug)]
pub enum Exp
{
    Value(f64),
    Sum(Vec<Box<Exp>>),
    Product(Vec<Box<Exp>>),
    Sin(Box<Exp>),
    Time,
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
            Exp::Sin(freq) => (time * freq.calc(time) * 0.5 * std::f64::consts::FRAC_1_PI).sin(),
            Exp::Time => time,
        }
    }

    pub fn parse(parser: &mut Parser) -> Result<Exp, ParseError>
    {
        let mut terms = Vec::new();
        terms.push(Exp::parse_term(parser)?);

        loop
        {
            let next = parser.peek().clone();

            if let Token::Symbol(symbol) = next
            {
                if symbol == '+'
                {
                    parser.expect_symbol(symbol)?;
                    terms.push(Exp::parse_term(parser)?);
                    continue;
                }
            }
            break;
        }

        if terms.len() == 1
        {
            Ok(terms.into_iter().next().unwrap())
        }
        else
        {
            Ok(Exp::Sum(terms.into_iter().map(|t| Box::new(t)).collect()))
        }
    }

    fn parse_term(parser: &mut Parser) -> Result<Exp, ParseError>
    {
        let mut factors = Vec::new();
        factors.push(Exp::parse_factor(parser)?);

        loop
        {
            let next = parser.peek().clone();

            if let Token::Symbol(symbol) = next
            {
                if symbol == '*'
                {
                    parser.expect_symbol(symbol)?;
                    factors.push(Exp::parse_factor(parser)?);
                    continue;
                }
            }
            break;
        }

        if factors.len() == 1
        {
            Ok(factors.into_iter().next().unwrap())
        }
        else
        {
            Ok(Exp::Product(factors.into_iter().map(|f| Box::new(f)).collect()))
        }
    }

    fn parse_factor(parser: &mut Parser) -> Result<Exp, ParseError>
    {
        let location = parser.cur_location();

        match parser.peek().clone()
        {
            Token::Integer(val) =>
            {
                let _ = parser.expect(TokenKind::Integer)?;
                Ok(Exp::Value(val as f64))
            },
            Token::Value(val) =>
            {
                let _ = parser.expect(TokenKind::Value)?;
                Ok(Exp::Value(val))
            },
            Token::Ident(ident) =>
            {
                match ident.as_ref()
                {
                    "sin" =>
                    {
                        let _ = parser.expect(TokenKind::Ident);
                        let _ = parser.expect_symbol('(');
                        let result = Exp::parse(parser)?;
                        let _ = parser.expect_symbol(')');
                        Ok(Exp::Sin(Box::new(result)))
                    },
                    "t" =>
                    {
                        let _ = parser.expect(TokenKind::Ident);
                        Ok(Exp::Time)
                    },
                    _ => Err(location.into_error_named(format!("Unknown function/variable \"{}\"", ident)))
                }
            },
            _ => Err(location.into_error_named("Expected expression factor".to_owned()))
        }
    }
}