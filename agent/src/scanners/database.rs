use crate::DependencyScanner;
use crate::Dependency;

pub struct DatabaseScanner;

impl DependencyScanner for DatabaseScanner {
    fn scan(&self) -> Vec<Dependency> {
        println!("Scanning database for dependencies");
        todo!()
    }
}
