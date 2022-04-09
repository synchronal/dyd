use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
pub struct Manifest {
    since: String,
    remotes: HashMap<String, Remote>,
}

#[derive(Debug, Deserialize)]
pub struct Remote {
    name: String,
    origin: String,
}
