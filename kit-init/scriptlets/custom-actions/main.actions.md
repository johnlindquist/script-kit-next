# Shared Actions

### Copy URL

<!-- description: Copy the selected URL -->

```bash
echo -n "{{content}}" | pbcopy
```

### Open in Safari

<!-- description: Open the selected URL in Safari -->

```bash
open -a Safari "{{content}}"
```

### Open in Chrome

<!-- description: Open the selected URL in Google Chrome -->

```bash
open -a "Google Chrome" "{{content}}"
```
