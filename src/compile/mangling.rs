use super::type_::SeaType;

pub fn mangle_fun(id: String, params: Vec<SeaType>, rets: SeaType) -> String {
    return format!(
        "{}_{}_{}",
        id,
        params
            .iter()
            .map(|it| it.mangle())
            .collect::<Vec<String>>()
            .join(" "),
        rets.mangle()
    );
}
