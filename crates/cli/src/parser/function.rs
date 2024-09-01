extern crate regex;
extern crate serde;

use std::convert::TryFrom;

use self::regex::Regex;
use self::serde::{Deserialize, Serialize};

use super::error::Error;
use super::log::InnerLog;

#[derive(Debug, Serialize, Deserialize)]
pub struct Function<'a> {
    pub id: &'a str,
    #[serde(skip)]
    pub consumption_start: u32,
    #[serde(skip)]
    pub consumption_end: u32,
    pub children: Vec<InnerLog<'a>>,
}

lazy_static::lazy_static! {
    static ref RE_START: Regex = Regex::new(r"Program log: (\w+) \{\{").unwrap();
    static ref RE_CONSUMPTION: Regex = Regex::new(r"Program consumption: (\d+) units remaining").unwrap();
}

// A compute log starts with a line that contains "Program log: <fn_name> {"
// and ends with al ine that contains "Program log: } // <fn_name>"
impl Function<'_> {
    pub const LOG_COST: i32 = 409;
    pub fn new(fn_name: &'_ str) -> Function<'_> {
        Function {
            id: fn_name,
            consumption_start: 0,
            consumption_end: 0,
            children: vec![],
        }
    }

    pub fn is_end(&self, lines: &[&str]) -> bool {
        lines.len() >= 2 && lines[1].contains(self.id) && lines[1].contains("}} //")
    }

    pub fn consume_start_lines<'a>(
        &mut self,
        lines: &'a [&'a str],
    ) -> Result<&'a [&'a str], Error> {
        let captures = RE_CONSUMPTION
            .captures(lines[0])
            .ok_or(Error::Function(lines[0].to_string()))?;
        self.consumption_start = captures.get(1).unwrap().as_str().parse().unwrap();
        Ok(&lines[1..])
    }

    pub fn consume_end_lines<'a>(&mut self, lines: &'a [&'a str]) -> Result<&'a [&'a str], Error> {
        let captures = RE_CONSUMPTION
            .captures(lines[0])
            .ok_or(Error::Function(lines[0].to_string()))?;
        self.consumption_end = captures.get(1).unwrap().as_str().parse().unwrap();
        Ok(&lines[2..])
    }

    pub fn try_from_slice<'a>(
        lines: &'a [&'a str],
    ) -> Result<(Function<'a>, &'a [&'a str]), Error> {
        if lines.len() < 2 {
            return Err(Error::Function("Not enough lines".to_string()));
        }
        let mut compute = Function::try_from(lines[0])?;
        let mut lines = compute.consume_start_lines(&lines[1..])?;

        while !lines.is_empty() && !compute.is_end(lines) {
            let (inner_log, remaining_lines) = InnerLog::from_slice(lines);
            tracing::debug!("child {:?}", inner_log);
            compute.children.push(inner_log);
            lines = remaining_lines;
        }
        if lines.is_empty() {
            return Err(Error::Function("No end line".to_string()));
        }
        let lines = compute.consume_end_lines(lines)?;

        Ok((compute, lines))
    }
}

impl<'a> TryFrom<&'a str> for Function<'a> {
    type Error = Error;

    fn try_from(line: &'a str) -> Result<Self, Self::Error> {
        let captures = RE_START
            .captures(line)
            .ok_or(Error::Function(line.to_string()))?;
        let fn_name = captures.get(1).unwrap().as_str();
        Ok(Self::new(fn_name))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[tracing_test::traced_test]
    fn test_compute_from_slice() {
        const SLICE: &[&str] = &[
            "Program log: one {{",
            "Program consumption: 198708 units remaining",
            "Program consumption: 198082 units remaining",
            "Program log: }} // one",
        ];

        let (invoke, remaining_lines) = Function::try_from_slice(SLICE).unwrap();
        tracing::debug!("{:?}", invoke);
        tracing::debug!("{:?}", remaining_lines);
        assert_eq!(remaining_lines.len(), 0);
        assert_eq!(invoke.id, "one");
        assert_eq!(invoke.consumption_start, 198708);
        assert_eq!(invoke.consumption_end, 198082);

        const SLICE_WITH_GARBAGE: &[&str] = &[
            "Program log: one {{",
            "Program consumption: 198708 units remaining",
            "some garbage",
            "Program consumption: 198082 units remaining",
            "Program log: }} // one",
            "some garbage",
        ];

        let (invoke, remaining_lines) = Function::try_from_slice(SLICE_WITH_GARBAGE).unwrap();
        assert_eq!(invoke.id, "one");
        assert_eq!(invoke.consumption_start, 198708);
        assert_eq!(invoke.consumption_end, 198082);
        assert_eq!(remaining_lines.len(), 1);

        const SLICE_WITH_CHILDREN: &[&str] = &[
            "Program log: one {{",
            "Program consumption: 198708 units remaining",
            "Program log: two {{",
            "Program consumption: 197766 units remaining",
            "Program consumption: 197386 units remaining",
            "Program log: }} // two",
            "Program consumption: 196819 units remaining",
            "Program log: }} // one",
        ];

        let (invoke, remaining_lines) = Function::try_from_slice(SLICE_WITH_CHILDREN).unwrap();
        assert_eq!(invoke.id, "one");
        assert_eq!(invoke.consumption_start, 198708);
        assert_eq!(invoke.consumption_end, 196819);
        assert_eq!(remaining_lines.len(), 0);
        assert_eq!(invoke.children.len(), 1);
    }
}
