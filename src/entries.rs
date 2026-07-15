#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ContentKind {
    #[default]
    Text,
    Html,
}

impl ContentKind {
    pub fn is_markup(self) -> bool {
        self == Self::Html
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct EntryContent {
    pub value: String,
    pub kind: ContentKind,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Entry {
    pub id: Option<String>,
    pub title: String,
    pub link: Option<String>,
    pub summary: Option<EntryContent>,
    pub content: Option<EntryContent>,
    pub published: Option<String>,
}

impl Entry {
    pub fn body(&self) -> Option<&EntryContent> {
        self.content.as_ref().or(self.summary.as_ref())
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct FeedDocument {
    pub title: String,
    pub entries: Vec<Entry>,
}
