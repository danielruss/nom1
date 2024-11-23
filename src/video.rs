// from youtube video: https://www.youtube.com/watch?v=t9P_aOAP2s4
#[derive(Debug, PartialEq, Clone)]
pub enum Value<'a> {
    SimpleString(&'a str),
    SimpleError(&'a str),
    Integer(i64),
    BulkString(&'a str),
    Array(Vec<Value<'a>>),
    Null,
    Boolean(bool),
}

pub fn parse_value(input: &str) -> IResult<&str, Value> {
    let (input, type_char) = one_of("+-:$*_,(!=%~")(input)?;

    let parser = match type_char {
        '+' => parse_simple_string,
        '-' => parse_simple_error,
        ':' => parse_integer,
        '$' => parse_bulk_string,
        '*' => todo!(),
        '_' => todo!(),
        ',' => todo!(),
        '(' => todo!(),
        '!' => todo!(),
        '=' => todo!(),
        '%' => todo!(),
        '~' => todo!(),
        _ => unreachable!("Invalid type_char {}", type_char),
    };
    terminated(parser, eof)(input)
}

fn crlf(input: &str) -> IResult<&str, &str> {
    tag("\r\n")(input)
}

fn parse_simple_string_raw(input: &str) -> IResult<&str, &str> {
    terminated(take_while(|c| c != '\r' && c != '\n'), crlf)(input)
}
fn parse_simple_string(input: &str) -> IResult<&str, Value> {
    let (input, value) = parse_simple_string_raw(input)?;
    Ok((input, Value::SimpleString(value)))
}

fn parse_simple_error(input: &str) -> IResult<&str, Value> {
    let (input, value) = parse_simple_string_raw(input)?;
    Ok((input, Value::SimpleError(value)))
}

fn parse_integer(input: &str) -> IResult<&str, Value> {
    let (input, value) = terminated(i64, crlf)(input)?;
    Ok((input, Value::Integer(value)))
}

// define a parser for values of -1 or greater to avoid
// having to create a custom error...
fn u64_or_minus1(input: &str) -> IResult<&str, i64> {
    let (input, value) = verify(i64, |v| *v >= -1)(input)?;
    return Ok((input, value));
}
fn parse_bulk_string(input: &str) -> IResult<&str, Value> {
    // <length>\r\n\<data>\r\n
    let (input, length) = terminated(u64_or_minus1, crlf)(input)?;
    match length {
        -1 => Ok((input, Value::Null)),
        _ => {
            let (input, value) = terminated(take(length as usize), crlf)(input)?;
            Ok((input, Value::BulkString(value)))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simplestring() {
        assert_eq!(parse_value("+OK\r\n"), Ok(("", Value::SimpleString("OK"))));
        assert_eq!(
            parse_value("+This is a simple string\r\n"),
            Ok(("", Value::SimpleString("This is a simple string")))
        );
    }

    #[test]
    fn test_simplerror() {
        assert_eq!(
            parse_value("-ERR bad\r\n"),
            Ok(("", Value::SimpleError("ERR bad")))
        );
        assert!(parse_value("-ERR bad\r\nThis should not be here").is_err())
    }

    #[test]
    fn test_int64() {
        assert_eq!(parse_value(":123\r\n"), Ok(("", Value::Integer(123))));
    }
    #[test]
    fn test_bulk_string() {
        assert_eq!(
            parse_value("$5\r\nabcde\r\n"),
            Ok(("", Value::BulkString("abcde")))
        );
        assert_eq!(parse_value("$-1\r\n"), Ok(("", Value::Null)));
        assert_eq!(parse_value("$0\r\n\r\n"), Ok(("", Value::BulkString(""))));
        assert!(parse_value("$-4\r\n").is_err());
        assert!(parse_value("$2\r\nabcde\r\n").is_err());
        assert!(parse_value("$20\r\nabcde\r\n").is_err());
    }
}
