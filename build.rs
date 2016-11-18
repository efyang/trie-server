extern crate rand;

use std::env;
use std::fs::{File, copy};
use std::io::Write;
use std::path::Path;
use rand::Rng;

const MAX_LENGTH: usize = 20;
const DICTIONARY_FILE_NAME: &'static str = "dictionary.txt";
const DICTIONARY_UNREDUCED_SIZE: usize = 20_000_000;
const DICTIONARY_SIZE: usize = 10_000_000;

fn generate_random_string(rng: &mut rand::ThreadRng, chars: &[char]) -> String {
    let strlen = rng.gen_range(1, MAX_LENGTH);
    let mut ret = String::new();
    for _ in 0..strlen {
        ret.push(chars[rng.gen_range(0, chars.len())]);
    }
    ret
}

fn main() {
    let out_dir_str = env::var("OUT_DIR").unwrap();
    let out_dir = Path::new(&out_dir_str);
    let out_dir_p = out_dir.join(DICTIONARY_FILE_NAME);
    let target_dir_p = out_dir.parent()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join(DICTIONARY_FILE_NAME);
    if !out_dir_p.exists() || !target_dir_p.exists() {
        let mut chars_to_use = (0..26).map(|c| (c + 'a' as u8) as char).collect::<Vec<_>>();
        for c in 0..9 {
            chars_to_use.push(::std::char::from_digit(c, 10).unwrap());
        }
        chars_to_use.push('\'');
        chars_to_use.push('-');
        let mut f = File::create(&out_dir_p).unwrap();
        let mut rng = rand::thread_rng();
        let mut words = Vec::new();
        for _ in 0..DICTIONARY_UNREDUCED_SIZE {
            words.push(generate_random_string(&mut rng, &chars_to_use));
        }
        words.dedup();
        for word in words.into_iter().take(DICTIONARY_SIZE) {
            f.write((word + "\n").as_bytes())
                .unwrap();
        }
        drop(f);
        copy(out_dir_p, target_dir_p).unwrap();
    }
}
