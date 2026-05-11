use ere::regex;

fn main() {
    // Default bind mode (Named) should reject fields with no matching capture group.
    // Only bind=None should allow unbound fields.
    #[regex(r"^(?<year>[12][0-9]{3})$")]
    struct YearOnly<'a> {
        #[group(0)]
        matched: &'a str,
        year: &'a str,
        month: Option<&'a str>,
    }

    let _ = YearOnly::exec("2024");
}
