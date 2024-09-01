extern crate serde;
use self::serde::{Deserialize, Serialize};

use super::{function::Function, invoke::Invoke, log::InnerLog};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Report {
    pub naive_local: i32,
    pub naive_global: i32,
    pub local: i32,
    pub global: i32,
    #[cfg(debug_assertions)]
    pub start: i32,
    #[cfg(debug_assertions)]
    pub end: i32,
    #[cfg(debug_assertions)]
    pub n_children: i32,
}

impl From<&dyn Consumer> for Report {
    fn from(consumer: &dyn Consumer) -> Self {
        Report {
            naive_local: consumer.naive_local(),
            naive_global: consumer.naive_global(),
            local: consumer.local_ex_log(),
            global: consumer.global_ex_log(),
            #[cfg(debug_assertions)]
            start: consumer.compute_start(),
            #[cfg(debug_assertions)]
            end: consumer.compute_end(),
            #[cfg(debug_assertions)]
            n_children: consumer.number_of_children(),
        }
    }
}

/// In the report, we want to correct for the cost of logging to get a more accurate
/// representation of the compute units consumed by the program.
/// The logs we are traversing have a tree structure, where each node has a cost
/// associated with it, and a cost of producing the log.
/// For each log, there are two costs incurred:
/// 1. The cost for the caller
/// 2. The cost for the logger
///
/// For example, if our logging function looks like this:
/// ```rust,ignore
/// fn some_function_name() {
///     solana_program::log::sol_log_compute_units();
///     solana_program::msg!("Program log: some_function_name {{");
///     { // some code }
///     solana_program::msg!("}} // some_function_name");
///     solana_program::log::sol_log_compute_units();
///     }
/// }
/// ```
/// Then the cost for the caller is ONE solana_program::log::sol_log_compute_units()
/// call, and the cost for the logger is ONE solana_program::log::sol_log_compute_units()
/// call + TWO solana_program::msg!() calls.
/// The first cost is not reflected in the log itself, but the second cost is.
/// So the first cost needs to be corrected add the caller level, and the second cost
/// needs to be corrected at the logger level.
pub trait Consumer: std::fmt::Debug {
    fn compute_start(&self) -> i32;
    fn compute_end(&self) -> i32;
    fn number_of_children(&self) -> i32;
    fn children(&self) -> Vec<&dyn Consumer>;
    fn log_cost_inner(&self) -> i32;
    fn log_cost_caller(&self) -> i32;

    /// The cost of logging this log, excluding the internal cost.
    fn log_cost_outer(&self) -> i32 {
        self.log_cost_caller() - self.log_cost_inner()
    }

    /// The total cost of executing the function and all its children.
    /// This includes the inner log cost.
    /// This includes the inner log cost of its children and all their children.
    /// This includes the outer log cost of its children.
    fn naive_global(&self) -> i32 {
        self.compute_start() - self.compute_end()
    }

    /// The total cost of executing its children.
    /// This includes the inner log cost of its children and all their children.
    fn children_naive_global(&self) -> i32 {
        self.children()
            .iter()
            .map(|c| c.naive_global())
            .sum::<i32>()
    }

    /// The cost of executing the function, excluding its children,
    /// This excludes the inner log cost of its children and all their children.
    /// This does not exclude the caller log cost of its children.
    fn naive_local(&self) -> i32 {
        self.naive_global() - self.children_naive_global()
    }

    /// The cost of executing the function, excluding its children,
    /// This excludes all cost of logging.
    fn local_ex_log(&self) -> i32 {
        self.naive_local()
            - self
                .children()
                .iter()
                .map(|c| c.log_cost_outer())
                .sum::<i32>()
    }

    /// The cost of executing the function and all its children
    /// This excludes all cost of logging.
    fn global_ex_log(&self) -> i32 {
        self.local_ex_log()
            + self
                .children()
                .iter()
                .map(|c| c.global_ex_log())
                .sum::<i32>()
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

    fn log_cost_caller(&self) -> i32 {
        Self::LOG_COST_CALLER
    }

    fn log_cost_inner(&self) -> i32 {
        Self::LOG_COST_INNER
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
    fn log_cost_caller(&self) -> i32 {
        Self::LOG_COST_CALLER
    }

    fn log_cost_inner(&self) -> i32 {
        Self::LOG_COST_INNER
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

    fn log_cost_caller(&self) -> i32 {
        match self {
            InnerLog::Function(c) => c.log_cost_caller(),
            InnerLog::Invoke(i) => i.log_cost_caller(),
            InnerLog::Unknown(_) => 0,
        }
    }

    fn log_cost_inner(&self) -> i32 {
        match self {
            InnerLog::Function(c) => c.log_cost_inner(),
            InnerLog::Invoke(i) => i.log_cost_inner(),
            InnerLog::Unknown(_) => 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::log::Log;

    use super::*;
    const INPUT: &[&str] = &[
        "Program SOME_PROGRAM invoke [1]",
        "Program log: fn_one {{",
        "Program consumption: 199004 units remaining",
        "Program log: fn_two {{",
        "Program consumption: 198799 units remaining",
        "Program consumption: 198660 units remaining",
        "Program log: }} // fn_two",
        "Program log: fn_three {{",
        "Program consumption: 198331 units remaining",
        "Program log: fn_four {{",
        "Program consumption: 198092 units remaining",
        "Program 11111111111111111111111111111111 invoke [2]",
        "Program 11111111111111111111111111111111 success",
        "Program log: fn_five {{",
        "Program consumption: 194997 units remaining",
        "Program log: fn_six {{",
        "Program consumption: 194727 units remaining",
        "Program consumption: 194246 units remaining",
        "Program log: }} // fn_six",
        "Program consumption: 194041 units remaining",
        "Program log: }} // fn_five",
        "Program log: fn_seven {{",
        "Program consumption: 193729 units remaining",
        "Program consumption: 193617 units remaining",
        "Program log: }} // fn_seven",
        "Program consumption: 193409 units remaining",
        "Program log: }} // fn_four",
        "Program consumption: 193199 units remaining",
        "Program log: }} // fn_three",
        "Program consumption: 192993 units remaining",
        "Program log: }} // fn_one",
        "Program SOME_PROGRAM consumed 7218 of 200000 compute units",
        "Program SOME_PROGRAM success",
    ];

    #[test]
    #[tracing_test::traced_test]
    fn test_compute_consumption() {
        fn assert_children(log: &dyn Consumer) {
            assert!(log.naive_local() >= 0);
            assert!(log.naive_global() >= 0);
            assert!(log.local_ex_log() >= 0);
            assert!(log.global_ex_log() >= 0);
            for child in log.children() {
                assert_children(child);
            }
        }
        let log = Log::from_slice(INPUT);
        let text = serde_json::to_string_pretty(&log).unwrap();
        tracing::debug!("{}", text);
        for inner_log in log.inner_logs {
            tracing::debug!("{:?}", inner_log);
            let consumer = &inner_log as &dyn Consumer;
            assert_children(consumer);
        }
    }
}
