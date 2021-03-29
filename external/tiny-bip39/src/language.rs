use crate::error::ErrorKind;
use crate::util::{Bits, Bits11};
use rustc_hash::FxHashMap;
use zeroize::Zeroize;

pub struct WordMap {
    inner: FxHashMap<&'static str, Bits11>,
}

pub struct WordList {
    inner: Vec<&'static str>,
}

impl WordMap {
    pub fn get_bits(&self, word: &str) -> Result<Bits11, ErrorKind> {
        match self.inner.get(word) {
            Some(n) => Ok(*n),
            None => Err(ErrorKind::InvalidWord)?,
        }
    }
}

impl WordList {
    pub fn get_word(&self, bits: Bits11) -> &'static str {
        self.inner[bits.bits() as usize]
    }

    pub fn get_words_by_prefix(&self, prefix: &str) -> &[&'static str] {
        let start = self.inner
            .binary_search(&prefix)
            .unwrap_or_else(|idx| idx);
        let count = self.inner[start..].iter()
            .take_while(|word| word.starts_with(prefix))
            .count();

        &self.inner[start..start + count]
    }
}

mod lazy {
    use super::{Bits11, WordList, WordMap};
    use once_cell::sync::Lazy;

    /// lazy generation of the word list
    fn gen_wordlist(lang_words: &'static str) -> WordList {
        let inner: Vec<_> = lang_words.split_whitespace().collect();

        debug_assert!(inner.len() == 2048, "Invalid wordlist length");

        WordList { inner }
    }

    /// lazy generation of the word map
    fn gen_wordmap(wordlist: &WordList) -> WordMap {
        let inner = wordlist
            .inner
            .iter()
            .enumerate()
            .map(|(i, item)| (*item, Bits11::from(i as u16)))
            .collect();

        WordMap { inner }
    }

    pub static WORDLIST_ENGLISH: Lazy<WordList> =
        Lazy::new(|| gen_wordlist(include_str!("langs/english.txt")));
    #[cfg(feature = "chinese-simplified")]
    pub static WORDLIST_CHINESE_SIMPLIFIED: Lazy<WordList> =
        Lazy::new(|| gen_wordlist(include_str!("langs/chinese_simplified.txt")));
    #[cfg(feature = "chinese-traditional")]
    pub static WORDLIST_CHINESE_TRADITIONAL: Lazy<WordList> =
        Lazy::new(|| gen_wordlist(include_str!("langs/chinese_traditional.txt")));
    #[cfg(feature = "french")]
    pub static WORDLIST_FRENCH: Lazy<WordList> =
        Lazy::new(|| gen_wordlist(include_str!("langs/french.txt")));
    #[cfg(feature = "italian")]
    pub static WORDLIST_ITALIAN: Lazy<WordList> =
        Lazy::new(|| gen_wordlist(include_str!("langs/italian.txt")));
    #[cfg(feature = "japanese")]
    pub static WORDLIST_JAPANESE: Lazy<WordList> =
        Lazy::new(|| gen_wordlist(include_str!("langs/japanese.txt")));
    #[cfg(feature = "korean")]
    pub static WORDLIST_KOREAN: Lazy<WordList> =
        Lazy::new(|| gen_wordlist(include_str!("langs/korean.txt")));
    #[cfg(feature = "spanish")]
    pub static WORDLIST_SPANISH: Lazy<WordList> =
        Lazy::new(|| gen_wordlist(include_str!("langs/spanish.txt")));

    pub static WORDMAP_ENGLISH: Lazy<WordMap> = Lazy::new(|| gen_wordmap(&WORDLIST_ENGLISH));
    #[cfg(feature = "chinese-simplified")]
    pub static WORDMAP_CHINESE_SIMPLIFIED: Lazy<WordMap> =
        Lazy::new(|| gen_wordmap(&WORDLIST_CHINESE_SIMPLIFIED));
    #[cfg(feature = "chinese-traditional")]
    pub static WORDMAP_CHINESE_TRADITIONAL: Lazy<WordMap> =
        Lazy::new(|| gen_wordmap(&WORDLIST_CHINESE_TRADITIONAL));
    #[cfg(feature = "french")]
    pub static WORDMAP_FRENCH: Lazy<WordMap> = Lazy::new(|| gen_wordmap(&WORDLIST_FRENCH));
    #[cfg(feature = "italian")]
    pub static WORDMAP_ITALIAN: Lazy<WordMap> = Lazy::new(|| gen_wordmap(&WORDLIST_ITALIAN));
    #[cfg(feature = "japanese")]
    pub static WORDMAP_JAPANESE: Lazy<WordMap> = Lazy::new(|| gen_wordmap(&WORDLIST_JAPANESE));
    #[cfg(feature = "korean")]
    pub static WORDMAP_KOREAN: Lazy<WordMap> = Lazy::new(|| gen_wordmap(&WORDLIST_KOREAN));
    #[cfg(feature = "spanish")]
    pub static WORDMAP_SPANISH: Lazy<WordMap> = Lazy::new(|| gen_wordmap(&WORDLIST_SPANISH));
}

/// The language determines which words will be used in a mnemonic phrase, but also indirectly
/// determines the binary value of each word when a [`Mnemonic`][Mnemonic] is turned into a [`Seed`][Seed].
///
/// These are not of much use right now, and may even be removed from the crate, as there is no
/// official language specified by the standard except English.
///
/// [Mnemonic]: ./mnemonic/struct.Mnemonic.html
/// [Seed]: ./seed/struct.Seed.html
#[derive(Debug, Clone, Copy, PartialEq, Zeroize)]
#[zeroize(drop)]
pub enum Language {
    English,
    #[cfg(feature = "chinese-simplified")]
    ChineseSimplified,
    #[cfg(feature = "chinese-traditional")]
    ChineseTraditional,
    #[cfg(feature = "french")]
    French,
    #[cfg(feature = "italian")]
    Italian,
    #[cfg(feature = "japanese")]
    Japanese,
    #[cfg(feature = "korean")]
    Korean,
    #[cfg(feature = "spanish")]
    Spanish,
}

impl Language {
    /// Construct a word list from its language code. Returns None
    /// if the language code is not valid or not supported.
    pub fn from_language_code(language_code: &str) -> Option<Self> {
        match &language_code.to_ascii_lowercase()[..] {
            "en" => Some(Language::English),
            #[cfg(feature = "chinese-simplified")]
            "zh-hans" => Some(Language::ChineseSimplified),
            #[cfg(feature = "chinese-traditional")]
            "zh-hant" => Some(Language::ChineseTraditional),
            #[cfg(feature = "french")]
            "fr" => Some(Language::French),
            #[cfg(feature = "italian")]
            "it" => Some(Language::Italian),
            #[cfg(feature = "japanese")]
            "ja" => Some(Language::Japanese),
            #[cfg(feature = "korean")]
            "ko" => Some(Language::Korean),
            #[cfg(feature = "spanish")]
            "es" => Some(Language::Spanish),
            _ => None,
        }
    }

    /// Get the word list for this language
    pub fn wordlist(&self) -> &'static WordList {
        match *self {
            Language::English => &lazy::WORDLIST_ENGLISH,
            #[cfg(feature = "chinese-simplified")]
            Language::ChineseSimplified => &lazy::WORDLIST_CHINESE_SIMPLIFIED,
            #[cfg(feature = "chinese-traditional")]
            Language::ChineseTraditional => &lazy::WORDLIST_CHINESE_TRADITIONAL,
            #[cfg(feature = "french")]
            Language::French => &lazy::WORDLIST_FRENCH,
            #[cfg(feature = "italian")]
            Language::Italian => &lazy::WORDLIST_ITALIAN,
            #[cfg(feature = "japanese")]
            Language::Japanese => &lazy::WORDLIST_JAPANESE,
            #[cfg(feature = "korean")]
            Language::Korean => &lazy::WORDLIST_KOREAN,
            #[cfg(feature = "spanish")]
            Language::Spanish => &lazy::WORDLIST_SPANISH,
        }
    }

    /// Get a [`WordMap`][WordMap] that allows word -> index lookups in the word list
    ///
    /// The index of an individual word in the word list is used as the binary value of that word
    /// when the phrase is turned into a [`Seed`][Seed].
    pub fn wordmap(&self) -> &'static WordMap {
        match *self {
            Language::English => &lazy::WORDMAP_ENGLISH,
            #[cfg(feature = "chinese-simplified")]
            Language::ChineseSimplified => &lazy::WORDMAP_CHINESE_SIMPLIFIED,
            #[cfg(feature = "chinese-traditional")]
            Language::ChineseTraditional => &lazy::WORDMAP_CHINESE_TRADITIONAL,
            #[cfg(feature = "french")]
            Language::French => &lazy::WORDMAP_FRENCH,
            #[cfg(feature = "italian")]
            Language::Italian => &lazy::WORDMAP_ITALIAN,
            #[cfg(feature = "japanese")]
            Language::Japanese => &lazy::WORDMAP_JAPANESE,
            #[cfg(feature = "korean")]
            Language::Korean => &lazy::WORDMAP_KOREAN,
            #[cfg(feature = "spanish")]
            Language::Spanish => &lazy::WORDMAP_SPANISH,
        }
    }
}

impl Default for Language {
    fn default() -> Language {
        Language::English
    }
}

#[cfg(test)]
mod test {
    use super::lazy;
    use super::Language;
    use super::WordList;

    #[test]
    fn words_by_prefix() {
        let wl = &lazy::WORDLIST_ENGLISH;
        let res = wl.get_words_by_prefix("woo");
        assert_eq!(res, ["wood","wool"]);
    }

    #[test]
    fn all_words_by_prefix() {
        let wl = &lazy::WORDLIST_ENGLISH;
        let res = wl.get_words_by_prefix("");
        assert_eq!(res.len(), 2048);
    }

    #[test]
    fn words_by_invalid_prefix() {
        let wl = &lazy::WORDLIST_ENGLISH;
        let res = wl.get_words_by_prefix("woof");
        assert!(res.is_empty());
    }

    fn is_wordlist_nfkd(wl: &WordList) -> bool {
        for idx in 0..2047 {
            let word = wl.get_word(idx.into());
            if !unicode_normalization::is_nfkd(word) {
                return false;
            }
        }
        return true;
    }

    #[test]
    #[cfg(feature = "chinese-simplified")]
    fn chinese_simplified_wordlist_is_nfkd() {
        assert!(is_wordlist_nfkd(&lazy::WORDLIST_CHINESE_SIMPLIFIED));
    }

    #[test]
    #[cfg(feature = "chinese-traditional")]
    fn chinese_traditional_wordlist_is_nfkd() {
        assert!(is_wordlist_nfkd(&lazy::WORDLIST_CHINESE_TRADITIONAL));
    }

    #[test]
    #[cfg(feature = "french")]
    fn french_wordlist_is_nfkd() {
        assert!(is_wordlist_nfkd(&lazy::WORDLIST_FRENCH));
    }

    #[test]
    #[cfg(feature = "italian")]
    fn italian_wordlist_is_nfkd() {
        assert!(is_wordlist_nfkd(&lazy::WORDLIST_ITALIAN));
    }

    #[test]
    #[cfg(feature = "japanese")]
    fn japanese_wordlist_is_nfkd() {
        assert!(is_wordlist_nfkd(&lazy::WORDLIST_JAPANESE));
    }

    #[test]
    #[cfg(feature = "korean")]
    fn korean_wordlist_is_nfkd() {
        assert!(is_wordlist_nfkd(&lazy::WORDLIST_KOREAN));
    }

    #[test]
    #[cfg(feature = "spanish")]
    fn spanish_wordlist_is_nfkd() {
        assert!(is_wordlist_nfkd(&lazy::WORDLIST_SPANISH));
    }

    #[test]
    fn from_language_code_en() {
        assert_eq!(
            Language::from_language_code("En").expect("en is a valid language"),
            Language::English
        );
    }

    #[test]
    #[cfg(feature = "chinese-simplified")]
    fn from_language_code_cn_hans() {
        assert_eq!(
            Language::from_language_code("Zh-Hans").expect("zh-hans is a valid language"),
            Language::ChineseSimplified
        );
    }

    #[test]
    #[cfg(feature = "chinese-traditional")]
    fn from_language_code_cn_hant() {
        assert_eq!(
            Language::from_language_code("zh-hanT").expect("zh-hant is a valid language"),
            Language::ChineseTraditional
        );
    }

    #[test]
    #[cfg(feature = "french")]
    fn from_language_code_fr() {
        assert_eq!(
            Language::from_language_code("fr").expect("fr is a valid language"),
            Language::French
        );
    }

    #[test]
    #[cfg(feature = "italian")]
    fn from_language_code_it() {
        assert_eq!(
            Language::from_language_code("It").expect("it is a valid language"),
            Language::Italian
        );
    }

    #[test]
    #[cfg(feature = "japanese")]
    fn from_language_code_ja() {
        assert_eq!(
            Language::from_language_code("Ja").expect("ja is a valid language"),
            Language::Japanese
        );
    }

    #[test]
    #[cfg(feature = "korean")]
    fn from_language_code_ko() {
        assert_eq!(
            Language::from_language_code("kO").expect("ko is a valid language"),
            Language::Korean
        );
    }

    #[test]
    #[cfg(feature = "spanish")]
    fn from_language_code_es() {
        assert_eq!(
            Language::from_language_code("ES").expect("es is a valid language"),
            Language::Spanish
        );
    }

    #[test]
    fn from_invalid_language_code() {
        assert_eq!(Language::from_language_code("not a real language"), None);
    }
}
