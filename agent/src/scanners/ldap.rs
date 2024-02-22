use crate::Dependency;
use crate::DependencyScanner;

pub struct LDAPScanner;

impl DependencyScanner for LDAPScanner {
    fn scan(&self) -> Vec<Dependency> {
        println!("Scanning database for dependencies");
        todo!()
    }
}
