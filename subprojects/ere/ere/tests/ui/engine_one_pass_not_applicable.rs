use ere::regex;

fn main() {
    #[regex(r"(a+)(a+)", engine = OnePassU8)]
    struct Greedy<'a>(&'a str, &'a str, &'a str);

    let _ = Greedy::exec("aaa");
}
