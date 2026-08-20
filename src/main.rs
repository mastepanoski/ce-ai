mod error;
mod state;

fn main() {
    println!("ce-ai: compound-engineering plugin manager");
}

#[cfg(test)]
mod tests {
    #[test]
    fn sanity_check() {
        assert_eq!(2 + 2, 4);
    }
}
