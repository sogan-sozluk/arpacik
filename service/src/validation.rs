use validator::ValidationError;

pub fn validate_password(password: &str) -> Result<(), ValidationError> {
    let mut has_whitespace = false;
    let mut has_upper = false;
    let mut has_lower = false;
    let mut has_digit = false;

    for c in password.chars() {
        has_whitespace |= c.is_whitespace();
        has_lower |= c.is_lowercase();
        has_upper |= c.is_uppercase();
        has_digit |= c.is_ascii_digit();
    }

    let is_valid = !has_whitespace && has_upper && has_lower && has_digit && password.len() >= 8;

    if !is_valid {
        return Err(ValidationError::new(
            "Parola en az bir büyük harf, bir küçük harf, bir sayı ve en az 8 karakter içermelidir",
        ));
    }

    Ok(())
}
