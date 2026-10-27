//! Guitar Pro 3–5 store most text in a legacy single-byte encoding (often
//! Windows-1252 for Western European scores, Windows-1251 for Cyrillic).

use encoding_rs::{WINDOWS_1251, WINDOWS_1252};

/// Decode a length-delimited field from a GP3/4/5 binary stream.
pub fn decode_gp_legacy_string(bytes: &[u8]) -> String {
    if bytes.is_empty() {
        return String::new();
    }
    if let Ok(text) = std::str::from_utf8(bytes) {
        return text.to_string();
    }

    let as_1251 = WINDOWS_1251.decode(bytes).0;
    let as_1252 = WINDOWS_1252.decode(bytes).0;

    if prefer_windows_1251(&as_1251, &as_1252) {
        as_1251.into_owned()
    } else {
        as_1252.into_owned()
    }
}

fn prefer_windows_1251(as_1251: &str, as_1252: &str) -> bool {
    let cyrillic_1251 = count_cyrillic(as_1251);
    if cyrillic_1251 == 0 {
        return false;
    }
    let cyrillic_1252 = count_cyrillic(as_1252);
    if cyrillic_1252 != 0 {
        return cyrillic_1251 > cyrillic_1252;
    }
    let letters = as_1251.chars().filter(|ch| ch.is_alphabetic()).count();
    if letters < 3 {
        return false;
    }
    if cyrillic_1251 >= 4 {
        return true;
    }
    // Short Western strings with accents (e.g. "été") decode to fake Cyrillic in CP1251.
    let western = count_western_diacritics(as_1252);
    if western >= cyrillic_1251 && letters <= 8 {
        return false;
    }
    if cyrillic_1251 > cyrillic_1252 {
        return true;
    }
    let ratio = cyrillic_1251 as f32 / letters as f32;
    cyrillic_1251 >= 2 && ratio >= 0.2
}

fn count_cyrillic(text: &str) -> usize {
    text.chars()
        .filter(|ch| ('\u{0400}'..='\u{04FF}').contains(ch))
        .count()
}

fn count_western_diacritics(text: &str) -> usize {
    text.chars()
        .filter(|ch| matches!(ch,
            'À' | 'Á' | 'Â' | 'Ã' | 'Ä' | 'Å' | 'Æ' | 'Ç' | 'È' | 'É' | 'Ê' | 'Ë' | 'Ì' | 'Í' | 'Î'
                | 'Ï' | 'Ñ' | 'Ò' | 'Ó' | 'Ô' | 'Õ' | 'Ö' | 'Ù' | 'Ú' | 'Û' | 'Ü' | 'Ý' | 'à' | 'á'
                | 'â' | 'ã' | 'ä' | 'å' | 'æ' | 'ç' | 'è' | 'é' | 'ê' | 'ë' | 'ì' | 'í' | 'î' | 'ï'
                | 'ñ' | 'ò' | 'ó' | 'ô' | 'õ' | 'ö' | 'ù' | 'ú' | 'û' | 'ü' | 'ý' | 'ÿ'
        ))
        .count()
}

#[cfg(test)]
mod tests {
    use super::decode_gp_legacy_string;

    #[test]
    fn utf8_passthrough() {
        assert_eq!(decode_gp_legacy_string("Привет".as_bytes()), "Привет");
    }

    #[test]
    fn windows_1252_western() {
        let bytes = [0x46, 0x49, 0x43, 0x48, 0x49, 0x45, 0x52]; // FICHIER
        assert_eq!(decode_gp_legacy_string(&bytes), "FICHIER");
    }

    #[test]
    fn windows_1251_russian_title() {
        // "Без меня"
        let bytes = [0xC1, 0xE5, 0xE7, 0x20, 0xEC, 0xE5, 0xED, 0xFF];
        assert_eq!(decode_gp_legacy_string(&bytes), "Без меня");
    }

    #[test]
    fn windows_1251_mixed_russian_and_english() {
        let bytes = [
            193, 229, 240, 232, 32, 248, 232, 237, 229, 235, 252, 32, 40, 32, 76, 105, 107, 101, 32,
            97, 32, 114, 111, 108, 108, 105, 110, 103, 32, 115, 116, 111, 110, 101, 32, 41,
        ];
        assert_eq!(
            decode_gp_legacy_string(&bytes),
            "Бери шинель ( Like a rolling stone )"
        );
    }

    #[test]
    fn windows_1252_french_diacritics() {
        // "été" in CP1252
        let bytes = [0xE9, 0x74, 0xE9];
        assert_eq!(decode_gp_legacy_string(&bytes), "été");
    }
}
