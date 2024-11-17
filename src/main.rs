use nom::bytes::complete::tag;
use nom::character::complete::i32;
use nom::character::complete::space0;
use nom::sequence::delimited;
use nom::sequence::pair;
use nom::sequence::tuple;
use nom::IResult;

#[derive(Debug)]
#[allow(dead_code)]
struct Point3D {
    x: i32,
    y: i32,
    z: i32,
}

// IResult is is the <input type,output type>
fn parse_point_0(input: &str) -> IResult<&str, Point3D> {
    // tag("Point: ") => does input start with "Point: "
    // if not: error, otherwise
    // the underscore would store the text "Point: "  which we
    // are not saving. input now stores the rest of the input text (21, 17, -11)
    let (input, _) = tag("Point: ")(input)?;
    // now remove the opening parenthesis ==> 21,17,-11)
    let (input, _) = tag("(")(input)?;
    // use the i32 to consume all digits and store the results in the varible x
    // after this => ,17, -11
    let (input, x) = i32(input)?;
    // remove the comma =>17, -11
    let (input, _) = tag(", ")(input)?;
    let (input, y) = i32(input)?;
    let (input, _) = tag(", ")(input)?;
    let (input, z) = i32(input)?;
    let (input, _) = tag(")")(input)?;

    Ok((input, Point3D { x, y, z }))
}

// Lets break down the parse_point_0 to use smaller
// parsers.

// this take a string slice and returns nothing (an empty tuple)
// the idea is that we are eating the separator.
// asserts there is a comma and 0 or more spaces.s
fn separator(input: &str) -> IResult<&str, ()> {
    let (input, _) = pair(tag(","), space0)(input)?;
    Ok((input, ()))
}

fn parse_coordinates(input: &str) -> IResult<&str, (i32, i32, i32)> {
    let (input, (x, _, y, _, z)) = tuple((i32, separator, i32, separator, i32))(input)?;
    Ok((input, (x, y, z)))
}

fn parse_point_1(input: &str) -> IResult<&str, Point3D> {
    let (input, (_, (x, y, z), _)) = tuple((tag("Point: ("), parse_coordinates, tag(")")))(input)?;
    Ok((input, Point3D { x, y, z }))
}

fn parse_point_2(input: &str) -> IResult<&str, Point3D> {
    let (input, _) = tag("Point: ")(input)?;
    let (input, (x, y, z)) = delimited(tag("("), parse_coordinates, tag(")"))(input)?;
    Ok((input, Point3D { x, y, z }))
}

fn main() {
    let text = r#"
    [X1]  This is question 1
    [X2]  This is question 2.
    "#;

    println!("Hello, world!");
    println!("{}", text);
    // Example from video:
    let s = "Point: (22, 17, -11)";
    let (_, point) = parse_point_0(s).unwrap();
    println!("parse_point_0 ==> {:?}", point);
    let (_, point) = parse_point_1(s).unwrap();
    println!("parse_point_1 ==> {:?}", point);
    let (_, point) = parse_point_2(s).unwrap();
    println!("parse_point_2 ==> {:?}", point);
}
