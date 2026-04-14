pub use fantazia_lib::rhythm::*;

#[cfg(test)]
mod tests {
    #[test]
    fn test_binary_duration() {
        use super::BinaryDuration;
        println!("{}", BinaryDuration::new(2, 0));
        println!("{}", BinaryDuration::new(2, 1));
        println!("{}", BinaryDuration::new(2, 2));
        println!("{}", BinaryDuration::new(3, 0));
        println!("{}", BinaryDuration::new(3, 1));
    }

    #[test]
    fn test_binary_duration_parse() {
        use super::BinaryDuration;
        let rhythm = "8. 16 | 4 4 4 | 2 8. 16 | 4 4 4 | 2 8. 16 | 4 4 4 | 4 4 8. 16 | 4 4 4 | 2.";
        rhythm
            .replace("| ", "")
            .split(' ')
            .map(|s| s.parse::<BinaryDuration>().unwrap())
            .for_each(|d| {
                dbg!(d);
            });
    }
}
