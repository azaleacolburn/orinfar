pub fn change_count(c: char, count: &mut u32) {
    let c = u32::try_from(c.to_digit(10).expect("Numeric digit not in base 10")).unwrap();
    if *count == 1 {
        *count = 0;
    }

    // NOTE
    // This will fail too early I think
    if *count >= 9999 {
        return;
    }

    *count *= 10;
    *count += c;

    if *count == 0 {
        *count = 1;
    }
}
