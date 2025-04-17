use crate::parser::error::ParseError;

pub struct SeaType {
    pointers: u8,
    name: String,
    arrays: Vec<u8>,
    funptr_args: Vec<Box<SeaType>>,
    funptr_rets: Option<Box<SeaType>>,
}

impl SeaType {
    fn from(s: String) -> Result<Self, ParseError> {
        let name = s.split_once('[').unwrap().0.trim_start_matches('^').to_string();
        let mut funptr_args: Vec<Box<SeaType>> = vec![];
        let mut funptr_rets: Option<Box<SeaType>> = None;

        if name == "fun" {
            let a = s.find('(');
            let b = s.rfind(')');
            if a.is_none() || b.is_none() {
                return Err(ParseError::FunPtrMissingParenthesis)
            }
            s[a.unwrap() .. b.unwrap()].split(',').for_each(|arg| {
                funptr_args.push(Box::from(SeaType::from(arg.to_string()).unwrap()));
            });
            if s.contains(':') {
                funptr_rets = Some(Box::from(SeaType::from(s.rsplit_once(':').unwrap().1.to_string()).unwrap()));
            }
        }

        let pointers: u8 = name.chars().take_while(|ch| *ch == '^').count().try_into().unwrap();

        let mut arrays: Vec<u8> = vec![];
        if name.contains('[') {
            name.split('[').skip(1).for_each(|it| arrays.push(it.trim_end_matches(']').parse::<u8>().unwrap()));
        }

        Ok(SeaType {
            pointers,
            name,
            arrays,
            funptr_args,
            funptr_rets
        })
    }
}
