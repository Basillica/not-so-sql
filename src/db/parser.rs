use super::{query::Identifier, schema::Row};
use nom::{
    branch::alt,
    bytes::complete::{is_not, tag},
    character::complete::{alphanumeric1, char, multispace0, multispace1},
    combinator::{map, opt},
    multi::separated_list0,
    sequence::{delimited, preceded, separated_pair, tuple},
    IResult,
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
        condition: Option<String>,              // Optional WHERE condition
    },
    InsertStatement {
        table: Identifier,
        columns: Vec<String>,
        values: Vec<String>,
    },
    // InsertStateMent {
    //     table: Identifier,
    //     values: HashMap<String, String>,
    // },
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

impl<'a> Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, current: 0 }
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

    fn select_statement(input: &'a str) -> IResult<&str, ASTNode> {
        let (input, _) = tag("SELECT")(input)?; // Parse SELECT keyword
        let (input, _) = multispace1(input)?; // Parse space after SELECT
        let (input, projection) = alt((
            map(tag("*"), |_| vec![Identifier("*".to_string())]), // Parse * for all columns
            Parser::projection_list,                              // Parse column list
        ))(input)?;
        let (input, _) = multispace1(input)?; // Parse space after projection
        let (input, _) = tag("FROM")(input)?; // Parse FROM keyword
        let (input, _) = multispace1(input)?; // Parse space after FROM
        let (input, table) = Parser::identifier(input)?; // Parse table name

        let (input, _condition) = opt(preceded(
            tuple((multispace1, tag("WHERE"), multispace1)),
            is_not(";"), // Capture everything until `;` or end
        ))(input)?;
        // let (input, _cond) = Parser::parse_condition(input)?; // Parse condition dynamically

        Ok((input, ASTNode::SelectStatement { projection, table }))
    }

    fn delete_statement(input: &'a str) -> IResult<&str, ASTNode> {
        let (input, _) = tag("DELETE")(input)?; // Match "DELETE"
        let (input, _) = multispace1(input)?; // Match spaces
        let (input, _) = tag("FROM")(input)?; // Match "FROM"
        let (input, _) = multispace1(input)?; // Match spaces
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

    fn update_statement(input: &'a str) -> IResult<&str, ASTNode> {
        let (input, _) = tag("UPDATE")(input)?; // Match "UPDATE"
        let (input, _) = multispace1(input)?; // Match spaces
        let (input, table) = Parser::identifier(input)?; // Parse table name
        let (input, _) = multispace1(input)?; // Match spaces
        let (input, _) = tag("SET")(input)?; // Match "SET"
        let (input, _) = multispace1(input)?; // Match spaces

        // Parse column-value assignments
        let mut assignment = separated_list0(
            delimited(multispace0, tag(","), multispace0), // Handle commas
            separated_pair(
                Parser::identifier,
                delimited(multispace0, tag("="), multispace0),
                is_not(",;"),
            ), // col = value
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

    fn insert_statement(input: &'a str) -> IResult<&str, ASTNode> {
        let (input, _) = tag("INSERT")(input)?; // Match "INSERT"
        let (input, _) = multispace1(input)?; // Match spaces
        let (input, _) = tag("INTO")(input)?; // Match "INTO"
        let (input, _) = multispace1(input)?; // Match spaces
        let (input, table) = Parser::identifier(input)?; // Parse table name

        // Parse optional column list
        let (input, columns) = opt(delimited(
            char('('),
            separated_list0(
                delimited(multispace0, char(','), multispace0),
                Parser::identifier,
            ),
            char(')'),
        ))(input)?;

        let (input, _) = multispace1(input)?; // Match spaces
        let (input, _) = tag("VALUES")(input)?; // Match "VALUES"
        let (input, _) = multispace0(input)?; // Match optional spaces

        // Parse values
        let (input, values) = delimited(
            char('('),
            separated_list0(
                delimited(multispace0, char(','), multispace0),
                alt((alphanumeric1, preceded(char('\''), is_not("\'")))),
            ),
            char(')'),
        )(input)?;

        // Map values into a list of strings
        let values: Vec<String> = values.into_iter().map(|v| v.to_string()).collect();
        let columns: Vec<String> = columns
            .unwrap_or_else(Vec::new)
            .into_iter()
            .map(|c| c.0)
            .collect();

        Ok((
            input,
            ASTNode::InsertStatement {
                table,
                columns,
                values,
            },
        ))
    }

    fn parse_condition(input: &'a str) -> IResult<&str, Box<dyn Fn(&Row) -> bool + 'a>> {
        // Parse the column name (identifier)
        let (input, column) = alphanumeric1(input.trim())?;
        let (input, _) = multispace1(input)?;

        // Match operators (like '=', '>', '<')
        let (input, operator) = alt((tag("="), tag(">"), tag("<")))(input)?;
        let (input, _) = multispace1(input)?;

        // Parse value: Either alphanumeric or quoted string
        let (input, value) = alt((
            alphanumeric1, // Numeric or alphanumeric values like 1, 42, 'string'
            preceded(char('\''), is_not("\'")), // For quoted strings
        ))(input)?;

        // Create the closure to evaluate the condition
        let condition = move |row: &Row| {
            if let Some(row_value) = row.data.get(column) {
                match operator {
                    "=" => row_value == value,
                    ">" => {
                        row_value.parse::<i32>().unwrap_or(0) > value.parse::<i32>().unwrap_or(0)
                    }
                    "<" => {
                        row_value.parse::<i32>().unwrap_or(0) < value.parse::<i32>().unwrap_or(0)
                    }
                    _ => false,
                }
            } else {
                false
            }
        };

        Ok((input, Box::new(condition)))
    }

    pub fn parse(input: &str) -> Result<ASTNode, String> {
        let select_parser = |input| Parser::select_statement(input);
        let delete_parser = |input| Parser::delete_statement(input);
        let update_parser = |input| Parser::update_statement(input);
        let insert_parser = |input| Parser::insert_statement(input);

        let mut parsers = alt((
            select_parser,
            delete_parser,
            update_parser,
            insert_parser, // Parser::select_statement,
                           // Parser::delete_statement,
                           // Parser::update_statement,
        ));

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
