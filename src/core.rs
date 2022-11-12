use std::collections::HashMap;

#[derive(Default)]
pub struct CiDefinition {
    pub jobs: HashMap<String, Job>,
}

#[derive(Debug, Default, PartialEq, Clone)]
pub struct Job {
    pub script: Vec<String>,
    pub variables: Vec<(String, String)>,
    pub artifacts: Vec<String>,
}
