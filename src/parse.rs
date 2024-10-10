use super::{Element, Expression};
use nom::{
    branch::alt,
    character::complete::{alphanumeric1, char, satisfy},
    combinator::{map, recognize},
    multi::{many0, many1},
    sequence::{delimited, terminated, tuple},
    IResult,
};

pub fn expression(s: &str) -> IResult<&str, Vec<Element>> {
    many1(element)(s)
}

fn element(s: &str) -> IResult<&str, Element> {
    alt((exp_item, subexpression))(s)
}

fn exp_item(s: &str) -> IResult<&str, Element> {
    map(item, |i| Element::Item(i.to_string()))(s)
}

fn subexpression(s: &str) -> IResult<&str, Element> {
    map(delimited(char('('), expression, char(')')), |e| {
        Element::SubExpression(Expression(e))
    })(s)
}

pub fn declaration(s: &str) -> IResult<&str, (&str, Vec<&str>)> {
    terminated(tuple((item, many0(item))), char('='))(s)
}

fn item(s: &str) -> IResult<&str, &str> {
    alt((alphanumeric_char, curly_brace_delimited))(s)
}

fn alphanumeric_char(s: &str) -> IResult<&str, &str> {
    recognize(satisfy(|c| c.is_alphanumeric()))(s)
}

fn curly_brace_delimited(s: &str) -> IResult<&str, &str> {
    delimited(char('{'), alphanumeric1, char('}'))(s)
}
