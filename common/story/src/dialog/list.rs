//! List of all the dialogs in the game.

#[derive(PartialEq, Eq, Debug, Clone, Copy, Hash, strum::EnumIter)]
#[allow(missing_docs)]
pub enum DialogRoot {
    EnterTheApartmentElevator,
    MarieBlabbering,
}

impl DialogRoot {
    /// Parse the dialog file into a dialog graph.
    pub fn parse(self) -> super::DialogGraph {
        super::deser::subgraph_from_toml(
            self.path(),
            toml::from_str(self.contents()).unwrap(),
        )
    }

    /// Get the path to the dialog file rooted in the assets directory.
    ///
    /// TODO: this can be done with a macro
    /// TODO: explicit node names must include the file path
    fn contents(self) -> &'static str {
        use DialogRoot::*;

        match self {
            EnterTheApartmentElevator => {
                include_str!("assets/enter_the_elevator.toml")
            }
            MarieBlabbering => {
                include_str!("assets/marie_blabbering.toml")
            }
        }
    }

    fn path(self) -> &'static str {
        use DialogRoot::*;

        match self {
            EnterTheApartmentElevator => "assets/enter_the_elevator.toml",
            MarieBlabbering => "assets/marie_blabbering.toml",
        }
    }
}

impl std::fmt::Display for DialogRoot {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.path())
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
