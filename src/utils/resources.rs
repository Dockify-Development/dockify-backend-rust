use serde::Deserialize;

/*
    This source file is a part of Dockify
    Dockify is licensed under the Server Side Public License (SSPL), Version 1.
    Find the LICENSE file in the root of this repository for more details.
*/
#[derive(Deserialize)]
pub struct ContainerResources {
    pub cpu_shares: i64,
    pub memory: i64,
    pub memory_swap: i64,
    pub cpu_cores: i64,
}

impl ContainerResources {
    pub fn new(memory: i64, memory_swap: i64, cpu_cores: i64, cpu_shares: i64) -> Self {
        Self {
            memory,
            memory_swap,
            cpu_cores,
            cpu_shares,
        }
    }
    pub fn calculate_price(&self) -> i64 {
        self.memory_cost() + self.swap_cost() + self.cpu_cost()
    }
    fn memory_cost(&self) -> i64 {
        bytes_to_gigabytes(self.memory) * (2) + (2)
    }
    fn swap_cost(&self) -> i64 {
        bytes_to_gigabytes(self.memory_swap) + (1)
    }
    fn cpu_cost(&self) -> i64 {
        self.cpu_cores * 15
    }
}
fn bytes_to_gigabytes(bytes: i64) -> i64 {
    bytes / (1024 * 1024 * 1024)
}
