#[derive(PartialEq, Eq, Debug)]
pub enum TokenKind
{
    Integer,
    Value,
    Ident,
    Symbol,
    Newline,
}

#[derive(Clone)]
pub enum Token
{
    Integer(usize),
    Value(f64),
    Ident(String),
    Symbol(char),
    Newline,
}

impl Token
{
    pub fn kind(&self) -> TokenKind
    {
        match self
        {
            Token::Integer(_) => TokenKind::Integer,
            Token::Value(_) => TokenKind::Value,
            Token::Ident(_) => TokenKind::Ident,
            Token::Symbol(_) => TokenKind::Symbol,
            Token::Newline => TokenKind::Newline,
        }
    }
}

#[derive(Debug)]
pub enum ParseErrorCondition
{
    Named(String),
    StringParse(std::string::ParseError),
    Io(std::io::Error),
}

#[derive(Debug)]
pub struct ParseLocation
{
    line_num: usize,
    column: usize,
    line: String,
}

impl ParseLocation
{
    pub fn into_error(self, condition: ParseErrorCondition) -> ParseError
    {
        ParseError
        {
            condition,
            location: self,
        }
    }
    pub fn into_error_named(self, description: String) -> ParseError
    {
        self.into_error(ParseErrorCondition::Named(description))
    }
}

#[derive(Debug)]
pub struct ParseError
{
    condition: ParseErrorCondition,
    location: ParseLocation,
}

pub struct Parser
{
    lines: Vec<String>,
    cur_line: usize,
    cur_token: usize,
    cur_line_tokens: Vec<Token>,
    cur_line_token_indexes: Vec<usize>,
}

impl Parser
{
    pub fn new(contents: String) -> Self
    {
        let lines = contents.lines().map(|l| l.to_owned()).collect::<Vec<_>>();
        let cur_line = 0;
        let cur_token = 0;

        let mut first_line = "";
        if !lines.is_empty()
        {
            first_line = &lines[0];
        }
        let (cur_line_tokens, cur_line_token_indexes) = tokenize_line(first_line);

        Parser { lines, cur_line, cur_token, cur_line_tokens, cur_line_token_indexes }
    }

    pub fn more_lines(&self) -> bool
    {
        self.cur_line < self.lines.len()
    }

    pub fn expect(&mut self, kind: TokenKind) -> Result<Token, ParseError>
    {
        let token = self.peek().clone();
        if token.kind() != kind
        {
            return Err(self.create_error_named(format!("Expected {:?}", kind)));
        }
        self.advance();
        Ok(token)
    }

    pub fn peek<'a>(&'a self) -> &'a Token
    {
        &self.cur_line_tokens[self.cur_token]
    }

    pub fn expect_ident(&mut self) -> Result<String, ParseError>
    {
        match self.peek().clone()
        {
            Token::Ident(ident) =>
            {
                self.advance();
                Ok(ident)
            },
            _ => Err(self.create_error_named("Expected identifier".to_owned()))
        }
    }

    pub fn expect_value(&mut self) -> Result<f64, ParseError>
    {
        match self.peek().clone()
        {
            Token::Integer(integer) =>
            {
                self.advance();
                Ok(integer as f64)
            },
            Token::Value(value) =>
            {
                self.advance();
                Ok(value)
            },
            _ => Err(self.create_error_named("Expected value".to_owned()))
        }
    }

    pub fn expect_symbol(&mut self, symbol: char) -> Result<(), ParseError>
    {
        match self.peek().clone()
        {
            Token::Symbol(actual) =>
            {
                if actual == symbol
                {
                    self.advance();
                    return Ok(())
                }
            },
            _ => (),
        }
        Err(self.create_error_named(format!("Expected '{}'", symbol)))
    }

    pub fn cur_location(&self) -> ParseLocation
    {
        ParseLocation
        {
            line_num: self.cur_line + 1,
            column: self.cur_line_token_indexes[self.cur_token],
            line: self.lines[self.cur_line].clone(),
        }
    }

    fn advance(&mut self)
    {
        self.cur_token += 1;
        if (self.cur_token >= self.cur_line_tokens.len())
        {
            self.cur_token = 0;
            self.cur_line += 1;

            if self.cur_line < self.lines.len()
            {
                (self.cur_line_tokens, self.cur_line_token_indexes) = tokenize_line(&self.lines[self.cur_line]);
            }
            else
            {
                (self.cur_line_tokens, self.cur_line_token_indexes) = tokenize_line("");
            }
        }
    }

    fn create_error_named(&self, str: String) -> ParseError
    {
        self.create_error(ParseErrorCondition::Named(str))
    }

    fn create_error(&self, condition: ParseErrorCondition) -> ParseError
    {
        self.cur_location().into_error(condition)
    }
}

fn tokenize_line(line: &str) -> (Vec<Token>, Vec<usize>)
{
    let mut tokens = Vec::new();
    let mut indexes = Vec::new();

    let chars = line.chars().collect::<Vec<_>>();
    let mut i = 0;

    loop
    {
        // Skip whitespace

        while (i < chars.len()) && chars[i].is_whitespace()
        {
            i += 1;
        }

        if i >= chars.len()
        {
            break;
        }

        // What to do depends on character

        let start_index = i;
        let start = chars[i];
        if start.is_ascii_digit()
        {
            let mut num = String::new();
            num.push(start);
            i += 1;

            while (i < chars.len()) && chars[i].is_numeric()
            {
                num.push(chars[i]);
                i += 1;
            }

            if (i < chars.len()) && (chars[i] == '.')
            {
                num.push(chars[i]);
                i += 1;

                while (i < chars.len()) && chars[i].is_numeric()
                {
                    num.push(chars[i]);
                    i += 1;
                }

                tokens.push(Token::Value(num.parse().unwrap()));
                indexes.push(start_index);
            }
            else // integer
            {
                tokens.push(Token::Integer(num.parse().unwrap()));
                indexes.push(start_index);
            }
        }
        else if start.is_alphabetic()
        {
            let mut ident = String::new();
            ident.push(start);
            i += 1;

            while (i < chars.len()) && chars[i].is_alphanumeric()
            {
                ident.push(chars[i]);
                i += 1;
            }

            tokens.push(Token::Ident(ident));
            indexes.push(start_index);
        }
        else
        {
            i += 1;

            tokens.push(Token::Symbol(start));
            indexes.push(start_index);
        }
    }

    tokens.push(Token::Newline);
    indexes.push(i);

    (tokens, indexes)
}