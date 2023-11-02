use nutype::nutype;

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
    title: String,

    /// `content` is the text (sequence of Unicode code points) to display after
    /// the `title`, generally on the device's content section. It must be
    /// ***non-empty***.
    content: Content,

    /// `indent` is the indentation level of the screen.
    /// Zero indicates top-level.
    indent: Indent,

    /// `expert` indicates that the screen should only be displayed
    /// via an opt-in from the user.
    expert: bool,
}

pub struct Cbor;

impl Screen {
    pub fn cbor(&self) -> Cbor {
        unimplemented!()
    }
}
