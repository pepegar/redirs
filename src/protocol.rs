use std::collections::{HashMap, HashSet};

use nom::{
    bytes::complete::{take, take_until}, character::complete::{anychar, crlf, digit1}, combinator::map_res, error::Error, multi::count, IResult
};

use anyhow::{anyhow, Result};

/// Basic datatypes for RESP protocol
#[derive(Clone, Debug)]
pub enum RESP<'a> {
    SimpleString(&'a str),
    SimpleError(&'a str),
    BulkString(&'a str),
    NullBulkString,
    Integer(i64),
    Array(Vec<RESP<'a>>),
    Null,
    Boolean(bool),
    Double(f64),
    BigNumber(f64),
    BulkError(Vec<String>),
    VerbatimString(&'a str),
    Map(HashMap<RESP<'a>, RESP<'a>>),
    Set(HashSet<RESP<'a>>),
    Push(Vec<RESP<'a>>),
 }

impl <'a> PartialEq for RESP<'a> {
    fn eq(&self, other: &Self) -> bool {
        return match (self, other) {
            (RESP::SimpleString(x), RESP::SimpleString(y)) => x == y,
            (RESP::SimpleError(x), RESP::SimpleError(y)) => x == y,
            (RESP::BulkString(x), RESP::BulkString(y)) => x == y,
            (RESP::NullBulkString, RESP::NullBulkString) => true,
            (RESP::Integer(x), RESP::Integer(y)) => x == y,
            (RESP::Array(x), RESP::Array(y)) => x == y,
            (RESP::Null, RESP::Null) => true,
            (RESP::Boolean(x), RESP::Boolean(y)) => x == y,
            (RESP::Double(x), RESP::Double(y)) => x == y,
            (RESP::BigNumber(x), RESP::BigNumber(y)) => x == y,
            (RESP::BulkError(x), RESP::BulkError(y)) => x == y,
            (RESP::VerbatimString(x), RESP::VerbatimString(y)) => x == y,
            (RESP::Map(x), RESP::Map(y)) => false, // todo: IDK what to do here :/
            (RESP::Set(x), RESP::Set(y)) => false, // todo: idk what to do here
            (RESP::Push(x), RESP::Push(y)) => x == y,
            (_, _) => false
        }
    }
}

/// RESP implementation for commands
impl <'a> RESP<'a> {
    pub fn decode(input: &str) -> Result<RESP> {
        RESP::parse(input)
            .map_err(|err| anyhow!("parsing error: {:?}", err))
            .map(|(_, cmd)| cmd)
    }
    
    pub fn encode(self: &RESP<'a>) -> String {
        match self {
            RESP::SimpleString(s) => format!("+{}\r\n", s),
            RESP::SimpleError(s) => format!("-{}\r\n", s),
            RESP::BulkString(s) => format!("${}\r\n{}\r\n", s.len(), s),
            RESP::NullBulkString => "$-1\r\n".to_string(),
            RESP::Integer(i) => format!(":{}", i),
            RESP::Array(arr) =>
                format!(
                    "*{}\r\n{}",
                    arr.len(),
                    arr.iter().fold(
                        String::new(),
                        |acc, curr| acc + curr.encode().as_str()
                    )
                ),
            RESP::Null => "_\r\n".to_string(),
            RESP::Boolean(_) => todo!(),
            RESP::Double(_) => todo!(),
            RESP::BigNumber(_) => todo!(),
            RESP::BulkError(_) => todo!(),
            RESP::VerbatimString(_) => todo!(),
            RESP::Map(_) => todo!(),
            RESP::Set(_) => todo!(),
            RESP::Push(_) => todo!(),
        }
    }
    
    fn parse(input: &str) -> IResult<&str, RESP, Error<&str>> {
        let (input, first) = anychar::<&str, Error<&str>>(input).unwrap();
        
        match first {
            '+' => parse_simple_string(input),
            '-' => todo!(),
            ':' => parse_integer(input),
            '$' => parse_bulk_str(input),
            '*' => parse_array(input),
            '_' => todo!(),
            '#' => todo!(),
            ',' => todo!(),
            '(' => todo!(),
            '!' => todo!(),
            '=' => todo!(),
            '%' => todo!(),
            '~' => todo!(),
            '>' => todo!(),
            _   => todo!(),
        }
    }

}

fn parse_array(input: &str) -> IResult<&str, RESP> {
    let (input, len) = map_res(digit1, |digit_str: &str| digit_str.parse::<usize>())(input)?;
    let (input, _) = crlf(input)?;

    count(RESP::parse, len)(input)
        .map(|(input, cmds)| (input, RESP::Array(cmds)))
}

fn parse_bulk_str(input: &str) -> IResult<&str, RESP> {
    let (input, len) = map_res(digit1, |digit_str: &str| digit_str.parse::<usize>())(input)?;
    let (input, _) = crlf(input)?;
    let (input, data) = take(len)(input)?;
    let (input, _) = crlf(input)?;


    Ok((input, RESP::BulkString(data)))
}

fn parse_integer(rest: &str) -> IResult<&str, RESP> {
    todo!()
}

fn parse_simple_string(input: &str) -> IResult<&str, RESP> {
    let (input, data) = take_until::<&str, &str, Error<&str>>("\r\n")(input)?;

    Ok((input, RESP::SimpleString(data)))
}


#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_decode_simplestring() {
        let expected = RESP::SimpleString("hello");
        
        let string = "+hello\r\n";

        assert_eq!(
            expected,
            RESP::parse(string).unwrap().1
        );
    }

    #[test]
    fn test_decode_bulk() {
        let expected = RESP::BulkString("ECHO");
        
        let string = "$4\r\nECHO\r\n";

        assert_eq!(
            expected,
            RESP::parse(string).unwrap().1
        );
    }

    #[test]
    fn test_decode_array() {
        let expected = RESP::Array(
            vec![
                RESP::BulkString("ECHO"),
                RESP::BulkString("hey"),
            ]
        );
        
        let string = "*2\r\n$4\r\nECHO\r\n$3\r\nhey\r\n";

        assert_eq!(
            expected,
            RESP::parse(string).unwrap().1
        );
    }

    #[test]
    fn test_encode() {
        let cmd = RESP::Array(
            vec![
                RESP::BulkString("ECHO"),
                RESP::BulkString("hey"),
            ]
        );
        
        let expected = "*2\r\n$4\r\nECHO\r\n$3\r\nhey\r\n";

        assert_eq!(
            expected,
            cmd.encode()
        );
    }

    #[test]
    fn test_roundtrip() {
        let cmd = RESP::Array(
            vec![
                RESP::BulkString("ECHO"),
                RESP::BulkString("hey"),
            ]
        );
        
        assert_eq!(
            cmd,
            RESP::parse(cmd.encode().as_str()).unwrap().1
        );        
    }
    
}
