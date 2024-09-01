extern crate serde;
use self::serde::{Deserialize, Serialize};
use super::{
    consumption::{Consumer, Report},
    function::Function,
    invoke::Invoke,
};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
#[serde(tag = "type")]
pub enum InnerLog<'a> {
    #[serde(borrow)]
    Invoke(Invoke<'a>),
    Function(Function<'a>),
    Unknown(Unknown<'a>),
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
#[serde(tag = "type")]
pub enum InnerLogWrapper<'a> {
    Invoke(&'a Invoke<'a>),
    Function(&'a Function<'a>),
    Unknown(&'a Unknown<'a>),
}

impl<'a> From<&'a InnerLog<'a>> for InnerLogWrapper<'a> {
    fn from(inner_log: &'a InnerLog<'a>) -> InnerLogWrapper<'a> {
        match inner_log {
            InnerLog::Invoke(invoke) => InnerLogWrapper::Invoke(invoke),
            InnerLog::Function(function) => InnerLogWrapper::Function(function),
            InnerLog::Unknown(unknown) => InnerLogWrapper::Unknown(unknown),
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct InnerLogWithReport<'a> {
    #[serde(flatten)]
    #[serde(borrow)]
    pub inner_log: InnerLogWrapper<'a>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(flatten)]
    pub report: Option<Report>,
}

impl<'a> From<&'a InnerLog<'a>> for InnerLogWithReport<'a> {
    fn from(inner_log: &'a InnerLog<'a>) -> InnerLogWithReport<'a> {
        InnerLogWithReport {
            inner_log: InnerLogWrapper::from(inner_log),
            report: Some(Report::from(inner_log as &dyn Consumer)),
        }
    }
}

impl<'a> Serialize for InnerLog<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        InnerLogWithReport::from(self).serialize(serializer)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Unknown<'a> {
    pub line: &'a str,
}

impl<'a> Unknown<'a> {
    pub fn new(line: &'a str) -> Unknown<'a> {
        Unknown { line }
    }
}

impl<'a> InnerLog<'a> {
    pub fn from_slice(lines: &'a [&'a str]) -> (InnerLog<'a>, &'a [&'a str]) {
        match Function::try_from_slice(lines) {
            Ok((compute, remaining_lines)) => {
                return (InnerLog::Function(compute), remaining_lines);
            }
            Err(e) => {
                tracing::trace!("function error {:?}", e);
            }
        }
        match Invoke::try_from_slice(lines) {
            Ok((invoke, remaining_lines)) => {
                return (InnerLog::Invoke(invoke), remaining_lines);
            }
            Err(e) => {
                tracing::trace!("invoke error {:?}", e);
            }
        }
        tracing::trace!("inner log unknown: {:?}", lines[0]);
        (InnerLog::Unknown(Unknown::new(lines[0])), &lines[1..])
    }
}

#[derive(Debug, Deserialize)]
pub struct Log<'a> {
    #[serde(borrow)]
    pub inner_logs: Vec<InnerLog<'a>>,
}

impl<'a> Serialize for Log<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.collect_seq(self.inner_logs.iter())
    }
}

impl<'a> Log<'a> {
    pub fn new() -> Log<'a> {
        Log { inner_logs: vec![] }
    }

    pub fn from_slice(mut lines: &'a [&'a str]) -> Log<'a> {
        let mut log = Log::new();
        loop {
            if lines.is_empty() {
                break;
            }
            let (inner_log, remaining_lines) = InnerLog::from_slice(lines);
            log.inner_logs.push(inner_log);
            lines = remaining_lines;
        }
        log
    }
}

impl<'a> Default for Log<'a> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const INPUT: &[&str] = &[
      "Program CTir9Q39BN9seZcSAuHkA4i7gLCaCxyXjP7Ch6KxLEBf invoke [1]",
          "Program log: process_instruction {{",
          "Program consumption: 196528 units remaining",
              "Program log: fn_one {{",
              "Program consumption: 196323 units remaining",
              "Program consumption: 196184 units remaining",
              "Program log: }} // fn_one",
              "Program log: fn_seven {{",
              "Program consumption: 195859 units remaining",
                  "Program log: fn_one {{",
                  "Program consumption: 195647 units remaining",
                  "Program consumption: 195500 units remaining",
                  "Program log: }} // fn_one",
                  "Program log: fn_two {{",
                  "Program consumption: 194311 units remaining",
                  "Program consumption: 194209 units remaining",
                  "Program log: }} // fn_two",
                  "Program log: fn_three {{",
                  "Program consumption: 193892 units remaining",
                  "Program consumption: 193403 units remaining",
                  "Program log: }} // fn_three",
                  "Program log: some_swap {{",
                  "Program consumption: 193084 units remaining",
                      "Program 3csxHvSE7PjTDFmdtALpatt3192SXriTqLMG3MTTKvy2 invoke [2]",
                      "Program log: ray_log: ASqBC7ItBDkNrr/iwBhTo2b5Jb+yZYpnyUw3Qsx426jaVOGzgaE+qaKichZBEQRqEp5xDZ/BB+2A",
                          "Program 8TDqzm1CUgbNac1YkP81bEWmNr5683KxzsEz8DrbJb49 invoke [3]",
                          "Program log: Instruction: Transfer",
                          "Program 8TDqzm1CUgbNac1YkP81bEWmNr5683KxzsEz8DrbJb49 consumed 4736 of 168220 compute units",
                          "Program 8TDqzm1CUgbNac1YkP81bEWmNr5683KxzsEz8DrbJb49 success",
                          "Program 8TDqzm1CUgbNac1YkP81bEWmNr5683KxzsEz8DrbJb49 invoke [3]",
                          "Program log: Instruction: Transfer",
                          "Program 8TDqzm1CUgbNac1YkP81bEWmNr5683KxzsEz8DrbJb49 consumed 4645 of 160503 compute units",
                          "Program 8TDqzm1CUgbNac1YkP81bEWmNr5683KxzsEz8DrbJb49 success",
                      "Program 3csxHvSE7PjTDFmdtALpatt3192SXriTqLMG3MTTKvy2 consumed 32070 of 187125 compute units",
                      "Program 3csxHvSE7PjTDFmdtALpatt3192SXriTqLMG3MTTKvy2 success",
                  "Program consumption: 154929 units remaining",
                  "Program log: }} // some_swap",
                  "Program log: fn_six {{",
                  "Program consumption: 154609 units remaining",
                      "Program log: fn_three {{",
                      "Program consumption: 154395 units remaining",
                      "Program consumption: 153906 units remaining",
                      "Program log: }} // fn_three",
                      "Program log: fn_four {{",
                      "Program consumption: 153589 units remaining",
                          "Program 8TDqzm1CUgbNac1YkP81bEWmNr5683KxzsEz8DrbJb49 invoke [2]",
                          "Program log: Instruction: CloseAccount",
                          "Program 8TDqzm1CUgbNac1YkP81bEWmNr5683KxzsEz8DrbJb49 consumed 2997 of 151549 compute units",
                          "Program 8TDqzm1CUgbNac1YkP81bEWmNr5683KxzsEz8DrbJb49 success",
                      "Program consumption: 148404 units remaining",
                      "Program log: }} // fn_four",
                      "Program log: fn_five {{",
                      "Program consumption: 148083 units remaining",
                      "Program 11111111111111111111111111111111 invoke [2]",
                      "Program 11111111111111111111111111111111 success",
                      "Program consumption: 146136 units remaining",
                      "Program log: }} // fn_five",
                  "Program log: SomeEvent {{ version: 0, msg: \"Hello, World!\" }}",
                  "Program consumption: 141943 units remaining",
                  "Program log: }} // fn_six",
              "Program consumption: 141731 units remaining",
              "Program log: }} // fn_seven",
          "Program consumption: 141525 units remaining",
          "Program log: }} // process_instruction",
      "Program CTir9Q39BN9seZcSAuHkA4i7gLCaCxyXjP7Ch6KxLEBf consumed 59045 of 200000 compute units",
      "Program CTir9Q39BN9seZcSAuHkA4i7gLCaCxyXjP7Ch6KxLEBf success"
    ] ;

    #[test]
    #[tracing_test::traced_test]
    fn test_fn_one_slice() {
        let log = Log::from_slice(INPUT);
        tracing::debug!("{:?}", log);
        assert_eq!(log.inner_logs.len(), 1);
    }
}
