//! List of all the dialogs in the game.

// Generates a `DialogRoot` enum where each toml file is a variant.
// It derives `strum::EnumMessage` where the detailed message is the toml file
// and regular message is the file name.
common_story_macros::embed_dialogs!();

impl DialogRoot {
    /// Parse the dialog file into a dialog graph.
    pub fn parse(self) -> super::DialogGraph {
        super::deser::subgraph_from_toml(
            self,
            toml::from_str(self.contents()).unwrap(),
        )
    }

    fn contents(self) -> &'static str {
        use strum::EnumMessage;
        self.get_detailed_message().unwrap()
    }
}

impl std::fmt::Display for DialogRoot {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{self:?}")
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
