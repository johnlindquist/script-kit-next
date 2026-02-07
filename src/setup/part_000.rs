use std::fs;
use std::path::{Path, PathBuf};
use tracing::{debug, info, instrument, warn};
/// Embedded config template (included at compile time)
const EMBEDDED_CONFIG_TEMPLATE: &str = include_str!("../../kit-init/config-template.ts");
/// Embedded SDK content (included at compile time)
const EMBEDDED_SDK: &str = include_str!("../../scripts/kit-sdk.ts");
/// Optional theme example (included at compile time)
const EMBEDDED_THEME_EXAMPLE: &str = include_str!("../../kit-init/theme.example.json");
/// Embedded package.json template for user's kit directory
/// The "type": "module" enables top-level await in all .ts scripts
const EMBEDDED_PACKAGE_JSON: &str = r#"{
  "name": "@scriptkit/kit",
  "type": "module",
  "private": true,
  "scripts": {
    "typecheck": "tsc --noEmit"
  }
}
"#;
/// Embedded GUIDE.md comprehensive user guide
const EMBEDDED_GUIDE_MD: &str = include_str!("../../kit-init/GUIDE.md");
/// Embedded CleanShot X extension (built-in extension that ships with the app)
const EMBEDDED_CLEANSHOT_EXTENSION: &str = include_str!("../../kit-init/extensions/cleanshot/main.md");
/// Embedded CleanShot X shared actions (built-in actions for all cleanshot scriptlets)
const EMBEDDED_CLEANSHOT_ACTIONS: &str =
    include_str!("../../kit-init/extensions/cleanshot/main.actions.md");
/// Embedded 1Password extension (built-in extension that ships with the app)
const EMBEDDED_1PASSWORD_EXTENSION: &str = include_str!("../../kit-init/extensions/1password/main.md");
/// Embedded Quick Links extension (built-in extension that ships with the app)
const EMBEDDED_QUICKLINKS_EXTENSION: &str =
    include_str!("../../kit-init/extensions/quicklinks/main.md");
/// Embedded Quick Links shared actions (built-in actions for all quicklinks scriptlets)
const EMBEDDED_QUICKLINKS_ACTIONS: &str =
    include_str!("../../kit-init/extensions/quicklinks/main.actions.md");
/// Embedded Window Management extension (built-in extension that ships with the app)
const EMBEDDED_WINDOW_MANAGEMENT_EXTENSION: &str =
    include_str!("../../kit-init/extensions/window-management/main.md");
/// Embedded AI Text Tools extension (built-in extension that ships with the app)
const EMBEDDED_AI_TEXT_TOOLS_EXTENSION: &str =
    include_str!("../../kit-init/extensions/ai-text-tools/main.md");
/// Embedded Examples extension - main scriptlet examples (built-in extension that ships with the app)
const EMBEDDED_EXAMPLES_MAIN: &str = include_str!("../../kit-init/extensions/examples/main.md");
/// Embedded Examples extension - advanced scriptlet examples (built-in extension that ships with the app)
const EMBEDDED_EXAMPLES_ADVANCED: &str =
    include_str!("../../kit-init/extensions/examples/advanced.md");
/// Embedded Examples extension - howto guide (built-in extension that ships with the app)
const EMBEDDED_EXAMPLES_HOWTO: &str = include_str!("../../kit-init/extensions/examples/howto.md");
