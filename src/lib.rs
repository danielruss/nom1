#[allow(unused_imports)]
use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::bytes::complete::take_until;
use nom::bytes::complete::take_while;
use nom::character::complete::multispace1;
use nom::character::complete::space0;
use nom::combinator::opt;
use nom::multi::{many0, many1};
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
    questions: Vec<ModuleItem>,
}
impl Loop {
    fn new(tag: Tag, markdown: &str, questions: Vec<ModuleItem>) -> Self {
        Loop {
            tag,
            markdown: String::from(markdown.trim()),
            questions,
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
    fn new_loop(tag: Tag, markdown: &str, items: Vec<ModuleItem>) -> Self {
        ModuleItem::Loop(Loop::new(tag, markdown, items))
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
    let mut chars = input.char_indices();
    let mut char2 = input.chars();
    let _ = char2.next();

    while let Some((current_index, current_char)) = chars.next() {
        let optional_next_char = char2.next();
        match (current_char, optional_next_char) {
            // there is a comment scan until \n or EOS...
            ('/', Some('/')) => {
                while let Some((_, in_comment_char)) = chars.next() {
                    let _ = char2.next();
                    if in_comment_char == '\n' {
                        break;
                    }
                }
            }
            // hit new question?
            ('[', Some(next_char)) if next_char.is_uppercase() => {
                return Ok((&input[current_index..], &input[0..current_index]));
            }
            // hit grid/loop?
            ('<', Some(_)) => {
                // safely calculate start and end to make sure that
                // we dont hit any multi-byte characters.  If we hit a multibyte
                // character, then it is not the end of the module item.
                let start = (current_index + 1).min(input.len());
                let end = (current_index + 5).min(input.len());
                if input.is_char_boundary(end) && end > start {
                    let tag = &input[start..end];
                    if tag == "loop" || tag == "grid" {
                        //println!("--- found tag {}", tag);
                        return Ok((&input[current_index..], &input[0..current_index]));
                    }
                }
            }
            // keep going
            (_, _) => {}
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

fn parse_loop(input: &str) -> IResult<&str, (&str, Vec<ModuleItem>)> {
    let (input, markdown) = terminated(take_until("</loop>"), tag("</loop>"))(input)?;
    let loop_input = markdown;
    println!("in parse_loop: ======= \n loop input:\n{:?}", loop_input);
    let (_, items) = many0(parse_question_loop_grid)(loop_input)?;
    println!("items:\n {:?}", items);

    Ok((input, (markdown, items)))
}

fn parse_grid(input: &str) -> IResult<&str, &str> {
    let (input, markdown) = terminated(take_until("</grid>"), tag("</grid>"))(input)?;
    Ok((input, markdown))
}

fn comment(input: &str) -> IResult<&str, &str> {
    println!("==============================>{}<===============", input);
    let (input, (_, comment, _)) =
        tuple((tag("//"), take_while(|c| c != '\n'), opt(tag("\n"))))(input)?;
    Ok((input, comment))
}

fn parse_whitespace_or_comment(input: &str) -> IResult<&str, &str> {
    println!("Parsing input: '{}'", input);
    //let (input, _) = many0(alt((multispace0, comment)))(input)?;
    let (input, _) = many0(alt((comment, multispace1)))(input)?;
    println!("Remaining input: '{}'", input);
    Ok((input, ""))
}

fn parse_question_loop_grid(input: &str) -> IResult<&str, ModuleItem> {
    // nom nom nom any whitespace..
    println!(
        "in parse_question_loop_grid: before nomming off ws...\n{}",
        input
    );
    let (input, _) = parse_whitespace_or_comment(input)?;
    println!(
        "in parse_question_loop_grid: after nomming off ws...\n{}",
        input
    );
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
            println!("in parse_loop_grid... (loop) \n{}", input);
            let (input, (markdown, questions)) = parse_loop(input)?;
            Ok((input, ModuleItem::new_loop(tag, markdown, questions)))
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
    #[allow(unused_imports)]
    use nom::{character::complete::alpha0, error::Error};
    use nom::{character::complete::multispace0, sequence::preceded, Err};
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
                ModuleItem::new_loop(
                    Tag::new("loop", ""),
                    "[Q1]lala\n[Q2]lili",
                    vec![
                        ModuleItem::new_question("Q1", "lala\n"),
                        ModuleItem::new_question("Q2", "lili")
                    ]
                )
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
        let mi = ModuleItem::new_loop(
            Tag::new("loop", ""),
            "[A1]this is a test [Q2] end!!",
            vec![
                ModuleItem::new_question("A1", "this is a test"),
                ModuleItem::new_question("Q2", "end!!"),
            ],
        );
        assert_eq!(parse_question_loop_grid(markdown), Ok(("", mi)));

        let markdown = "<loop id=\"loopid\">\n[A1]this is a test [Q2] end!!</loop>";
        let mi = ModuleItem::new_loop(
            Tag::new("loop", r#"id="loopid""#),
            "[A1]this is a test [Q2] end!!",
            vec![
                ModuleItem::new_question("A1", "this is a test"),
                ModuleItem::new_question("Q2", "end!!"),
            ],
        );
        println!("{:?}", parse_question_loop_grid(markdown));
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

    #[test]
    fn test_comment_in_preamble() {
        let input = "// this is a comment with a question header [Q1]\n[Q1] ding dong";
        let (input, preamble) = take_until_next_module_item(input).unwrap();
        assert_eq!(
            preamble,
            "// this is a comment with a question header [Q1]\n"
        );
        assert_eq!(input, "[Q1] ding dong");

        let input = "[Q1] ding dong";
        let (input, preamble) = take_until_next_module_item(input).unwrap();
        assert_eq!(input, "[Q1] ding dong");
        assert_eq!(preamble, "");

        let input = "[Q1] ding dong";
        let (input, preamble) = take_until_next_module_item(input).unwrap();
        assert_eq!(input, "[Q1] ding dong");
        assert_eq!(preamble, "");
    }

    #[test]
    fn test_parse_whitespace_or_comment() {
        let input = "// this is a comment with a question header [Q1]\n[Q1] ding dong";
        let (input, _) = parse_whitespace_or_comment(input).unwrap();
        println!("{}", input);
        assert_eq!(input, "[Q1] ding dong");
    }

    #[test]
    fn test_d1() {
        let input = r#"       
        [Q1] Question
        [1] response 1
        [2] response 2
        [3] response 3

        [Q2 displayif="lala"] Another Question
        [1] r1
        [3] r3
        [4] r4
        "#;

        fn parse_p1(input: &str) -> IResult<&str, &str> {
            let (input, _) = multispace0(input)?;
            println!("input:\n{}", input);
            let (input, q) = delimited(tag("["), take_until("]"), tag("]"))(input)?;
            Ok((input, q))
        }

        let res = parse_p1(input);
        match res {
            Ok((input, q)) => {
                println!("new input\n{}\nq:{}", input, q);
            }
            Err(e) => {
                eprintln!("{:#?}", e);
                assert!(false)
            }
        }
    }

    #[test]
    fn peeking() {
        let input = r#"
        // this is a comment
        [INTROM]Hello and welcome to the survey,  It is a prelude of what 
        you will see... 
        [Q1] Question
        [1] response 1
        [2] response 2
        [3] response 3

        [Q2 displayif="lala"] Another Question
        [1] r1
        [3] r3
        [4] r4
        "#;

        fn q_parser(input: &str) -> IResult<&str, &str> {
            println!(" The input string slice is {} chars long", input.len());
            println!(" ===> Question Parser called on {}", input);
            if input.len() < 2 {
                return Err(Err::Error(Error::new(
                    "not long enough to be a question",
                    nom::error::ErrorKind::Fail,
                )));
            }
            let mut iter = input.char_indices();
            let mut end_index = input.len();
            while let Some((indx, _)) = iter.next() {
                if indx == 0 {
                    continue;
                }
                // check if we are at a '[X'
                let mut iter2 = input.chars();
                let r1 = iter2.nth(indx);
                let r2 = iter2.next();
                if let (Some(x), Some(y)) = (r1, r2) {
                    if x == '[' && y.is_uppercase() {
                        end_index = indx;
                        break;
                    }
                }
            }
            // Dont return string slice return a QuestionType
            Ok((&input[end_index..], &input[0..end_index]))
        }
        fn parse_m(input: &str) -> IResult<&str, Vec<&str>> {
            let (input, (_, q)) = tuple((multispace0, many1(q_parser)))(input)?;
            Ok((input, q))
        }
        let res = parse_m(input);
        println!();
        match res {
            Ok((input, qs)) => {
                for q in qs {
                    println!("q:\n{}", q);
                }
                println!("new input:\n{}", input);
            }
            Err(e) => {
                eprintln!("{}", e);
                assert!(false)
            }
        }
    }
}
