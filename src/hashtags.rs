use strum::EnumString;

#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumString)]
pub enum FunTags {
    #[strum(serialize = "noret")]
    NoRet,
    #[strum(serialize = "inline")]
    Inline,
    #[strum(serialize = "extern")]
    Extern,
    #[strum(serialize = "static")]
    Static,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumString)]
pub enum RecTags {
    #[strum(serialize = "union")]
    Union,
    #[strum(serialize = "static")]
    Static,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumString)]
pub enum DefTags {
    #[strum(serialize = "static")]
    Static,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumString)]
pub enum TagTags {
    #[strum(serialize = "static")]
    Static,

    #[strum(serialize = "nohelpers")]
    NoHelpers,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumString)]
pub enum TagRecTags {
    #[strum(serialize = "static")]
    Static,
}
