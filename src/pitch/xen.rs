pub use fantazia_lib::pitch::xen::*;

#[cfg(test)]
mod tests {

    #[test]
    fn test_edo() {
        use super::edo::EDO;
        let edo = EDO::new(5407372813);
        dbg!(edo.diatonic());
    }
}
