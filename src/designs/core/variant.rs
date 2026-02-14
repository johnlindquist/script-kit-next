/// Design variant enumeration
///
/// Each variant represents a distinct visual style for the script list.
/// Use `Cmd+1` through `Cmd+0` to switch between designs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[repr(u8)]
pub enum DesignVariant {
    /// Default design (uses existing implementation)
    /// Keyboard: Cmd+1
    #[default]
    Default = 1,

    /// Minimal design with reduced visual elements
    /// Keyboard: Cmd+2
    Minimal = 2,

    /// Retro terminal aesthetic with monospace fonts and green-on-black
    /// Keyboard: Cmd+3
    RetroTerminal = 3,

    /// Glassmorphism with frosted glass effects and transparency
    /// Keyboard: Cmd+4
    Glassmorphism = 4,

    /// Brutalist design with raw, bold typography
    /// Keyboard: Cmd+5
    Brutalist = 5,

    /// Neon cyberpunk with glowing accents and dark backgrounds
    /// Keyboard: Cmd+6
    NeonCyberpunk = 6,

    /// Paper-like design with warm tones and subtle shadows
    /// Keyboard: Cmd+7
    Paper = 7,

    /// Apple Human Interface Guidelines inspired design
    /// Keyboard: Cmd+8
    AppleHIG = 8,

    /// Material Design 3 (Material You) inspired design
    /// Keyboard: Cmd+9
    Material3 = 9,

    /// Compact design with smaller items for power users
    /// Keyboard: Cmd+0
    Compact = 10,

    /// Playful design with rounded corners and vibrant colors
    /// Not directly accessible via keyboard shortcut
    Playful = 11,
}

impl DesignVariant {
    /// Get all available design variants
    pub fn all() -> &'static [DesignVariant] {
        &[
            DesignVariant::Default,
            DesignVariant::Minimal,
            DesignVariant::RetroTerminal,
            DesignVariant::Glassmorphism,
            DesignVariant::Brutalist,
            DesignVariant::NeonCyberpunk,
            DesignVariant::Paper,
            DesignVariant::AppleHIG,
            DesignVariant::Material3,
            DesignVariant::Compact,
            DesignVariant::Playful,
        ]
    }

    /// Get the next design variant in the cycle
    ///
    /// Cycles through all designs: Default -> Minimal -> RetroTerminal -> ... -> Playful -> Default
    pub fn next(self) -> DesignVariant {
        let all = Self::all();
        let current_idx = all.iter().position(|&v| v == self).unwrap_or(0);
        let next_idx = (current_idx + 1) % all.len();
        all[next_idx]
    }

    /// Get the previous design variant in the cycle
    #[allow(dead_code)]
    pub fn prev(self) -> DesignVariant {
        let all = Self::all();
        let current_idx = all.iter().position(|&v| v == self).unwrap_or(0);
        let prev_idx = if current_idx == 0 {
            all.len() - 1
        } else {
            current_idx - 1
        };
        all[prev_idx]
    }

    /// Get the display name for this variant
    pub fn name(&self) -> &'static str {
        match self {
            DesignVariant::Default => "Default",
            DesignVariant::Minimal => "Minimal",
            DesignVariant::RetroTerminal => "Retro Terminal",
            DesignVariant::Glassmorphism => "Glassmorphism",
            DesignVariant::Brutalist => "Brutalist",
            DesignVariant::NeonCyberpunk => "Neon Cyberpunk",
            DesignVariant::Paper => "Paper",
            DesignVariant::AppleHIG => "Apple HIG",
            DesignVariant::Material3 => "Material 3",
            DesignVariant::Compact => "Compact",
            DesignVariant::Playful => "Playful",
        }
    }

    /// Get the keyboard shortcut number for this variant (1-10, where 0 = 10)
    #[allow(dead_code)]
    pub fn shortcut_number(&self) -> Option<u8> {
        match self {
            DesignVariant::Default => Some(1),
            DesignVariant::Minimal => Some(2),
            DesignVariant::RetroTerminal => Some(3),
            DesignVariant::Glassmorphism => Some(4),
            DesignVariant::Brutalist => Some(5),
            DesignVariant::NeonCyberpunk => Some(6),
            DesignVariant::Paper => Some(7),
            DesignVariant::AppleHIG => Some(8),
            DesignVariant::Material3 => Some(9),
            DesignVariant::Compact => Some(0), // Cmd+0 maps to 10
            DesignVariant::Playful => None,    // No direct shortcut
        }
    }

    /// Create a variant from a keyboard number (1-9, 0 for 10)
    #[allow(dead_code)]
    pub fn from_keyboard_number(num: u8) -> Option<DesignVariant> {
        match num {
            1 => Some(DesignVariant::Default),
            2 => Some(DesignVariant::Minimal),
            3 => Some(DesignVariant::RetroTerminal),
            4 => Some(DesignVariant::Glassmorphism),
            5 => Some(DesignVariant::Brutalist),
            6 => Some(DesignVariant::NeonCyberpunk),
            7 => Some(DesignVariant::Paper),
            8 => Some(DesignVariant::AppleHIG),
            9 => Some(DesignVariant::Material3),
            0 => Some(DesignVariant::Compact),
            _ => None,
        }
    }

    /// Get a short description of this design variant
    pub fn description(&self) -> &'static str {
        match self {
            DesignVariant::Default => "The standard Script Kit appearance",
            DesignVariant::Minimal => "Clean and minimal with reduced visual noise",
            DesignVariant::RetroTerminal => "Classic terminal aesthetics with green phosphor glow",
            DesignVariant::Glassmorphism => "Frosted glass effects with soft transparency",
            DesignVariant::Brutalist => "Bold, raw typography with strong contrast",
            DesignVariant::NeonCyberpunk => "Dark backgrounds with vibrant neon accents",
            DesignVariant::Paper => "Warm, paper-like tones with subtle textures",
            DesignVariant::AppleHIG => "Following Apple Human Interface Guidelines",
            DesignVariant::Material3 => "Google Material Design 3 (Material You) inspired",
            DesignVariant::Compact => "Dense layout for power users with many scripts",
            DesignVariant::Playful => "Fun, rounded design with vibrant colors",
        }
    }
}
