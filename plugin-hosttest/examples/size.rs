fn main() {
    use skillsim_engine::search::*;
    println!("Searcher {} bytes, SetResult {} bytes, Query {} bytes", core::mem::size_of::<Searcher>(), core::mem::size_of::<SetResult>(), core::mem::size_of::<Query>());
}
