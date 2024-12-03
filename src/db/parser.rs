use super::{query::Identifier, schema::Row};
use std::collections::HashMap;
use nom::{
    branch::alt, bytes::complete::{is_not, tag}, character::complete::{alphanumeric1, multispace0, multispace1}, combinator::{map, opt}, multi::separated_list0, sequence::{delimited, preceded, separated_pair, terminated, tuple}, IResult
};


#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Token {
    Keyword(String),
    Identifier(String),
    Literal(String),
    Operator(char),
    Whitespace,
    Comma,
    LeftParenthesis,
    RightParenthesis,
    Eof,
}


pub enum ASTNode {
    SelectStatement {
        projection: Vec<Identifier>,
        table: Identifier,
    },
    DeleteStatement {
        table: Identifier,
        condition: Option<String>, // Optional WHERE condition
    },
    UpdateStatement {
        table: Identifier,
        assignments: Vec<(Identifier, String)>, // Column assignments: col = value
        condition: Option<String>,             // Optional WHERE condition
    },
    InsertStateMent {
        table: Identifier,
        values: HashMap<String, String>
    },
    Identifier(String),
}

// pub enum ASTNode {
//     SelectStatement {
//         projection: Vec<Identifier>,
//         table: Identifier,
//     },
//     DeleteStatement {
//         table: Identifier,
//         condition: Box<dyn Fn(&Row) -> bool>, // Closure for condition
//     },
//     UpdateStatement {
//         table: Identifier,
//         updates: HashMap<String, String>,
//         condition: Box<dyn Fn(&Row) -> bool>, // Closure for condition
//     },
//     InsertStatement {
//         table: Identifier,
//         values: HashMap<String, String>,
//     },
//     Identifier(String),
// }


pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}


impl <'a> Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, current: 0 }
    }

    pub fn parse_v2(&mut self) -> ASTNode {
        self.parse_select_statement()
    }

    pub fn parse_select_statement(&mut self) -> ASTNode {
        assert_eq!(self.next_token(), Token::Keyword("SELECT".to_string()));
        let mut projection = Vec::new();
        loop {
            match self.next_token() {
                Token::Identifier(ident) => projection.push(Identifier(ident)),
                Token::Comma => continue,
                _ => break,
            }  
        }

        assert_eq!(self.next_token(), Token::Keyword("FROM".to_string()));
        let table = match self.next_token() {
            Token::Identifier(ident) => Identifier(ident),
            _ => panic!("expected identifier for the specified table name")
        };

        ASTNode::SelectStatement {
            projection,
            table
        }
    }

    fn next_token(&mut self) -> Token {
        let token = self.tokens.get(self.current).cloned().unwrap_or(Token::Eof);
        self.current += 1;
        token
    }

    fn identifier(input: &str) -> IResult<&str, Identifier> {
        map(alphanumeric1, |s: &str| Identifier(s.to_string()))(input)
    }

    /// Parses a list of projections (e.g., `col1, col2`)
    fn projection_list(input: &str) -> IResult<&str, Vec<Identifier>> {
        separated_list0(
            delimited(multispace0, tag(","), multispace0), // Handle commas with spaces
            Parser::identifier,
        )(input)
    }

    fn select_statement(&'a self, input: &'a str) -> IResult<&str, ASTNode> {
        let (input, _) = tag("SELECT")(input)?; // Parse SELECT keyword
        let (input, _) = multispace1(input)?; // Parse space after SELECT
        let (input, projection) = alt((
            map(tag("*"), |_| vec![Identifier("*".to_string())]), // Parse * for all columns
            Parser::projection_list,                                     // Parse column list
        ))(input)?;
        let (input, _) = multispace1(input)?; // Parse space after projection
        let (input, _) = tag("FROM")(input)?; // Parse FROM keyword
        let (input, _) = multispace1(input)?; // Parse space after FROM
        let (input, table) = Parser::identifier(input)?; // Parse table name
        Ok((
            input,
            ASTNode::SelectStatement {
                projection,
                table,
            },
        ))
    }

    /// simple_select_statement has been modified to handle a simple SELECT statement
    /// it expects a query to be of the form SELECT * FROM users for example
    /// 
    fn simple_select_statement(&'a self, input: &'a str) -> IResult<&str, ASTNode> {
        let (input, _) = tag("SELECT")(input)?; // SELECT
        let (input, _) = multispace0(input)?; // space
        let (input, _) = tag("*")(input)?; // *
        let (input, _) = multispace0(input)?; // space
        let (input, _) = tag("FROM")(input)?; // FROM
        let (input, _) = multispace0(input)?; // space
        let (input, table) = Parser::identifier(input)?;
    
        Ok((
            input,
            ASTNode::SelectStatement {
                projection: vec![Identifier("*".to_string())],
                table,
            },
        ))
    }

    fn delete_statement(&'a self, input: &'a str) -> IResult<&str, ASTNode> {
        let (input, _) = tag("DELETE")(input)?; // Match "DELETE"
        let (input, _) = multispace1(input)?;  // Match spaces
        let (input, _) = tag("FROM")(input)?;  // Match "FROM"
        let (input, _) = multispace1(input)?;  // Match spaces
        let (input, table) = Parser::identifier(input)?; // Parse table name
    
        // Optional WHERE clause
        let (input, condition) = opt(preceded(
            tuple((multispace1, tag("WHERE"), multispace1)),
            is_not(";"), // Capture everything until `;` or end
        ))(input)?;

        // let condition_closure = condition.unwrap_or_else(|| Box::new(|_: &Row| true));
        let (input, _cond) = Parser::parse_condition(input)?; // Parse condition dynamically
        println!("some fucking cond: {:?}", input);

    
        Ok((
            input,
            ASTNode::DeleteStatement {
                table,
                // condition: Some(condition),
                condition: condition.map(|c| c.trim().to_string()),
                // condition: Box::new(condition),
            },
        ))
    }
    

    fn update_statement(&'a self, input: &'a str) -> IResult<&str, ASTNode> {
        let (input, _) = tag("UPDATE")(input)?; // Match "UPDATE"
        let (input, _) = multispace1(input)?;  // Match spaces
        let (input, table) = Parser::identifier(input)?; // Parse table name
        let (input, _) = multispace1(input)?;  // Match spaces
        let (input, _) = tag("SET")(input)?;   // Match "SET"
        let (input, _) = multispace1(input)?;  // Match spaces
    
        // Parse column-value assignments
        let mut assignment = separated_list0(
            delimited(multispace0, tag(","), multispace0), // Handle commas
            separated_pair(Parser::identifier, delimited(multispace0, tag("="), multispace0), is_not(",;")), // col = value
        );
    
        let (input, assignments) = assignment(input)?;
    
        // Optional WHERE clause
        let (input, condition) = opt(preceded(
            tuple((multispace1, tag("WHERE"), multispace1)),
            is_not(";"), // Capture everything until `;` or end
        ))(input)?;
    
        // Map assignments into a Vec<(Identifier, String)>
        let assignments = assignments
            .into_iter()
            .map(|(col, val)| (col, val.trim().to_string()))
            .collect();
    
        Ok((
            input,
            ASTNode::UpdateStatement {
                table,
                assignments,
                condition: condition.map(|c| c.trim().to_string()),
            },
        ))
    }

    fn parse_condition(input: &str) -> IResult<&str, impl Fn(&Row) -> bool + use<'_>>  {
        let (input, column) = Parser::identifier(input)?;
        let (input, _) = multispace0(input)?;
        let (input, operator) = alt((tag("="), tag(">"), tag("<")))(input)?;
        let (input, _) = multispace0(input)?;
        let (input, value) = Parser::identifier(input)?;
    
        let column = column.0;
        let value = value.0;
    
        let condition = move |row: &Row| {
            if let Some(row_value) = row.data.get(&column) {
                match operator {
                    "=" => row_value == &value,
                    ">" => row_value.parse::<i32>().unwrap_or(0) > value.parse::<i32>().unwrap_or(0),
                    "<" => row_value.parse::<i32>().unwrap_or(0) < value.parse::<i32>().unwrap_or(0),
                    _ => false,
                }
            } else {
                false
            }
        };
    
        Ok((input, condition))
    }
    
    
    
    pub fn parse(self, input: &str) -> Result<ASTNode, String> {
        let select_parser = |input| self.select_statement(input);
        let delete_parser = |input| self.delete_statement(input);
        let update_parser = |input| self.update_statement(input);
    
        let mut parsers = alt((select_parser, delete_parser, update_parser));

        match parsers(input) {
            Ok((remaining, ast)) => {
                if remaining.trim().is_empty() {
                    Ok(ast)
                } else {
                    Err(format!("Unexpected input after query: '{}'", remaining))
                }
            }
            Err(nom::Err::Error(e)) | Err(nom::Err::Failure(e)) => {
                Err(format!("Parse error: {:?}", e))
            }
            Err(nom::Err::Incomplete(_)) => Err("Incomplete input".to_string()),
        }
    }
}