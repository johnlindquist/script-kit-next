#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Emoji {
    pub emoji: &'static str,
    pub name: &'static str,
    pub keywords: &'static [&'static str],
    pub category: EmojiCategory,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EmojiCategory {
    SmileysEmotion,
    PeopleBody,
    AnimalsNature,
    FoodDrink,
    TravelPlaces,
    Activities,
    Objects,
    Symbols,
    Flags,
}

impl EmojiCategory {
    pub const fn display_name(self) -> &'static str {
        match self {
            EmojiCategory::SmileysEmotion => "Smileys & Emotion",
            EmojiCategory::PeopleBody => "People & Body",
            EmojiCategory::AnimalsNature => "Animals & Nature",
            EmojiCategory::FoodDrink => "Food & Drink",
            EmojiCategory::TravelPlaces => "Travel & Places",
            EmojiCategory::Activities => "Activities",
            EmojiCategory::Objects => "Objects",
            EmojiCategory::Symbols => "Symbols",
            EmojiCategory::Flags => "Flags",
        }
    }
}

macro_rules! emoji {
    ($emoji:expr, $name:expr, $category:expr, [$($keyword:expr),+ $(,)?]) => {
        Emoji {
            emoji: $emoji,
            name: $name,
            keywords: &[$($keyword),+],
            category: $category,
        }
    };
}

use EmojiCategory::*;

pub const ALL_CATEGORIES: &[EmojiCategory] = &[
    SmileysEmotion,
    PeopleBody,
    AnimalsNature,
    FoodDrink,
    TravelPlaces,
    Activities,
    Objects,
    Symbols,
    Flags,
];

pub const EMOJIS: &[Emoji] = &[
    emoji!(
        "😀",
        "grinning face",
        SmileysEmotion,
        ["happy", "smile", "face"]
    ),
    emoji!(
        "😃",
        "grinning face with big eyes",
        SmileysEmotion,
        ["happy", "joy", "eyes"]
    ),
    emoji!(
        "😄",
        "grinning face with smiling eyes",
        SmileysEmotion,
        ["smile", "eyes", "cheerful"]
    ),
    emoji!(
        "😁",
        "beaming face with smiling eyes",
        SmileysEmotion,
        ["beam", "smile", "teeth"]
    ),
    emoji!(
        "😆",
        "grinning squinting face",
        SmileysEmotion,
        ["laugh", "squint", "face"]
    ),
    emoji!(
        "😅",
        "grinning face with sweat",
        SmileysEmotion,
        ["relief", "sweat", "laugh"]
    ),
    emoji!(
        "😂",
        "face with tears of joy",
        SmileysEmotion,
        ["laugh", "tears", "funny"]
    ),
    emoji!(
        "🤣",
        "rolling on the floor laughing",
        SmileysEmotion,
        ["rofl", "laugh", "hilarious"]
    ),
    emoji!(
        "😊",
        "smiling face with smiling eyes",
        SmileysEmotion,
        ["blush", "smile", "warm"]
    ),
    emoji!(
        "😇",
        "smiling face with halo",
        SmileysEmotion,
        ["angel", "innocent", "halo"]
    ),
    emoji!(
        "🙂",
        "slightly smiling face",
        SmileysEmotion,
        ["smile", "friendly", "calm"]
    ),
    emoji!(
        "🙃",
        "upside-down face",
        SmileysEmotion,
        ["silly", "sarcasm", "playful"]
    ),
    emoji!(
        "😉",
        "winking face",
        SmileysEmotion,
        ["wink", "flirt", "playful"]
    ),
    emoji!(
        "😌",
        "relieved face",
        SmileysEmotion,
        ["relaxed", "calm", "relief"]
    ),
    emoji!(
        "😍",
        "smiling face with heart-eyes",
        SmileysEmotion,
        ["love", "crush", "adore"]
    ),
    emoji!(
        "🥰",
        "smiling face with hearts",
        SmileysEmotion,
        ["love", "affection", "hearts"]
    ),
    emoji!(
        "😘",
        "face blowing a kiss",
        SmileysEmotion,
        ["kiss", "love", "flirty"]
    ),
    emoji!(
        "😗",
        "kissing face",
        SmileysEmotion,
        ["kiss", "smooch", "love"]
    ),
    emoji!(
        "😙",
        "kissing face with smiling eyes",
        SmileysEmotion,
        ["kiss", "smile", "affection"]
    ),
    emoji!(
        "😚",
        "kissing face with closed eyes",
        SmileysEmotion,
        ["kiss", "shy", "love"]
    ),
    emoji!(
        "😋",
        "face savoring food",
        SmileysEmotion,
        ["yum", "food", "delicious"]
    ),
    emoji!(
        "😛",
        "face with tongue",
        SmileysEmotion,
        ["tongue", "playful", "tease"]
    ),
    emoji!(
        "😜",
        "winking face with tongue",
        SmileysEmotion,
        ["wink", "tongue", "joke"]
    ),
    emoji!(
        "🤪",
        "zany face",
        SmileysEmotion,
        ["crazy", "wild", "silly"]
    ),
    emoji!(
        "😝",
        "squinting face with tongue",
        SmileysEmotion,
        ["tongue", "silly", "tease"]
    ),
    emoji!(
        "🤑",
        "money-mouth face",
        SmileysEmotion,
        ["money", "rich", "cash"]
    ),
    emoji!(
        "🤗",
        "hugging face",
        SmileysEmotion,
        ["hug", "warm", "care"]
    ),
    emoji!(
        "🤭",
        "face with hand over mouth",
        SmileysEmotion,
        ["oops", "giggle", "surprised"]
    ),
    emoji!(
        "🤫",
        "shushing face",
        SmileysEmotion,
        ["quiet", "secret", "shh"]
    ),
    emoji!(
        "🤔",
        "thinking face",
        SmileysEmotion,
        ["think", "hmm", "question"]
    ),
    emoji!(
        "🤐",
        "zipper-mouth face",
        SmileysEmotion,
        ["silent", "secret", "zip"]
    ),
    emoji!(
        "🤨",
        "face with raised eyebrow",
        SmileysEmotion,
        ["skeptical", "doubt", "hmm"]
    ),
    emoji!(
        "😐",
        "neutral face",
        SmileysEmotion,
        ["meh", "neutral", "flat"]
    ),
    emoji!(
        "😑",
        "expressionless face",
        SmileysEmotion,
        ["blank", "deadpan", "neutral"]
    ),
    emoji!(
        "😶",
        "face without mouth",
        SmileysEmotion,
        ["speechless", "silent", "quiet"]
    ),
    emoji!(
        "😏",
        "smirking face",
        SmileysEmotion,
        ["smirk", "flirt", "sly"]
    ),
    emoji!(
        "😒",
        "unamused face",
        SmileysEmotion,
        ["annoyed", "meh", "sideeye"]
    ),
    emoji!(
        "🙄",
        "face with rolling eyes",
        SmileysEmotion,
        ["eyeroll", "annoyed", "sarcasm"]
    ),
    emoji!(
        "😬",
        "grimacing face",
        SmileysEmotion,
        ["awkward", "oops", "tense"]
    ),
    emoji!(
        "😮",
        "face with open mouth",
        SmileysEmotion,
        ["wow", "surprised", "shock"]
    ),
    emoji!(
        "😯",
        "hushed face",
        SmileysEmotion,
        ["quiet", "surprised", "wow"]
    ),
    emoji!(
        "😲",
        "astonished face",
        SmileysEmotion,
        ["astonished", "surprised", "amazed"]
    ),
    emoji!(
        "😳",
        "flushed face",
        SmileysEmotion,
        ["embarrassed", "blush", "shy"]
    ),
    emoji!(
        "🥺",
        "pleading face",
        SmileysEmotion,
        ["please", "puppy", "beg"]
    ),
    emoji!(
        "😢",
        "crying face",
        SmileysEmotion,
        ["sad", "tear", "upset"]
    ),
    emoji!(
        "😭",
        "loudly crying face",
        SmileysEmotion,
        ["cry", "sob", "sad"]
    ),
    emoji!(
        "😤",
        "face with steam from nose",
        SmileysEmotion,
        ["frustrated", "triumph", "huff"]
    ),
    emoji!(
        "😠",
        "angry face",
        SmileysEmotion,
        ["angry", "mad", "upset"]
    ),
    emoji!(
        "😡",
        "pouting face",
        SmileysEmotion,
        ["rage", "angry", "mad"]
    ),
    emoji!(
        "🤬",
        "face with symbols on mouth",
        SmileysEmotion,
        ["swear", "cursing", "rage"]
    ),
    emoji!(
        "😱",
        "face screaming in fear",
        SmileysEmotion,
        ["scared", "shock", "scream"]
    ),
    emoji!(
        "😨",
        "fearful face",
        SmileysEmotion,
        ["fear", "anxious", "scared"]
    ),
    emoji!(
        "😰",
        "anxious face with sweat",
        SmileysEmotion,
        ["stress", "anxious", "sweat"]
    ),
    emoji!(
        "😥",
        "sad but relieved face",
        SmileysEmotion,
        ["relief", "sad", "whew"]
    ),
    emoji!(
        "😓",
        "downcast face with sweat",
        SmileysEmotion,
        ["tired", "sweat", "sad"]
    ),
    emoji!(
        "🤯",
        "exploding head",
        SmileysEmotion,
        ["mindblown", "shock", "wow"]
    ),
    emoji!(
        "😴",
        "sleeping face",
        SmileysEmotion,
        ["sleep", "tired", "zzz"]
    ),
    emoji!(
        "🤤",
        "drooling face",
        SmileysEmotion,
        ["drool", "hungry", "desire"]
    ),
    emoji!(
        "😪",
        "sleepy face",
        SmileysEmotion,
        ["sleepy", "drowsy", "tired"]
    ),
    emoji!(
        "🤢",
        "nauseated face",
        SmileysEmotion,
        ["sick", "nausea", "gross"]
    ),
    emoji!(
        "🤮",
        "face vomiting",
        SmileysEmotion,
        ["vomit", "sick", "ill"]
    ),
    emoji!(
        "🤧",
        "sneezing face",
        SmileysEmotion,
        ["sneeze", "sick", "cold"]
    ),
    emoji!("🥵", "hot face", SmileysEmotion, ["hot", "sweat", "heat"]),
    emoji!(
        "🥶",
        "cold face",
        SmileysEmotion,
        ["cold", "freezing", "chill"]
    ),
    emoji!(
        "😵",
        "dizzy face",
        SmileysEmotion,
        ["dizzy", "confused", "woozy"]
    ),
    emoji!(
        "🥴",
        "woozy face",
        SmileysEmotion,
        ["woozy", "drunk", "dizzy"]
    ),
    emoji!(
        "😎",
        "smiling face with sunglasses",
        SmileysEmotion,
        ["cool", "sunglasses", "chill"]
    ),
    emoji!("🤓", "nerd face", SmileysEmotion, ["nerd", "smart", "geek"]),
    emoji!(
        "🧐",
        "face with monocle",
        SmileysEmotion,
        ["inspect", "fancy", "curious"]
    ),
    emoji!(
        "🤠",
        "cowboy hat face",
        SmileysEmotion,
        ["cowboy", "western", "hat"]
    ),
    emoji!(
        "🥳",
        "partying face",
        SmileysEmotion,
        ["party", "celebrate", "birthday"]
    ),
    emoji!(
        "😈",
        "smiling face with horns",
        SmileysEmotion,
        ["devil", "mischief", "horns"]
    ),
    emoji!(
        "👿",
        "angry face with horns",
        SmileysEmotion,
        ["devil", "angry", "horns"]
    ),
    emoji!("💀", "skull", SmileysEmotion, ["dead", "skull", "spooky"]),
    emoji!(
        "☠️",
        "skull and crossbones",
        SmileysEmotion,
        ["danger", "pirate", "poison"]
    ),
    emoji!(
        "💩",
        "pile of poo",
        SmileysEmotion,
        ["poo", "funny", "toilet"]
    ),
    emoji!(
        "🤡",
        "clown face",
        SmileysEmotion,
        ["clown", "circus", "silly"]
    ),
    emoji!(
        "👻",
        "ghost",
        SmileysEmotion,
        ["ghost", "spooky", "halloween"]
    ),
    emoji!("👽", "alien", SmileysEmotion, ["alien", "ufo", "space"]),
    emoji!("🤖", "robot", SmileysEmotion, ["robot", "ai", "bot"]),
    emoji!(
        "😺",
        "grinning cat",
        SmileysEmotion,
        ["cat", "smile", "pet"]
    ),
    emoji!(
        "😸",
        "grinning cat with smiling eyes",
        SmileysEmotion,
        ["cat", "joy", "pet"]
    ),
    emoji!(
        "😹",
        "cat with tears of joy",
        SmileysEmotion,
        ["cat", "laugh", "tears"]
    ),
    emoji!(
        "😻",
        "smiling cat with heart-eyes",
        SmileysEmotion,
        ["cat", "love", "heart"]
    ),
    emoji!(
        "😼",
        "cat with wry smile",
        SmileysEmotion,
        ["cat", "smirk", "pet"]
    ),
    emoji!("😽", "kissing cat", SmileysEmotion, ["cat", "kiss", "love"]),
    emoji!(
        "🙀",
        "weary cat",
        SmileysEmotion,
        ["cat", "shock", "scared"]
    ),
    emoji!("😿", "crying cat", SmileysEmotion, ["cat", "sad", "tears"]),
    emoji!("😾", "pouting cat", SmileysEmotion, ["cat", "angry", "mad"]),
    emoji!("❤️", "red heart", Symbols, ["heart", "love", "red"]),
    emoji!("🧡", "orange heart", Symbols, ["heart", "love", "orange"]),
    emoji!("💛", "yellow heart", Symbols, ["heart", "love", "yellow"]),
    emoji!("💚", "green heart", Symbols, ["heart", "love", "green"]),
    emoji!("💙", "blue heart", Symbols, ["heart", "love", "blue"]),
    emoji!("💜", "purple heart", Symbols, ["heart", "love", "purple"]),
    emoji!("🖤", "black heart", Symbols, ["heart", "love", "black"]),
    emoji!("🤍", "white heart", Symbols, ["heart", "love", "white"]),
    emoji!("🤎", "brown heart", Symbols, ["heart", "love", "brown"]),
    emoji!(
        "💔",
        "broken heart",
        Symbols,
        ["heartbreak", "sad", "breakup"]
    ),
    emoji!(
        "❣️",
        "heart exclamation",
        Symbols,
        ["heart", "emphasis", "love"]
    ),
    emoji!("💕", "two hearts", Symbols, ["hearts", "love", "affection"]),
    emoji!(
        "💞",
        "revolving hearts",
        Symbols,
        ["hearts", "romance", "love"]
    ),
    emoji!("💓", "beating heart", Symbols, ["heart", "pulse", "love"]),
    emoji!("💗", "growing heart", Symbols, ["heart", "love", "grow"]),
    emoji!(
        "💖",
        "sparkling heart",
        Symbols,
        ["heart", "sparkle", "love"]
    ),
    emoji!(
        "💘",
        "heart with arrow",
        Symbols,
        ["cupid", "heart", "romance"]
    ),
    emoji!(
        "💝",
        "heart with ribbon",
        Symbols,
        ["gift", "heart", "love"]
    ),
    emoji!("💯", "hundred points", Symbols, ["100", "perfect", "score"]),
    emoji!("💥", "collision", Symbols, ["boom", "impact", "explode"]),
    emoji!("💫", "dizzy", Symbols, ["dizzy", "star", "sparkle"]),
    emoji!("💦", "sweat droplets", Symbols, ["sweat", "water", "drops"]),
    emoji!("💨", "dashing away", Symbols, ["speed", "dash", "wind"]),
    emoji!(
        "👋",
        "waving hand",
        PeopleBody,
        ["wave", "hello", "goodbye"]
    ),
    emoji!(
        "🤚",
        "raised back of hand",
        PeopleBody,
        ["hand", "raised", "stop"]
    ),
    emoji!(
        "🖐️",
        "hand with fingers splayed",
        PeopleBody,
        ["hand", "five", "palm"]
    ),
    emoji!(
        "✋",
        "raised hand",
        PeopleBody,
        ["hand", "stop", "highfive"]
    ),
    emoji!(
        "🖖",
        "vulcan salute",
        PeopleBody,
        ["vulcan", "spock", "salute"]
    ),
    emoji!("👌", "ok hand", PeopleBody, ["ok", "hand", "perfect"]),
    emoji!(
        "🤌",
        "pinched fingers",
        PeopleBody,
        ["gesture", "pinched", "italian"]
    ),
    emoji!(
        "🤏",
        "pinching hand",
        PeopleBody,
        ["small", "pinch", "tiny"]
    ),
    emoji!(
        "✌️",
        "victory hand",
        PeopleBody,
        ["peace", "victory", "hand"]
    ),
    emoji!(
        "🤞",
        "crossed fingers",
        PeopleBody,
        ["luck", "hope", "fingers"]
    ),
    emoji!(
        "🫰",
        "hand with index finger and thumb crossed",
        PeopleBody,
        ["heart", "finger", "gesture"]
    ),
    emoji!(
        "🤟",
        "love-you gesture",
        PeopleBody,
        ["ily", "hand", "love"]
    ),
    emoji!(
        "🤘",
        "sign of the horns",
        PeopleBody,
        ["rock", "metal", "hand"]
    ),
    emoji!("🤙", "call me hand", PeopleBody, ["call", "phone", "shaka"]),
    emoji!(
        "👈",
        "backhand index pointing left",
        PeopleBody,
        ["left", "point", "hand"]
    ),
    emoji!(
        "👉",
        "backhand index pointing right",
        PeopleBody,
        ["right", "point", "hand"]
    ),
    emoji!(
        "👆",
        "backhand index pointing up",
        PeopleBody,
        ["up", "point", "hand"]
    ),
    emoji!(
        "👇",
        "backhand index pointing down",
        PeopleBody,
        ["down", "point", "hand"]
    ),
    emoji!(
        "☝️",
        "index pointing up",
        PeopleBody,
        ["up", "index", "point"]
    ),
    emoji!("👍", "thumbs up", PeopleBody, ["approve", "like", "yes"]),
    emoji!("👎", "thumbs down", PeopleBody, ["dislike", "no", "reject"]),
    emoji!("👊", "oncoming fist", PeopleBody, ["fist", "punch", "bump"]),
    emoji!(
        "✊",
        "raised fist",
        PeopleBody,
        ["fist", "power", "solidarity"]
    ),
    emoji!(
        "🤛",
        "left-facing fist",
        PeopleBody,
        ["fist", "left", "bump"]
    ),
    emoji!(
        "🤜",
        "right-facing fist",
        PeopleBody,
        ["fist", "right", "bump"]
    ),
    emoji!(
        "👏",
        "clapping hands",
        PeopleBody,
        ["clap", "applause", "praise"]
    ),
    emoji!(
        "🙌",
        "raising hands",
        PeopleBody,
        ["hooray", "celebrate", "hands"]
    ),
    emoji!("👐", "open hands", PeopleBody, ["open", "hug", "hands"]),
    emoji!(
        "🤲",
        "palms up together",
        PeopleBody,
        ["offer", "prayer", "hands"]
    ),
    emoji!(
        "🤝",
        "handshake",
        PeopleBody,
        ["deal", "agreement", "greet"]
    ),
    emoji!(
        "🙏",
        "folded hands",
        PeopleBody,
        ["please", "thanks", "pray"]
    ),
    emoji!(
        "💪",
        "flexed biceps",
        PeopleBody,
        ["strong", "muscle", "gym"]
    ),
    emoji!(
        "🦾",
        "mechanical arm",
        PeopleBody,
        ["prosthetic", "robot", "strength"]
    ),
    emoji!("🧠", "brain", PeopleBody, ["brain", "smart", "mind"]),
    emoji!("👀", "eyes", PeopleBody, ["look", "watch", "see"]),
    emoji!("👁️", "eye", PeopleBody, ["eye", "vision", "watch"]),
    emoji!("👄", "mouth", PeopleBody, ["lips", "mouth", "speak"]),
    emoji!("👅", "tongue", PeopleBody, ["tongue", "taste", "playful"]),
    emoji!("👂", "ear", PeopleBody, ["listen", "ear", "hear"]),
    emoji!("👃", "nose", PeopleBody, ["smell", "nose", "face"]),
    emoji!("🫶", "heart hands", PeopleBody, ["heart", "hands", "love"]),
    emoji!("🐶", "dog face", AnimalsNature, ["dog", "pet", "puppy"]),
    emoji!("🐱", "cat face", AnimalsNature, ["cat", "pet", "kitty"]),
    emoji!(
        "🐭",
        "mouse face",
        AnimalsNature,
        ["mouse", "animal", "small"]
    ),
    emoji!(
        "🐹",
        "hamster face",
        AnimalsNature,
        ["hamster", "pet", "cute"]
    ),
    emoji!(
        "🐰",
        "rabbit face",
        AnimalsNature,
        ["rabbit", "bunny", "cute"]
    ),
    emoji!("🦊", "fox", AnimalsNature, ["fox", "animal", "wild"]),
    emoji!("🐻", "bear", AnimalsNature, ["bear", "animal", "wild"]),
    emoji!("🐼", "panda", AnimalsNature, ["panda", "bear", "cute"]),
    emoji!(
        "🐨",
        "koala",
        AnimalsNature,
        ["koala", "animal", "australia"]
    ),
    emoji!("🐯", "tiger face", AnimalsNature, ["tiger", "cat", "wild"]),
    emoji!("🦁", "lion", AnimalsNature, ["lion", "cat", "wild"]),
    emoji!("🐮", "cow face", AnimalsNature, ["cow", "farm", "animal"]),
    emoji!("🐷", "pig face", AnimalsNature, ["pig", "farm", "animal"]),
    emoji!("🐸", "frog", AnimalsNature, ["frog", "animal", "green"]),
    emoji!(
        "🐵",
        "monkey face",
        AnimalsNature,
        ["monkey", "animal", "primate"]
    ),
    emoji!(
        "🦋",
        "butterfly",
        AnimalsNature,
        ["butterfly", "insect", "nature"]
    ),
    emoji!(
        "🌸",
        "cherry blossom",
        AnimalsNature,
        ["flower", "spring", "pink"]
    ),
    emoji!(
        "🌻",
        "sunflower",
        AnimalsNature,
        ["flower", "sun", "nature"]
    ),
    emoji!("🌈", "rainbow", AnimalsNature, ["rainbow", "color", "sky"]),
    emoji!(
        "🌙",
        "crescent moon",
        AnimalsNature,
        ["moon", "night", "sky"]
    ),
    emoji!("☀️", "sun", AnimalsNature, ["sun", "weather", "bright"]),
    emoji!("🔥", "fire", AnimalsNature, ["fire", "lit", "hot"]),
    emoji!("🍎", "red apple", FoodDrink, ["apple", "fruit", "food"]),
    emoji!("🍕", "pizza", FoodDrink, ["pizza", "food", "slice"]),
    emoji!("🍔", "hamburger", FoodDrink, ["burger", "food", "meal"]),
    emoji!("🍟", "french fries", FoodDrink, ["fries", "food", "snack"]),
    emoji!("🌮", "taco", FoodDrink, ["taco", "food", "mexican"]),
    emoji!("🍣", "sushi", FoodDrink, ["sushi", "food", "japanese"]),
    emoji!(
        "🍜",
        "steaming bowl",
        FoodDrink,
        ["ramen", "noodles", "soup"]
    ),
    emoji!("🍩", "doughnut", FoodDrink, ["donut", "sweet", "dessert"]),
    emoji!("🍪", "cookie", FoodDrink, ["cookie", "sweet", "dessert"]),
    emoji!("☕", "hot beverage", FoodDrink, ["coffee", "tea", "drink"]),
    emoji!("🍺", "beer mug", FoodDrink, ["beer", "drink", "bar"]),
    emoji!("🍷", "wine glass", FoodDrink, ["wine", "drink", "glass"]),
    emoji!("🥤", "cup with straw", FoodDrink, ["drink", "soda", "cold"]),
    emoji!("🧋", "bubble tea", FoodDrink, ["boba", "tea", "drink"]),
    emoji!("🍿", "popcorn", FoodDrink, ["snack", "movie", "popcorn"]),
    emoji!("📱", "mobile phone", Objects, ["phone", "mobile", "device"]),
    emoji!("💻", "laptop", Objects, ["computer", "laptop", "work"]),
    emoji!("⌚", "watch", Objects, ["watch", "time", "wearable"]),
    emoji!("📷", "camera", Objects, ["camera", "photo", "picture"]),
    emoji!("🎧", "headphone", Objects, ["headphones", "music", "audio"]),
    emoji!("🔋", "battery", Objects, ["battery", "power", "charge"]),
    emoji!(
        "🔌",
        "electric plug",
        Objects,
        ["plug", "power", "electric"]
    ),
    emoji!("💡", "light bulb", Objects, ["idea", "light", "bulb"]),
    emoji!(
        "🧯",
        "fire extinguisher",
        Objects,
        ["safety", "fire", "tool"]
    ),
    emoji!(
        "🛒",
        "shopping cart",
        Objects,
        ["shopping", "cart", "store"]
    ),
    emoji!(
        "🥹",
        "face holding back tears",
        SmileysEmotion,
        ["tears", "emotional", "moved"]
    ),
    emoji!(
        "🫠",
        "melting face",
        SmileysEmotion,
        ["melt", "awkward", "heat"]
    ),
    emoji!(
        "🫥",
        "dotted line face",
        SmileysEmotion,
        ["invisible", "faded", "awkward"]
    ),
    emoji!(
        "🫨",
        "shaking face",
        SmileysEmotion,
        ["shaking", "shocked", "vibrating"]
    ),
    emoji!(
        "🤥",
        "lying face",
        SmileysEmotion,
        ["lie", "pinocchio", "dishonest"]
    ),
    emoji!(
        "😮‍💨",
        "face exhaling",
        SmileysEmotion,
        ["exhale", "relief", "sigh"]
    ),
    emoji!(
        "😶‍🌫️",
        "face in clouds",
        SmileysEmotion,
        ["foggy", "confused", "dazed"]
    ),
    emoji!(
        "😵‍💫",
        "face with spiral eyes",
        SmileysEmotion,
        ["spiral", "dizzy", "hypnotized"]
    ),
    emoji!(
        "🫵",
        "index pointing at the viewer",
        PeopleBody,
        ["you", "point", "finger"]
    ),
    emoji!(
        "🫱",
        "rightwards hand",
        PeopleBody,
        ["hand", "right", "reach"]
    ),
    emoji!(
        "🫲",
        "leftwards hand",
        PeopleBody,
        ["hand", "left", "reach"]
    ),
    emoji!("🦶", "foot", PeopleBody, ["foot", "body", "kick"]),
    emoji!("🦵", "leg", PeopleBody, ["leg", "body", "step"]),
    emoji!(
        "🦻",
        "ear with hearing aid",
        PeopleBody,
        ["ear", "hearing", "accessibility"]
    ),
    emoji!("🫦", "biting lip", PeopleBody, ["lip", "nervous", "flirty"]),
    emoji!(
        "🫀",
        "anatomical heart",
        PeopleBody,
        ["heart", "organ", "anatomy"]
    ),
    emoji!("🐺", "wolf", AnimalsNature, ["wolf", "wild", "canine"]),
    emoji!("🐗", "boar", AnimalsNature, ["boar", "wild", "pig"]),
    emoji!(
        "🐴",
        "horse face",
        AnimalsNature,
        ["horse", "animal", "farm"]
    ),
    emoji!("🦄", "unicorn", AnimalsNature, ["unicorn", "magic", "myth"]),
    emoji!("🐔", "chicken", AnimalsNature, ["chicken", "bird", "farm"]),
    emoji!("🐧", "penguin", AnimalsNature, ["penguin", "bird", "cold"]),
    emoji!("🐦", "bird", AnimalsNature, ["bird", "animal", "tweet"]),
    emoji!("🐢", "turtle", AnimalsNature, ["turtle", "animal", "slow"]),
    emoji!(
        "🐬",
        "dolphin",
        AnimalsNature,
        ["dolphin", "ocean", "smart"]
    ),
    emoji!("🍌", "banana", FoodDrink, ["banana", "fruit", "food"]),
    emoji!("🍇", "grapes", FoodDrink, ["grapes", "fruit", "food"]),
    emoji!(
        "🍓",
        "strawberry",
        FoodDrink,
        ["strawberry", "fruit", "sweet"]
    ),
    emoji!("🥑", "avocado", FoodDrink, ["avocado", "fruit", "food"]),
    emoji!("🥓", "bacon", FoodDrink, ["bacon", "meat", "breakfast"]),
    emoji!("🍗", "poultry leg", FoodDrink, ["chicken", "meat", "food"]),
    emoji!("🍞", "bread", FoodDrink, ["bread", "food", "baked"]),
    emoji!("🧀", "cheese wedge", FoodDrink, ["cheese", "dairy", "food"]),
    emoji!("🍰", "shortcake", FoodDrink, ["cake", "dessert", "sweet"]),
    emoji!("🥗", "green salad", FoodDrink, ["salad", "healthy", "food"]),
    emoji!(
        "🚗",
        "automobile",
        TravelPlaces,
        ["car", "vehicle", "drive"]
    ),
    emoji!("🚕", "taxi", TravelPlaces, ["taxi", "cab", "vehicle"]),
    emoji!(
        "🚙",
        "sport utility vehicle",
        TravelPlaces,
        ["suv", "car", "vehicle"]
    ),
    emoji!("🚌", "bus", TravelPlaces, ["bus", "transit", "vehicle"]),
    emoji!(
        "🚎",
        "trolleybus",
        TravelPlaces,
        ["trolley", "bus", "transit"]
    ),
    emoji!(
        "🚓",
        "police car",
        TravelPlaces,
        ["police", "car", "emergency"]
    ),
    emoji!(
        "🚑",
        "ambulance",
        TravelPlaces,
        ["ambulance", "medical", "emergency"]
    ),
    emoji!(
        "🚒",
        "fire engine",
        TravelPlaces,
        ["fire", "truck", "emergency"]
    ),
    emoji!(
        "🚚",
        "delivery truck",
        TravelPlaces,
        ["truck", "delivery", "shipping"]
    ),
    emoji!("🚲", "bicycle", TravelPlaces, ["bike", "bicycle", "ride"]),
    emoji!(
        "✈️",
        "airplane",
        TravelPlaces,
        ["plane", "travel", "flight"]
    ),
    emoji!("🚀", "rocket", TravelPlaces, ["rocket", "space", "launch"]),
    emoji!(
        "🚂",
        "locomotive",
        TravelPlaces,
        ["train", "locomotive", "rail"]
    ),
    emoji!(
        "🚉",
        "railway station",
        TravelPlaces,
        ["station", "train", "travel"]
    ),
    emoji!("🏠", "house", TravelPlaces, ["house", "home", "building"]),
    emoji!("🏨", "hotel", TravelPlaces, ["hotel", "building", "travel"]),
    emoji!(
        "🗽",
        "statue of liberty",
        TravelPlaces,
        ["landmark", "nyc", "statue"]
    ),
    emoji!("⛵", "sailboat", TravelPlaces, ["boat", "sail", "water"]),
    emoji!(
        "⚽",
        "soccer ball",
        Activities,
        ["soccer", "football", "sport"]
    ),
    emoji!(
        "🏀",
        "basketball",
        Activities,
        ["basketball", "sport", "ball"]
    ),
    emoji!(
        "🏈",
        "american football",
        Activities,
        ["football", "sport", "nfl"]
    ),
    emoji!("⚾", "baseball", Activities, ["baseball", "sport", "ball"]),
    emoji!("🎾", "tennis", Activities, ["tennis", "sport", "racket"]),
    emoji!(
        "🏐",
        "volleyball",
        Activities,
        ["volleyball", "sport", "ball"]
    ),
    emoji!(
        "🏓",
        "ping pong",
        Activities,
        ["pingpong", "table tennis", "sport"]
    ),
    emoji!(
        "🏸",
        "badminton",
        Activities,
        ["badminton", "sport", "racket"]
    ),
    emoji!(
        "🥊",
        "boxing glove",
        Activities,
        ["boxing", "fight", "sport"]
    ),
    emoji!(
        "🎮",
        "video game",
        Activities,
        ["gaming", "controller", "play"]
    ),
    emoji!("🎯", "direct hit", Activities, ["target", "dart", "game"]),
    emoji!("🎲", "game die", Activities, ["dice", "game", "luck"]),
    emoji!(
        "🖥️",
        "desktop computer",
        Objects,
        ["desktop", "computer", "monitor"]
    ),
    emoji!("🖨️", "printer", Objects, ["printer", "print", "office"]),
    emoji!(
        "🕹️",
        "joystick",
        Objects,
        ["joystick", "game", "controller"]
    ),
    emoji!("💽", "computer disk", Objects, ["disk", "storage", "data"]),
    emoji!("📺", "television", Objects, ["tv", "screen", "video"]),
    emoji!("📚", "books", Objects, ["books", "study", "read"]),
    emoji!("✏️", "pencil", Objects, ["pencil", "write", "school"]),
    emoji!("🧰", "toolbox", Objects, ["toolbox", "tools", "repair"]),
    emoji!("🧲", "magnet", Objects, ["magnet", "science", "metal"]),
    emoji!("🧪", "test tube", Objects, ["test", "science", "lab"]),
    emoji!("✨", "sparkles", Symbols, ["sparkle", "shine", "magic"]),
    emoji!("⭐", "star", Symbols, ["star", "favorite", "rating"]),
    emoji!("🌟", "glowing star", Symbols, ["star", "glow", "sparkle"]),
    emoji!("🔔", "bell", Symbols, ["bell", "notification", "alert"]),
    emoji!("🎵", "musical note", Symbols, ["music", "note", "song"]),
    emoji!("✅", "check mark button", Symbols, ["check", "done", "yes"]),
    emoji!("❌", "cross mark", Symbols, ["cross", "no", "cancel"]),
    emoji!("⚠️", "warning", Symbols, ["warning", "alert", "caution"]),
    emoji!(
        "🚫",
        "prohibited",
        Symbols,
        ["prohibited", "no", "forbidden"]
    ),
    emoji!(
        "♻️",
        "recycling symbol",
        Symbols,
        ["recycle", "green", "eco"]
    ),
    emoji!("🆗", "OK button", Symbols, ["ok", "button", "agree"]),
    emoji!(
        "🇺🇸",
        "flag: United States",
        Flags,
        ["flag", "usa", "america"]
    ),
    emoji!("🇨🇦", "flag: Canada", Flags, ["flag", "canada", "country"]),
    emoji!(
        "🇬🇧",
        "flag: United Kingdom",
        Flags,
        ["flag", "uk", "britain"]
    ),
    emoji!("🇫🇷", "flag: France", Flags, ["flag", "france", "country"]),
    emoji!("🇩🇪", "flag: Germany", Flags, ["flag", "germany", "country"]),
    emoji!("🇯🇵", "flag: Japan", Flags, ["flag", "japan", "country"]),
    emoji!(
        "🇰🇷",
        "flag: South Korea",
        Flags,
        ["flag", "korea", "country"]
    ),
    emoji!("🇮🇳", "flag: India", Flags, ["flag", "india", "country"]),
    emoji!("🇧🇷", "flag: Brazil", Flags, ["flag", "brazil", "country"]),
    emoji!(
        "🇦🇺",
        "flag: Australia",
        Flags,
        ["flag", "australia", "country"]
    ),
];

pub fn emojis_by_category(category: EmojiCategory) -> Vec<&'static Emoji> {
    EMOJIS
        .iter()
        .filter(|emoji| emoji.category == category)
        .collect()
}

pub fn grouped_emojis() -> Vec<(EmojiCategory, Vec<&'static Emoji>)> {
    ALL_CATEGORIES
        .iter()
        .copied()
        .map(|category| (category, emojis_by_category(category)))
        .collect()
}

/// Number of columns in the emoji picker grid.
/// Shared between the renderer and the arrow-key navigation interceptor.
pub const GRID_COLS: usize = 8;

/// Build the filtered, category-ordered emoji list used by both the renderer
/// and the arrow-key navigation handler. This ensures selection indices stay
/// in sync between rendering and navigation.
pub fn filtered_ordered_emojis(
    filter: &str,
    selected_category: Option<EmojiCategory>,
) -> Vec<Emoji> {
    let mut filtered: Vec<Emoji> = search_emojis(filter).into_iter().copied().collect();
    if let Some(cat) = selected_category {
        filtered.retain(|e| e.category == cat);
    }
    let mut ordered: Vec<Emoji> = Vec::with_capacity(filtered.len());
    for cat in ALL_CATEGORIES.iter().copied() {
        ordered.extend(filtered.iter().copied().filter(|e| e.category == cat));
    }
    ordered
}

/// Compute the uniform-list row index that contains `selected_index`.
///
/// Row layout per category: 1 header row + ceil(count / GRID_COLS) cell rows.
/// Returns the row suitable for `scroll_handle.scroll_to_item(row, Nearest)`.
pub fn compute_scroll_row(selected_index: usize, ordered: &[Emoji]) -> usize {
    let cols = GRID_COLS;
    let mut flat_offset: usize = 0;
    let mut row_offset: usize = 0;
    for cat in ALL_CATEGORIES.iter().copied() {
        let cat_count = ordered.iter().filter(|e| e.category == cat).count();
        if cat_count == 0 {
            continue;
        }
        if flat_offset + cat_count > selected_index {
            let idx_in_cat = selected_index - flat_offset;
            let cell_row = idx_in_cat / cols;
            row_offset += 1 + cell_row;
            return row_offset;
        }
        row_offset += 1 + cat_count.div_ceil(cols);
        flat_offset += cat_count;
    }
    row_offset
}

pub fn search_emojis(query: &str) -> Vec<&Emoji> {
    let query = query.trim().to_ascii_lowercase();
    if query.is_empty() {
        return EMOJIS.iter().collect();
    }

    EMOJIS
        .iter()
        .filter(|emoji| {
            emoji.name.to_ascii_lowercase().contains(&query)
                || emoji
                    .keywords
                    .iter()
                    .any(|keyword| keyword.to_ascii_lowercase().contains(&query))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_emoji_database_has_296_entries() {
        assert_eq!(EMOJIS.len(), 296);
    }

    #[test]
    fn test_emoji_category_display_name_returns_human_readable_labels() {
        assert_eq!(SmileysEmotion.display_name(), "Smileys & Emotion");
        assert_eq!(PeopleBody.display_name(), "People & Body");
        assert_eq!(AnimalsNature.display_name(), "Animals & Nature");
        assert_eq!(FoodDrink.display_name(), "Food & Drink");
        assert_eq!(TravelPlaces.display_name(), "Travel & Places");
        assert_eq!(Activities.display_name(), "Activities");
        assert_eq!(Objects.display_name(), "Objects");
        assert_eq!(Symbols.display_name(), "Symbols");
        assert_eq!(Flags.display_name(), "Flags");
    }

    #[test]
    fn test_all_categories_has_expected_display_order() {
        assert_eq!(
            ALL_CATEGORIES,
            &[
                SmileysEmotion,
                PeopleBody,
                AnimalsNature,
                FoodDrink,
                TravelPlaces,
                Activities,
                Objects,
                Symbols,
                Flags
            ]
        );
    }

    #[test]
    fn test_emojis_by_category_returns_only_requested_category() {
        let travel_emojis = emojis_by_category(TravelPlaces);
        assert!(!travel_emojis.is_empty());
        assert!(travel_emojis
            .iter()
            .all(|emoji| emoji.category == TravelPlaces));
    }

    #[test]
    fn test_grouped_emojis_returns_all_categories_in_display_order() {
        let grouped = grouped_emojis();
        assert_eq!(grouped.len(), ALL_CATEGORIES.len());

        for ((category, emojis), expected_category) in
            grouped.iter().zip(ALL_CATEGORIES.iter().copied())
        {
            assert_eq!(*category, expected_category);
            assert!(emojis
                .iter()
                .all(|emoji| emoji.category == expected_category));
        }
    }

    #[test]
    fn test_grouped_emojis_covers_all_entries() {
        let grouped = grouped_emojis();
        let total_grouped_emojis: usize = grouped.iter().map(|(_, emojis)| emojis.len()).sum();
        assert_eq!(total_grouped_emojis, EMOJIS.len());
    }

    #[test]
    fn test_emoji_database_meets_category_targets() {
        assert!(emojis_by_category(SmileysEmotion).len() >= 50);
        assert!(emojis_by_category(PeopleBody).len() >= 30);
        assert!(emojis_by_category(AnimalsNature).len() >= 20);
        assert!(emojis_by_category(FoodDrink).len() >= 15);
        assert!(emojis_by_category(TravelPlaces).len() >= 15);
        assert!(emojis_by_category(Activities).len() >= 10);
        assert!(emojis_by_category(Objects).len() >= 15);
        assert!(emojis_by_category(Symbols).len() >= 15);
        assert!(emojis_by_category(Flags).len() >= 10);
    }

    #[test]
    fn test_search_emojis_matches_name_when_query_has_different_case() {
        let matches = search_emojis("GRINNING");
        assert!(matches.iter().any(|emoji| emoji.emoji == "😀"));
    }

    #[test]
    fn test_search_emojis_matches_keyword_when_query_is_substring() {
        let matches = search_emojis("appro");
        assert!(matches.iter().any(|emoji| emoji.emoji == "👍"));
    }

    #[test]
    fn test_search_emojis_returns_all_when_query_is_empty() {
        let matches = search_emojis("   ");
        assert_eq!(matches.len(), EMOJIS.len());
    }
}
