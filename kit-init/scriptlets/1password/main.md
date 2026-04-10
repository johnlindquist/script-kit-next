---
name: 1Password
description: Password manager CLI commands and quick access
author: Script Kit
icon: lock
---

# 1Password

Access your passwords, generate secure credentials, and manage vaults using the 1Password CLI.

> **Requires**: [1Password CLI](https://developer.1password.com/docs/cli/get-started/) (`op`) installed and configured.

---

## Open Quick Access

<!--
description: Open 1Password Quick Access overlay
-->

```applescript
tell application "System Events"
    keystroke space using {command down, shift down}
end tell
```

---

## Open 1Password

<!--
description: Launch the 1Password application
-->

```open
file:///Applications/1Password.app
```

---

## Search Items

<!--
description: Search for items by name or keyword
-->

```bash
op item list --format=json | jq -r '.[] | "\(.title) (\(.vault.name))"' | head -20
```

---

## List All Items

<!--
description: List all items across all vaults
-->

```bash
op item list --format=json | jq -r '.[] | "\(.title) | \(.category) | \(.vault.name)"'
```

---

## List Logins

<!--
description: List all login items
-->

```bash
op item list --categories Login --format=json | jq -r '.[] | "\(.title) (\(.vault.name))"'
```

---

## List Secure Notes

<!--
description: List all secure note items
-->

```bash
op item list --categories "Secure Note" --format=json | jq -r '.[] | "\(.title) (\(.vault.name))"'
```

---

## List Credit Cards

<!--
description: List all credit card items
-->

```bash
op item list --categories "Credit Card" --format=json | jq -r '.[] | "\(.title) (\(.vault.name))"'
```

---

## List Passwords

<!--
description: List all password items
-->

```bash
op item list --categories Password --format=json | jq -r '.[] | "\(.title) (\(.vault.name))"'
```

---

## List API Credentials

<!--
description: List all API credential items
-->

```bash
op item list --categories "API Credential" --format=json | jq -r '.[] | "\(.title) (\(.vault.name))"'
```

---

## List SSH Keys

<!--
description: List all SSH key items
-->

```bash
op item list --categories "SSH Key" --format=json | jq -r '.[] | "\(.title) (\(.vault.name))"'
```

---

## List Items by Tag

<!--
description: List items with a specific tag
-->

```bash
read -p "Enter tag: " tag && op item list --tags "$tag" --format=json | jq -r '.[] | "\(.title) (\(.vault.name))"'
```

---

## Get Item Details

<!--
description: Get full details of a specific item
-->

```bash
read -p "Item name: " name && op item get "$name" --format=json | jq '.'
```

---

## Copy Password

<!--
description: Copy password for an item to clipboard
-->

```bash
read -p "Item name: " name && op item get "$name" --fields password --reveal | pbcopy && echo "Password copied to clipboard"
```

---

## Copy Username

<!--
description: Copy username for an item to clipboard
-->

```bash
read -p "Item name: " name && op item get "$name" --fields username | pbcopy && echo "Username copied to clipboard"
```

---

## Copy One-Time Password

<!--
description: Copy OTP/2FA code for an item to clipboard
-->

```bash
read -p "Item name: " name && op item get "$name" --otp | pbcopy && echo "OTP copied to clipboard"
```

---

## Get Share Link

<!--
description: Get a shareable link for an item
-->

```bash
read -p "Item name: " name && op item get "$name" --share-link
```

---

## Generate Password

<!--
description: Generate a random secure password (20 chars)
-->

```bash
op item create --generate-password=20,letters,digits,symbols --dry-run --format=json | jq -r '.fields[] | select(.id == "password") | .value' | pbcopy && echo "Generated password copied to clipboard"
```

---

## Generate Memorable Password

<!--
description: Generate a memorable passphrase (4 words)
-->

```bash
op item create --generate-password='words,4,en' --dry-run --format=json | jq -r '.fields[] | select(.id == "password") | .value' | pbcopy && echo "Generated passphrase copied to clipboard"
```

---

## Generate PIN

<!--
description: Generate a numeric PIN (6 digits)
-->

```bash
op item create --generate-password=6,digits --dry-run --format=json | jq -r '.fields[] | select(.id == "password") | .value' | pbcopy && echo "Generated PIN copied to clipboard"
```

---

## List Vaults

<!--
description: List all available vaults
-->

```bash
op vault list --format=json | jq -r '.[] | "\(.name) (\(.id))"'
```

---

## Get Vault Details

<!--
description: Get details about a specific vault
-->

```bash
read -p "Vault name: " vault && op vault get "$vault" --format=json | jq '.'
```

---

## Account Info

<!--
description: Show current signed-in account information
-->

```bash
op whoami --format=json | jq '.'
```

---

## List Accounts

<!--
description: List all configured 1Password accounts
-->

```bash
op account list --format=json | jq -r '.[] | "\(.email) (\(.url))"'
```

---

## Sign In

<!--
description: Sign in to your 1Password account
-->

```bash
eval $(op signin)
```

---

## Sign Out

<!--
description: Sign out of the current 1Password session
-->

```bash
op signout && echo "Signed out successfully"
```

---

## Lock 1Password

<!--
description: Lock 1Password app
-->

```applescript
tell application "1Password" to activate
tell application "System Events"
    keystroke "l" using {command down, shift down}
end tell
```

---

## Create Login Item

<!--
description: Create a new login item interactively
-->

```bash
read -p "Title: " title && read -p "Username: " user && read -p "Website URL: " url && op item create --category=Login --title="$title" username="$user" --url="$url" --generate-password=20,letters,digits,symbols && echo "Login item created"
```

---

## Create Secure Note

<!--
description: Create a new secure note
-->

```bash
read -p "Title: " title && read -p "Note content: " content && op item create --category="Secure Note" --title="$title" "notesPlain=$content" && echo "Secure note created"
```

---

## Create API Credential

<!--
description: Create a new API credential item
-->

```bash
read -p "Title: " title && read -p "API Key: " key && op item create --category="API Credential" --title="$title" credential="$key" && echo "API credential created"
```

---

## Archive Item

<!--
description: Move an item to the archive
-->

```bash
read -p "Item name to archive: " name && op item edit "$name" --archive && echo "Item archived"
```

---

## Delete Item

<!--
description: Permanently delete an item
-->

```bash
read -p "Item name to delete: " name && read -p "Are you sure? (yes/no): " confirm && [ "$confirm" = "yes" ] && op item delete "$name" && echo "Item deleted"
```

---

## Read Secret Reference

<!--
description: Read a secret using op:// reference format
-->

```bash
read -p "Secret reference (op://vault/item/field): " ref && op read "$ref"
```

---

## Inject Secrets to Env

<!--
description: Run a command with secrets injected as environment variables
-->

```bash
read -p "Command to run: " cmd && op run -- $cmd
```

---

## Export Item to JSON

<!--
description: Export an item's data as JSON
-->

```bash
read -p "Item name: " name && op item get "$name" --format=json > ~/Desktop/"$name".json && echo "Exported to ~/Desktop/$name.json"
```

---

## Check CLI Version

<!--
description: Show 1Password CLI version
-->

```bash
op --version
```

---

## Update CLI

<!--
description: Check for and install CLI updates
-->

```bash
op update
```
