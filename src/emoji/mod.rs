#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Emoji {
    pub emoji: &'static str,
    pub name: &'static str,
    pub keywords: &'static [&'static str],
    pub category: EmojiCategory,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
];

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
    fn test_emoji_database_has_200_entries() {
        assert_eq!(EMOJIS.len(), 200);
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
