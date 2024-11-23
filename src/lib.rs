use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::bytes::complete::take_until;
use nom::character::complete::multispace0;
use nom::character::complete::space0;
use nom::multi::many0;
use nom::sequence::delimited;
use nom::sequence::terminated;
use nom::sequence::tuple;
use nom::IResult;



#[derive(Debug)]
pub struct Module {
    pub preamble: String,
    pub items: Vec<ModuleItem>,
}

#[derive(Debug, PartialEq)]
pub struct Question {
    pub header: String,
    pub markdown: String,
}
impl Question {
    fn new(header: &str, markdown: &str) -> Self {
        Question {
            header: String::from(header.trim()),
            markdown: String::from(markdown.trim()),
        }
    }

    pub fn render_markdown(&self) -> &str {
        return &self.markdown;
    }
}

#[derive(Debug, PartialEq)]
pub struct Grid {
    tag: Tag,
    markdown: String,
}

impl Grid {
    fn new(tag: Tag, markdown: &str) -> Self {
        Grid {
            tag,
            markdown: String::from(markdown.trim()),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Loop {
    tag: Tag,
    markdown: String,
}
impl Loop {
    fn new(tag: Tag, markdown: &str) -> Self {
        Loop {
            tag,
            markdown: String::from(markdown.trim()),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum ModuleItem {
    Question(Question),
    Loop(Loop),
    Grid(Grid),
}

impl ModuleItem {
    fn new_question(header: &str, markdown: &str) -> Self {
        ModuleItem::Question(Question::new(header, markdown))
    }
    fn new_loop(tag: Tag, markdown: &str) -> Self {
        ModuleItem::Loop(Loop::new(tag, markdown))
    }
    fn new_grid(tag: Tag, markdown: &str) -> Self {
        ModuleItem::Grid(Grid::new(tag, markdown))
    }
}

#[derive(Debug, PartialEq)]
struct Tag {
    name: String,
    params: String,
}

impl Tag {
    fn new(name: &str, params: &str) -> Self {
        Tag {
            name: String::from(name.trim()),
            params: String::from(params.trim()),
        }
    }
}

fn take_until_next_module_item(input: &str) -> IResult<&str, &str> {
    let chars = input.char_indices();
    let mut char2 = input.chars();
    let _ = char2.next();

    for ci in chars {
        let optional_next_char = char2.next();
        if let Some(next_char) = optional_next_char {
            if ci.1 == '[' && next_char.is_uppercase() {
                //println!("--- found {}{:?}", ci.1, next_char);
                return Ok((&input[ci.0..], &input[0..ci.0]));
            }
            if ci.1 == '<' {
                // safely calculate start and end to make sure that
                // we dont hit any multi-byte characters.  If we hit a multibyte
                // character, then it is not the end of the module item.
                let start = (ci.0 + 1).min(input.len());
                let end = (ci.0 + 5).min(input.len());
                if input.is_char_boundary(end) && end > start {
                    let tag = &input[start..end];
                    if tag == "loop" || tag == "grid" {
                        //println!("--- found tag {}", tag);
                        return Ok((&input[ci.0..], &input[0..ci.0]));
                    }
                }
            }
        }
    }
    Ok(("", input))
}

fn parse_question(input: &str) -> IResult<&str, ModuleItem> {
    let (input, header) = delimited(tag("["), take_until("]"), tag("]"))(input)?;
    let (input, markdown) = take_until_next_module_item(input)?;

    let item = ModuleItem::new_question(header, markdown);
    Ok((input, item))
}

fn parse_tag(input: &str) -> IResult<&str, Tag> {
    let (input, (_x1, _x2, name, params, _x3)) = tuple((
        tag("<"),
        space0,
        alt((tag("grid"), tag("loop"))),
        take_until(">"),
        tag(">"),
    ))(input)?;

    let tag = Tag::new(name, params);
    Ok((input, tag))
}

fn parse_loop(input: &str) -> IResult<&str, &str> {
    let (input, markdown) = terminated(take_until("</loop>"), tag("</loop>"))(input)?;
    return Ok((input, markdown));
}

fn parse_grid(input: &str) -> IResult<&str, &str> {
    let (input, markdown) = terminated(take_until("</grid>"), tag("</grid>"))(input)?;
    return Ok((input, markdown));
}

fn parse_question_loop_grid(input: &str) -> IResult<&str, ModuleItem> {
    // nom nom nom any whitespace..
    let (input, _) = multispace0(input)?;
    alt((parse_question, parse_loop_grid))(input)
}

fn parse_loop_grid(input: &str) -> IResult<&str, ModuleItem> {
    let (input, tag) = parse_tag(input)?;
    match &tag.name[..] {
        "grid" => {
            let (input, markdown) = parse_grid(input)?;
            Ok((input, ModuleItem::new_grid(tag, markdown)))
        }
        "loop" => {
            let (input, markdown) = parse_loop(input)?;
            Ok((input, ModuleItem::new_loop(tag, markdown)))
        }
        _ => unreachable!(),
    }
}

pub fn parse_module(input: &str) -> IResult<&str, Module> {
    let (input, preamble) = take_until_next_module_item(input)?;

    let (input, items) = many0(parse_question_loop_grid)(input)?;
    let m = Module {
        preamble: String::from(preamble),
        items,
    };
    return Ok((input, m));
}

#[cfg(test)]
mod tests {
    use super::*;
    use regex::Regex;

    #[test]
    fn test_take_until_next_module_item() {
        println!("=== case 1: we have a question followed by a question....");
        let input =
            "This is the first question without the header [1] hi [Q2] this is the second question";
        let (input, current) = take_until_next_module_item(input).unwrap();
        assert_eq!(input, "[Q2] this is the second question");
        assert_eq!(
            current,
            "This is the first question without the header [1] hi "
        );

        println!("=== case 2: we have a question followed by a loop....");
        let input = "¿Cuántas comidas comiste hoy? <loop> [L1] boo";
        let (input, current) = take_until_next_module_item(input).unwrap();
        assert_eq!(input, "<loop> [L1] boo");
        assert_eq!(current, "¿Cuántas comidas comiste hoy? ");

        println!("=== case 3: we have a question followed by a grid....");
        let input = "¿Cuántas comidas comiste hoy? <grid> [L1] boo";
        let (input, current) = take_until_next_module_item(input).unwrap();
        assert_eq!(input, "<grid> [L1] boo");
        assert_eq!(current, "¿Cuántas comidas comiste hoy? ");

        println!("=== case 4: last question....");
        let input = "¿Cuántas comidas comiste hoy?";
        let (input, current) = take_until_next_module_item(input).unwrap();
        println!("output: {} new input: {}", current, input);
        assert_eq!(input, "");
        assert_eq!(current, "¿Cuántas comidas comiste hoy?");
    }

    #[test]
    fn test_parse_tag() {
        let r = parse_tag("<fail>[Q1]lala\n[Q2]lili</fail>");
        assert!(r.is_err());

        let input = r#"<grid id="gridid"> [GID lasdf]"#;
        let (input, tag) = parse_tag(input).unwrap();
        assert_eq!(input, " [GID lasdf]");
        assert_eq!(tag, Tag::new("grid", r#"id="gridid""#));
    }

    #[test]
    fn test_create_mi() {
        let q = Question::new("Q1 displayif=q=1", "hi there");
        assert_eq!(q.markdown, "hi there");
        let qmi = ModuleItem::Question(q);
        // q is no longer accessable.
        assert_eq!(
            qmi,
            ModuleItem::Question(Question::new("Q1 displayif=q=1", "hi there"))
        );
        // using the variable in the match would transfer ownership.
        // by using a reference, I keep ownership of the moduleitem.
        match &qmi {
            ModuleItem::Question(q) => println!("QUESTION: {:?}", q),
            ModuleItem::Grid(g) => println!("{:?}", g),
            ModuleItem::Loop(l) => println!("{:?}", l),
        };
        println!("{:?}", qmi);
    }

    #[test]
    fn test_loop() {
        assert_eq!(
            parse_loop_grid("<loop>[Q1]lala\n[Q2]lili</loop>"),
            Ok((
                "",
                ModuleItem::new_loop(Tag::new("loop", ""), "[Q1]lala\n[Q2]lili")
            ))
        );
    }
    #[test]
    fn test_grid() {
        assert_eq!(
            parse_loop_grid("<grid>[Q1]lala\n[Q2]lili</grid>"),
            Ok((
                "",
                ModuleItem::new_grid(Tag::new("grid", ""), "[Q1]lala\n[Q2]lili")
            ))
        );
    }

    #[test]
    fn test_parse_item() {
        let markdown = "[Q1] this is a test [Q2] end!!";
        let mi = ModuleItem::new_question("Q1", "this is a test");
        assert_eq!(parse_question_loop_grid(markdown), Ok(("[Q2] end!!", mi)));

        let markdown = "<loop>\n[A1]this is a test [Q2] end!!</loop>";
        let mi = ModuleItem::new_loop(Tag::new("loop", ""), "[A1]this is a test [Q2] end!!");
        assert_eq!(parse_question_loop_grid(markdown), Ok(("", mi)));

        let markdown = "<loop id=\"loopid\">\n[A1]this is a test [Q2] end!!</loop>";
        let mi = ModuleItem::new_loop(
            Tag::new("loop", r#"id="loopid""#),
            "[A1]this is a test [Q2] end!!",
        );
        assert_eq!(parse_question_loop_grid(markdown), Ok(("", mi)));

        let markdown = "<grid>\n[A1]this is a test [Q2] end!!\n</grid>";
        let mi = ModuleItem::new_grid(Tag::new("grid", ""), "[A1]this is a test [Q2] end!!");
        assert_eq!(parse_question_loop_grid(markdown), Ok(("", mi)));
    }

    #[test]
    fn test_parse_module() {
        let markdown = r#"
[Q1] This is a question
[12] A
[13] B
[Q2] This is another question
[12] A
[13] B
<grid>
[G1]this is a grid question
[G2]so is this
</grid>
<loop id="bobo">
[L1] this is loop question 1
[L2] this is loop question 2
</loop>
[Q3] One last question.
(1) bing bong
(2) hi there?

        "#;
        let input = markdown;
        let (input, module) = parse_module(input).unwrap();
        for (indx, item) in module.items.iter().enumerate() {
            println!("{}: {:?}", indx, item)
        }
        println!("remaining input:\n {}", input);
    }

    #[test]
    fn test_read_module() {
        let text = r#"
// JSON WARNING --> asdfakdj;
// JSON WARNING --> [NOTAQUESTION] bing bong...
[QUESTION1] bing bong...
[1] hi
[2] there
        "#;
        let re = Regex::new(r"//.*").unwrap();
        let clean_text = re.replace_all(text, "");
        println!("{}", clean_text);
    }
}
