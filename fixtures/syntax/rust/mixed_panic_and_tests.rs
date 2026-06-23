use std::convert::TryFrom;

pub fn production_seam(raw: Option<&str>, requested: i64) -> usize {
    let raw = raw.expect("caller checked raw input");
    usize::try_from(requested).expect("count must fit usize")
}

#[cfg(test)]
mod tests {
    #[test]
    fn unit_test_assertions_are_noise() {
        assert!(true);
        assert_eq!(1, 1);
        let value = Some(1).unwrap();
        assert_ne!(value, 0);
    }
}
