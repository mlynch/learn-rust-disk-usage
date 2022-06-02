use crate::cli;

pub struct Context<'a> {
    pub args: cli::Args,
    pub root: &'a str
}