extern crate lazy_static;
extern crate regex;
extern crate serde;

use std::convert::TryFrom;

use self::regex::Regex;
use self::serde::{Deserialize, Serialize};

use super::{error::Error, log::InnerLog};

#[derive(Debug, Serialize, Deserialize)]
pub struct Invoke<'a> {
    pub id: &'a str,
    pub depth: u32,
    #[serde(skip)]
    pub consumption: u32,
    pub children: Vec<InnerLog<'a>>,
}

lazy_static::lazy_static! {
    static ref RE_START: Regex = Regex::new(r"Program (\w+) invoke \[(\d+)\]").unwrap();
    static ref RE_CONSUMPTION: Regex = Regex::new(r"Program (\w+) consumed (\d+) of (\d+) compute units").unwrap();
}

impl<'a> TryFrom<&'a str> for Invoke<'a> {
    type Error = Error;

    // Looks line "Program NAME invoke [DEPTH]"
    fn try_from(line: &'a str) -> Result<Self, Self::Error> {
        let captures = RE_START
            .captures(line)
            .ok_or(Error::Invoke(line.to_string()))?;
        let name = captures.get(1).unwrap().as_str();
        let depth = captures.get(2).unwrap().as_str().parse().unwrap();
        Ok(Self::new(name, depth))
    }
}

impl Invoke<'_> {
    pub const LOG_COST_CALLER: i32 = 0;
    pub const LOG_COST_INNER: i32 = 0;
    pub fn new(name: &'_ str, depth: u32) -> Invoke<'_> {
        Invoke {
            id: name,
            depth,
            consumption: 0,
            children: vec![],
        }
    }

    pub fn is_end_line(&self, line: &str) -> bool {
        line.contains(format!("Program {} success", self.id).as_str())
            || line.contains(format!("Program {} failed", self.id).as_str())
    }

    pub fn is_end(&self, lines: &[&str]) -> bool {
        (!lines.is_empty() && self.is_end_line(lines[0]))
            || (lines.len() >= 2 && self.is_end_line(lines[1]))
    }

    pub fn consume_end_lines<'a>(&mut self, lines: &'a [&'a str]) -> Result<&'a [&'a str], Error> {
        if self.is_end_line(lines[0]) {
            return Ok(&lines[1..]);
        }
        let captures = RE_CONSUMPTION
            .captures(lines[0])
            .ok_or(Error::Invoke(lines[0].to_string()))?;
        self.consumption = captures.get(2).unwrap().as_str().parse().unwrap();
        Ok(&lines[2..])
    }

    pub fn try_from_slice<'a>(lines: &'a [&'a str]) -> Result<(Invoke<'a>, &'a [&'a str]), Error> {
        if lines.len() < 2 {
            return Err(Error::Invoke("Not enough lines".to_string()));
        }
        let mut invoke = Invoke::try_from(lines[0])?;

        let mut lines = &lines[1..];
        while !lines.is_empty() && !invoke.is_end(lines) {
            let (inner_log, remaining_lines) = InnerLog::from_slice(lines);
            invoke.children.push(inner_log);
            lines = remaining_lines;
        }

        if lines.is_empty() {
            return Err(Error::Invoke("No end line".to_string()));
        }

        lines = invoke.consume_end_lines(lines)?;

        Ok((invoke, lines))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[tracing_test::traced_test]
    fn test_invoke_from_slice() {
        const SLICE: &[&str] = &[
            "Program EyXkTyKARndnKZqPXAEiP7nXRDqRXhsVXGQNW9cZudXy invoke [1]",
            "Program EyXkTyKARndnKZqPXAEiP7nXRDqRXhsVXGQNW9cZudXy consumed 3772 of 200000 compute units",
            "Program EyXkTyKARndnKZqPXAEiP7nXRDqRXhsVXGQNW9cZudXy success",
        ];

        let (invoke, remaining_lines) = Invoke::try_from_slice(SLICE).unwrap();
        tracing::debug!("{:?}", invoke);
        tracing::debug!("{:?}", remaining_lines);
        assert_eq!(invoke.id, "EyXkTyKARndnKZqPXAEiP7nXRDqRXhsVXGQNW9cZudXy");
        assert_eq!(invoke.depth, 1);
        assert_eq!(invoke.consumption, 3772);
        assert_eq!(remaining_lines.len(), 0);

        const SLICE_WITH_GARBAGE: &[&str] = &[
            "Program EyXkTyKARndnKZqPXAEiP7nXRDqRXhsVXGQNW9cZudXy invoke [1]",
            "garbage",
            "Program EyXkTyKARndnKZqPXAEiP7nXRDqRXhsVXGQNW9cZudXy consumed 3772 of 200000 compute units",
            "Program EyXkTyKARndnKZqPXAEiP7nXRDqRXhsVXGQNW9cZudXy success",
            "garbage",
        ];
        let (invoke, remaining_lines) = Invoke::try_from_slice(SLICE_WITH_GARBAGE).unwrap();
        assert_eq!(invoke.id, "EyXkTyKARndnKZqPXAEiP7nXRDqRXhsVXGQNW9cZudXy");
        assert_eq!(invoke.depth, 1);
        assert_eq!(invoke.consumption, 3772);
        assert_eq!(remaining_lines.len(), 1);

        const SLICE_WITH_CHILDREN: &[&str] = &[
            "Program EyXkTyKARndnKZqPXAEiP7nXRDqRXhsVXGQNW9cZudXy invoke [1]",
            "Program log: one {{",
            "Program consumption: 198708 units remaining",
            "Program consumption: 198708 units remaining",
            "Program log: }} // one",
            "Program log: two {{",
            "Program consumption: 197766 units remaining",
            "Program consumption: 197386 units remaining",
            "Program log: }} // two",
            "Program EyXkTyKARndnKZqPXAEiP7nXRDqRXhsVXGQNW9cZudXy consumed 3772 of 200000 compute units",
            "Program EyXkTyKARndnKZqPXAEiP7nXRDqRXhsVXGQNW9cZudXy success",
        ];

        let (invoke, remaining_lines) = Invoke::try_from_slice(SLICE_WITH_CHILDREN).unwrap();
        assert_eq!(invoke.id, "EyXkTyKARndnKZqPXAEiP7nXRDqRXhsVXGQNW9cZudXy");
        assert_eq!(invoke.depth, 1);
        assert_eq!(invoke.consumption, 3772);
        assert_eq!(remaining_lines.len(), 0);
        assert_eq!(invoke.children.len(), 2);
    }

    #[test]
    #[tracing_test::traced_test]
    fn test_parse_invoke() {
        const SLICE: &[&str] = &[
            "Program 11111111111111111111111111111111 invoke [2]",
            "Program 11111111111111111111111111111111 success",
        ];
        match Invoke::try_from_slice(SLICE) {
            Ok((invoke, remaining_lines)) => {
                tracing::debug!("{:?}", invoke);
                tracing::debug!("{:?}", remaining_lines);
            }
            Err(e) => {
                tracing::error!("{}", e);
                panic!();
            }
        }
    }
}
