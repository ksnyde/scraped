/// A singular "result" that is typically fit into a flat vector of results
#[derive(Clone, Serialize)]
pub struct FlatResult {
    /// The URL which was parsed.
    #[serde(serialize_with = "crate::util::url_to_string")]
    pub url: Url,
    /// The raw data extracted from the CSS selectors specified.
    pub data: HashMap<String, ResultKind>,
    /// Abstracted properties derived from `data` and converted to
    /// abstract JSON representation for serialization.s
    pub props: HashMap<String, Value>,
}

impl FlatResult {
    /// flattens a `ParseResults` struct from it's heirarchical structure to a
    /// vector of `FlatResult` results.
    pub fn flatten(r: &ParseResults) -> Vec<FlatResult> {
        let mut flat = vec![FlatResult {
            url: r.url.clone(),
            data: r.data.clone(),
            props: r.props.clone(),
        }];

        r.children.iter().for_each(|c| {
            FlatResult::flatten(c)
                .iter()
                .for_each(|i| flat.push(i.clone()));
        });

        flat
    }
}
