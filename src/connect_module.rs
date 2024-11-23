use std::io::Read;

pub fn get_module1() -> Result<String, Box<dyn std::error::Error>> {
    let mut res = reqwest::blocking::get("https://raw.githubusercontent.com/episphere/questionnaire/refs/heads/main/prod/module1.txt")?;
    let mut body = String::new();
    res.read_to_string(&mut body)?;

    Ok(body)
}

pub fn get_connect_module(name: &str) -> Result<String, Box<dyn std::error::Error>> {
    let mut res = reqwest::blocking::get(format!(
        "https://raw.githubusercontent.com/episphere/questionnaire/refs/heads/main/prod/{}",
        name
    ))?;
    let mut body = String::new();
    res.read_to_string(&mut body)?;

    Ok(body)
}

#[cfg(test)]
mod test {
    use super::{get_connect_module, get_module1};

    #[test]
    fn test_mod1() {
        let m1 = get_module1().unwrap();
        let m2 = get_connect_module("module1.txt").unwrap();
        assert_eq!(m1, m2)
    }
}
