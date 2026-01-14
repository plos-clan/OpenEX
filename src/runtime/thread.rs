use std::thread::Scope;

use crate::runtime::executor::interpretive;
use crate::runtime::{GlobalStore, MetadataUnit, MethodInfo};

pub struct ThreadManager<'scope, 'env> {
    scope: &'scope Scope<'scope, 'env>,
}

impl<'scope, 'env> ThreadManager<'scope, 'env> {
    pub const fn new(scope: &'scope Scope<'scope, 'env>) -> Self {
        Self { scope }
    }

    pub fn submit_join_thread(
        &self,
        unit_index: usize,
        metadata: &'env MetadataUnit,
        unit: &'env MethodInfo,
        units: &'env [MetadataUnit],
        globals: &'env mut GlobalStore,
    ) {
        self.scope
            .spawn(move || {
                interpretive(
                    unit.get_codes(),
                    metadata.constant_table,
                    metadata.names,
                    units,
                    unit_index,
                    metadata.globals,
                    globals,
                );
            })
            .join()
            .unwrap();
    }
}
