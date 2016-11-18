use rand::{self, Rng};

const MAX_LENGTH: usize = 20;

lazy_static! {
    static ref DICTIONARY_WORDS: Vec<&'static str> = {
        let mut v: Vec<&'static str> = include_str!(concat!(env!("OUT_DIR"), "/dictionary.txt"))
            .lines()
            .map(|s| s.trim())
            .collect();
        v.sort();
        v
    };

    static ref USABLE_CHARS: Vec<char> = {
        let mut chars_to_use = (0..26).map(|c| (c + 'a' as u8) as char).collect::<Vec<_>>();
        for c in 0..9 {
            chars_to_use.push(::std::char::from_digit(c, 10).unwrap());
        }
        chars_to_use.push('\'');
        chars_to_use.push('-');
        chars_to_use
    };
}

#[derive(Debug)]
pub struct Challenge {
    pub question: String,
    pub answer: bool,
}

impl Challenge {
    pub fn generate(rng: &mut rand::ThreadRng) -> Challenge {
        let answer = rng.gen();
        Challenge {
            question: generate_random_challenge_string(rng, &USABLE_CHARS, answer),
            answer: answer,
        }
    }
}

fn generate_random_challenge_string(rng: &mut rand::ThreadRng,
                                    chars: &[char],
                                    in_dictionary: bool)
                                    -> String {
    if !in_dictionary {
        // need to make sure that it isn't actually in the dictionary
        let mut out;
        loop {
            out = generate_random_string(rng, chars);
            if DICTIONARY_WORDS.binary_search(&out.as_str()).is_err() {
                break;
            }
        }
        out
    } else {
        DICTIONARY_WORDS[rng.gen_range(0, DICTIONARY_WORDS.len())].to_string()
    }
}

fn generate_random_string(rng: &mut rand::ThreadRng, chars: &[char]) -> String {
    let strlen = rng.gen_range(1, MAX_LENGTH);
    let mut ret = String::new();
    for _ in 0..strlen {
        ret.push(chars[rng.gen_range(0, chars.len())]);
    }
    ret
}
