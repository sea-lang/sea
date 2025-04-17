use strum::EnumString;

#[derive(Debug, Clone, Copy, PartialEq, EnumString)]
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

#[derive(Debug, Clone, Copy, EnumString)]
pub enum RecTags {
    #[strum(serialize = "union")]
    Union,
    #[strum(serialize = "static")]
    Static,
}

#[derive(Debug, Clone, Copy, EnumString)]
pub enum TagTags {
    #[strum(serialize = "static")]
    Static,
}

#[derive(Debug, Clone, Copy, EnumString)]
pub enum TagRecTags {
    #[strum(serialize = "static")]
    Static,
}

#[derive(Debug, Clone, Copy, EnumString)]
pub enum MacTags {
    #[strum(serialize = "static")]
    Static,
}
