extern crate serde;
use self::serde::{Deserialize, Serialize};

use super::{function::Function, invoke::Invoke, log::InnerLog};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Report {
    pub cu_local: i32,
    pub cu_global: i32,
}

impl From<&dyn Consumer> for Report {
    fn from(consumer: &dyn Consumer) -> Self {
        consumer.comsumption()
    }
}

pub trait Consumer: std::fmt::Debug {
    fn compute_start(&self) -> i32;
    fn compute_end(&self) -> i32;
    fn number_of_children(&self) -> i32;
    fn children(&self) -> Vec<&dyn Consumer>;
    fn log_cost(&self) -> i32;
    fn global_log_cost(&self) -> i32 {
        self.children()
            .iter()
            .map(|c| c.log_cost() + c.global_log_cost())
            .sum()
    }
    fn local_log_cost(&self) -> i32 {
        self.children().iter().map(|c| c.log_cost()).sum()
    }
    fn global(&self) -> i32 {
        self.compute_start() - self.compute_end()
    }
    fn local(&self) -> i32 {
        self.global() - self.children().iter().map(|c| c.global()).sum::<i32>()
    }
    fn local_corrected(&self) -> i32 {
        self.local() - self.local_log_cost()
    }
    fn global_corrected(&self) -> i32 {
        self.global() - self.global_log_cost()
    }

    fn comsumption(&self) -> Report {
        Report {
            cu_local: self.local_corrected(),
            cu_global: self.global_corrected(),
        }
    }
}

impl Consumer for Function<'_> {
    fn compute_start(&self) -> i32 {
        self.consumption_start as i32
    }

    fn compute_end(&self) -> i32 {
        self.consumption_end as i32
    }

    fn number_of_children(&self) -> i32 {
        self.children.iter().map(|c| c.number_of_children()).sum()
    }

    fn children(&self) -> Vec<&dyn Consumer> {
        self.children.iter().map(|c| c as &dyn Consumer).collect()
    }

    fn log_cost(&self) -> i32 {
        Self::LOG_COST
    }
}

impl Consumer for Invoke<'_> {
    fn compute_start(&self) -> i32 {
        self.consumption as i32
    }

    fn compute_end(&self) -> i32 {
        0
    }

    fn number_of_children(&self) -> i32 {
        self.children.iter().map(|c| c.number_of_children()).sum()
    }
    fn children(&self) -> Vec<&dyn Consumer> {
        self.children.iter().map(|c| c as &dyn Consumer).collect()
    }
    fn log_cost(&self) -> i32 {
        Self::LOG_COST
    }
}

impl Consumer for InnerLog<'_> {
    fn compute_start(&self) -> i32 {
        match self {
            InnerLog::Function(c) => c.compute_start(),
            InnerLog::Invoke(i) => i.compute_start(),
            InnerLog::Unknown(_) => 0,
        }
    }

    fn compute_end(&self) -> i32 {
        match self {
            InnerLog::Function(c) => c.compute_end(),
            InnerLog::Invoke(i) => i.compute_end(),
            InnerLog::Unknown(_) => 0,
        }
    }

    fn number_of_children(&self) -> i32 {
        match self {
            InnerLog::Function(c) => c.number_of_children(),
            InnerLog::Invoke(i) => i.number_of_children(),
            InnerLog::Unknown(_) => 0,
        }
    }

    fn children(&self) -> Vec<&dyn Consumer> {
        match self {
            InnerLog::Function(c) => c.children(),
            InnerLog::Invoke(i) => i.children(),
            InnerLog::Unknown(_) => vec![],
        }
    }

    fn log_cost(&self) -> i32 {
        match self {
            InnerLog::Function(c) => c.log_cost(),
            InnerLog::Invoke(i) => i.log_cost(),
            InnerLog::Unknown(_) => 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::log::Log;

    use super::*;
    const INPUT: [&str; 23] = [
        "Program 8E9zGUc81rSfgtq6RaEFUF7CTJ7o5GBXwmabq6wrs9hb invoke [1]",
        "Program log: fn_one {",
        "Program consumption: 199286 units remaining",
        "Program log: fn_two {",
        "Program consumption: 199042 units remaining",
        "Program log: fn_three {",
        "Program consumption: 198708 units remaining",
        "Program consumption: 198082 units remaining",
        "Program log: } // fn_three",
        "Program log: fn_four {",
        "Program consumption: 197766 units remaining",
        "Program log: fn_five {",
        "Program consumption: 197499 units remaining",
        "Program consumption: 197386 units remaining",
        "Program log: } // fn_five",
        "Program consumption: 196819 units remaining",
        "Program log: } // fn_four",
        "Program consumption: 196604 units remaining",
        "Program log: } // fn_two",
        "Program consumption: 196399 units remaining",
        "Program log: } // fn_one",
        "Program 8E9zGUc81rSfgtq6RaEFUF7CTJ7o5GBXwmabq6wrs9hb consumed 3772 of 200000 compute units",
        "Program 8E9zGUc81rSfgtq6RaEFUF7CTJ7o5GBXwmabq6wrs9hb success",
    ];

    #[test]
    #[tracing_test::traced_test]
    fn test_compute_consumption() {
        fn log_self_and_children(log: &dyn Consumer) {
            tracing::debug!("local {:?}", log.local());
            tracing::debug!("global {:?}", log.global());
            tracing::debug!("children log cost {:?}", log.global_log_cost());
            tracing::debug!("local (corrected) {:?}", log.local_corrected());
            tracing::debug!("global (corrected) {:?}", log.global_corrected());
            println!();
            for child in log.children() {
                log_self_and_children(child);
            }
        }
        let log = Log::from_slice(&INPUT);
        let text = serde_json::to_string_pretty(&log).unwrap();
        tracing::debug!("{}", text);
        for inner_log in log.inner_logs {
            tracing::debug!("{:?}", inner_log);
            let consumer = &inner_log as &dyn Consumer;
            log_self_and_children(consumer);
        }
    }
}
