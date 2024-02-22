use crate::models::Dependency;

pub trait DependencyScanner {
    fn scan(&self) -> Vec<Dependency>;
}
