use nom::branch::alt;

use nom::bytes::complete::tag;
use nom::bytes::complete::take;
use nom::bytes::complete::take_until;
use nom::bytes::complete::take_while;
use nom::character::complete::i64;
use nom::character::complete::one_of;
use nom::combinator::eof;
use nom::combinator::verify;
use nom::multi::many0;
use nom::sequence::delimited;
use nom::sequence::terminated;
use nom::IResult;

#[allow(dead_code)]
fn take_until_capital_with_bracket(input: &str) -> IResult<&str, &str> {
    let mut chars = input.chars().enumerate();
    while let Some((i, c)) = chars.next() {
        if c == '[' {
            if let Some((_j, next_c)) = chars.next() {
                if next_c.is_uppercase() {
                    return Ok((&input[i..], &input[..i]));
                }
            }
        }
    }
    Ok(("", input))
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct Module {
    questions: Vec<Question>,
}

#[allow(dead_code)]
#[derive(Debug, PartialEq)]
pub struct Question {
    header: String,
    markdown: String,
}

#[derive(Debug, PartialEq)]
pub enum ModuleItem {
    Question(Question),
    Loop(String),
    Grid(String),
}

fn parse_question(input: &str) -> IResult<&str, Question> {
    let (input, header) = delimited(tag("["), take_until("]"), tag("]"))(input)?;
    let (input, markdown) = take_until_capital_with_bracket(input)?;

    Ok((
        input,
        Question {
            header: String::from(header.trim()),
            markdown: String::from(markdown.trim()),
        },
    ))
}

fn parse_loop(input: &str) -> IResult<&str, ModuleItem> {
    let (input, loop_text) =
        delimited(tag("<loop>"), take_until("</loop>"), tag("</loop>"))(input)?;
    return Ok((input, ModuleItem::Loop(String::from(loop_text))));
}

fn parse_question_loop_grid(input: &str) -> IResult<&str, ModuleItem> {
    //alt(l)
    todo!()
}

#[allow(dead_code)]
fn parse_module(input: &str) -> IResult<&str, Module> {
    let (input, prelude) = take_until_capital_with_bracket(input)?;

    let (input, questions) = many0(parse_question)(input)?;
    let m = Module { questions };

    println!(
        "prelude: {} module: {:?}, \n remaining input >>{}<<",
        prelude, m, input
    );
    return Ok((input, m));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_take_until_qs() {
        assert_eq!(
            take_until_capital_with_bracket("// comment\n[ID1, ..,]"),
            Ok(("[ID1, ..,]", "// comment\n"))
        );
        _ = parse_module("lalalalazl\n\r[Q1] this is a test [Q2] end!!");
    }

    #[test]
    fn test_loop() {
        assert_eq!(
            parse_loop("<loop>[Q1]lala\n[Q2]lili</loop>"),
            Ok(("", ModuleItem::Loop(String::from("[Q1]lala\n[Q2]lili"))))
        );
        assert_eq!(
            parse_question_loop_grid("<loop>[Q1]lala\n[Q2]lili</loop>"),
            Ok(("", ModuleItem::Loop(String::from("[Q1]lala\n[Q2]lili"))))
        );
    }
}
