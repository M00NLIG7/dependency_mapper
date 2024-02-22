use crate::Config;
use crate::Dependency;
use crate::DependencyScanner;

pub struct Agent {
    config: Option<Config>,
    scanners: Vec<Box<dyn DependencyScanner>>,
}

impl Agent {
    pub fn new(scanners: Vec<Box<dyn DependencyScanner>>, config: Option<Config>) -> Self {
        Self { scanners, config }
    }

    pub fn scan(&self) -> Vec<Dependency> {
        self.scanners
            .iter()
            .flat_map(|scanner| scanner.scan())
            .collect()
    }

    pub fn add_scanner(&mut self, scanner: Box<dyn DependencyScanner>) {
        self.scanners.push(scanner);
    }

    pub fn post_results(&self) {
        println!("Posting results to server");
    }
}
