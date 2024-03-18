//! List of all the dialogs in the game.

#[derive(PartialEq, Eq, Debug, Clone, Copy, Hash, strum::EnumIter)]
#[allow(missing_docs)]
pub enum DialogRoot {
    EnterTheApartmentElevator,
}

impl DialogRoot {
    /// Parse the dialog file into a dialog graph.
    pub fn parse(self) -> super::Dialog {
        super::deser::from_toml(toml::from_str(self.contents()).unwrap())
    }

    /// Get the path to the dialog file rooted in the assets directory.
    ///
    /// TODO: this can be done with a macro
    fn contents(self) -> &'static str {
        use DialogRoot::*;

        match self {
            EnterTheApartmentElevator => {
                include_str!("assets/enter_the_elevator.toml")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use strum::IntoEnumIterator;

    use super::*;

    #[test]
    fn it_validates_dialogs() {
        for dialog in DialogRoot::iter() {
            println!("Validating {dialog:?}");
            dialog.parse();
        }
    }
}
