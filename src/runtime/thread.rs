use crate::runtime::executor::interpretive;
use crate::runtime::{MetadataUnit, MethodInfo};
use std::thread::Scope;

pub struct ThreadManager<'scope, 'env> {
    scope: &'scope Scope<'scope,'env>
}

impl<'scope, 'env> ThreadManager<'scope, 'env> {
    pub const fn new(scope: &'scope Scope<'scope,'env>) -> Self {
        Self {
            scope
        }
    }

    pub fn submit_join_thread(
        &self,
        metadata: &'env MetadataUnit,
        unit: &'env MethodInfo,
        units: &'env [MetadataUnit],
    ) {
        self.scope
            .spawn(|| {
                interpretive(
                    unit.get_codes(),
                    metadata.constant_table,
                    metadata.names,
                    units,
                    metadata.globals,
                );
            })
            .join()
            .unwrap();
    }
}















