use crate::compiler::KrezCompilerApi;
use crate::plugin::Plugin;

pub struct StdCollector {}

impl StdCollector {
    pub fn new() -> Self {
        Self {}
    }
}

impl Plugin for StdCollector {
    fn run(&mut self, _api: &mut KrezCompilerApi) {
        todo!();
    }
}
