use crate::commands::utils::structs::Command;
use crate::commands::{film, letterboxd};

pub fn all() -> Vec<Command> {
    vec![letterboxd::base(), film::base()]
}
