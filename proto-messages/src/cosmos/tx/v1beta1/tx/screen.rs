use std::collections::HashMap;

use super::cbor::{Cbor, CborPrimitivies};
use nutype::nutype;
use std::hash::Hash;

const SCREENS_KEY: u64 = 1;

const TITLE_KEY: u64 = 1;
const CONTENT_KEY: u64 = 2;
const INDENT_KEY: u64 = 3;
const EXPERT_KEY: u64 = 4;

/// Content is the text (sequence of Unicode code points) to display after
/// the Title, generally on the device's content section.
#[nutype(validate(not_empty))]
#[derive(*)]
pub struct Content(String);

/// Indent is the indentation level of the screen.
/// Zero indicates top-level.
#[nutype(validate(max = 16))]
#[derive(*)]
pub struct Indent(u8);

/// Screen is the abstract unit of Textual rendering.
#[derive(Debug)]
pub struct Screen {
    /// `title` is the text (sequence of Unicode code points) to display first,
    /// generally on the device's title section. It can be empty.
    pub title: String,

    /// `content` is the text (sequence of Unicode code points) to display after
    /// the `title`, generally on the device's content section. It must be
    /// ***non-empty***.
    pub content: Content,

    /// `indent` is the indentation level of the screen.
    /// Zero indicates top-level.
    pub indent: Indent,

    /// `expert` indicates that the screen should only be displayed
    /// via an opt-in from the user.
    pub expert: bool,
}

impl Screen {
    pub fn cbor_map(&self) -> HashMap<u64, CborPrimitivies<'_>> {
        let mut map = HashMap::new();
        if !self.title.is_empty() {
            let _ = map.insert(TITLE_KEY, CborPrimitivies::String(&self.title));
            // ignore returned
        }

        let _ = map.insert(CONTENT_KEY, CborPrimitivies::String(self.content.as_ref())); // nutype made validation that content is not empty

        if self.indent.into_inner() > 0 {
            let _ = map.insert(
                INDENT_KEY,
                CborPrimitivies::Uint64(self.indent.into_inner() as u64),
            );
        }
        if self.expert {
            let _ = map.insert(EXPERT_KEY, CborPrimitivies::Bool(self.expert));
        }

        map
    }
}

impl Cbor for &[Screen] {
    fn encode(&self, writter: &mut impl std::io::Write) -> Result<(), std::io::Error> {
        let items = self.iter().map(|this| this.cbor_map()).collect::<Vec<_>>();

        let mut hash_map = HashMap::with_capacity(1);
        let _ = hash_map.insert(SCREENS_KEY, items); // ignore returned

        hash_map.encode(writter)
    }
}

impl Cbor for Vec<Screen> {
    fn encode(&self, writter: &mut impl std::io::Write) -> Result<(), std::io::Error> {
        AsRef::<[Screen]>::as_ref(self).encode(writter)
    }
}
