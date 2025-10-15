pub enum CombinedVecResults<T> {
    Ok(Vec<T>),
    Err(syn::Error),
}

impl<T> CombinedVecResults<T> {
    pub fn into_result(self) -> Result<Vec<T>, proc_macro2::TokenStream> {
        match self {
            CombinedVecResults::Ok(v) => Ok(v),
            CombinedVecResults::Err(e) => Err(e.into_compile_error()),
        }
    }
}

impl<T> std::iter::FromIterator<Result<T, syn::Error>> for CombinedVecResults<T> {
    fn from_iter<I: IntoIterator<Item = Result<T, syn::Error>>>(iter: I) -> Self {
        let (fields, field_errors) = iter.into_iter().partition::<Vec<_>, _>(Result::is_ok);
        let field_errors = field_errors
            .into_iter()
            .map(|e| e.err().unwrap())
            .reduce(|mut a, b| {
                a.combine(b);
                a
            });
        if let Some(err) = field_errors {
            return Self::Err(err);
        }
        let fields = fields.into_iter().map(Result::unwrap).collect::<Vec<_>>();
        Self::Ok(fields)
    }
}
