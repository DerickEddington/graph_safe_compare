//! Things useful to unit tests, integration tests, and benchmarks.

#![forbid(unsafe_code)]


pub mod cases
{
    pub mod eq;
}

pub mod node_types
{
    pub mod borrow_pair;
    pub mod rc_pair;
    pub mod dyn_pair;
    pub mod diff_edge;
    pub mod diff_index;
}

pub mod shapes;

pub mod sizes
{
    use std::{
        env::{
            self,
            VarError,
        },
        str::FromStr,
    };

    fn get_env_var<T: FromStr>(
        name: &str,
        default: T,
    ) -> T
    {
        match env::var(name) {
            Ok(s) => match s.parse() {
                Ok(val) => val,
                Err(_) => panic!(),
            },
            Err(VarError::NotPresent) => default,
            Err(VarError::NotUnicode(_)) => panic!(),
        }
    }

    pub fn long_list_length() -> u32
    {
        const DEFAULT: u32 = if cfg!(debug_assertions) { 1_000_000 } else { 2_000_000 };

        get_env_var("MY_TEST_LONG_LIST_LENGTH", DEFAULT)
    }

    pub fn degenerate_depth() -> u32
    {
        const DEFAULT: u32 = if cfg!(debug_assertions) { 28 } else { 33 };

        get_env_var("MY_TEST_DEGENERATE_DEPTH", DEFAULT)
    }
}
