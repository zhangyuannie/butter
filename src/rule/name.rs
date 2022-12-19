use rand::{prelude::ThreadRng, Rng, RngCore};

const ADJECTIVES: &[&str] = &[
    "amazing", "aquatic", "artistic", "awesome", "big", "bold", "brave", "busy", "calm",
    "charming", "cool", "dynamic", "elegant", "friendly", "great", "honest", "innocent", "jovial",
    "kind", "lucky", "magical", "nice", "optimal", "precious", "quiet", "relaxed", "sweet",
    "talented", "ultimate", "vigilant", "wild", "xenial", "yellow", "zen",
];

const NAMES: &[&str] = &[
    "alpaca", "ant", "ape", "beaver", "bison", "bug", "cat", "coyote", "crow", "deer", "dog",
    "duck", "eagle", "fish", "flamingo", "giraffe", "hamster", "horse", "ibex", "jaguar",
    "kangaroo", "koala", "llama", "leopard", "lobster", "lynx", "moose", "nautilus", "octopus",
    "owl", "ox", "panda", "penguin", "pigeon", "pony", "quagga", "rabbit", "rhino", "salmon",
    "sheep", "snake", "turtle", "unicorn", "vole", "wolf",
];

pub struct RandomName {
    buf: String,
    rng: ThreadRng,
    has_postfix: bool,
}

impl RandomName {
    pub fn new() -> Self {
        let mut rng = rand::thread_rng();
        let adj = ADJECTIVES[rng.gen_range(0..ADJECTIVES.len())];
        let name = NAMES[rng.gen_range(0..NAMES.len())];

        let mut buf = String::with_capacity(adj.len() + name.len() + 1);

        buf.push_str(adj);
        buf.push('_');
        buf.push_str(name);

        RandomName {
            buf,
            rng,
            has_postfix: false,
        }
    }

    pub fn inc_len(&mut self) {
        const CHARSET: &[u8; 32] = b"abcdefghijkmnpqrstuvwxyz23456789";
        if !self.has_postfix {
            self.buf.push('_');
            self.has_postfix = true;
        }
        let idx = self.rng.next_u32() >> (32 - 5);
        self.buf.push(CHARSET[idx as usize] as char)
    }

    pub fn as_str(&self) -> &str {
        &self.buf
    }
}
