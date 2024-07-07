//! Some per scene configurations for the top-down game that are useful to have
//! tightly coupled with the top down crate.

use crate::WhichTopDownScene;

impl WhichTopDownScene {
    /// Returns snake case version of the scene name.
    pub fn snake_case(self) -> String {
        untools::camel_to_snake(self.as_ref(), false)
    }
}
