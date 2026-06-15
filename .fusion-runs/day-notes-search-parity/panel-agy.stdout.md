I will search the repository to find references to `print-timeout` or `timeout` command line options to understand the context of the user request.
I will search for any CLI tools or configuration scripts in the repository that might define or implement the `--print-timeout` option.
I will view the `scripts/agentic/agy-devtools.sh` script to understand what `--print-timeout` is and how it is used.
I will read lines 600 to 650 of `scripts/agentic/agy-devtools.sh` to see how `PRINT_TIMEOUT` is passed to the `agy` command line invocation.
I will run the command `/Users/johnlindquist/.local/bin/agy --help` to check the documentation for the `--print-timeout` flag of the `agy` CLI.
The `--print-timeout` option is a configuration flag for the **Antigravity CLI** (`agy`). 

### Purpose
It defines the maximum duration the CLI will wait for the model to generate and stream the response when running in **non-interactive print mode** (using the `--print` or `-p` flags). If the generation exceeds this timeout, the process will exit or time out.

### Default Value
* The default timeout is **`5m0s`** (5 minutes).

### Usage Example
You can override this timeout using standard duration formats (e.g., `s` for seconds, `m` for minutes):

```bash
# Run a prompt with an extended timeout of 10 minutes
agy --print "Refactor the day notes search container" --print-timeout 10m
```

*(Note: In the repository's helper script [agy-devtools.sh](file:///Users/johnlindquist/dev/script-kit-gpui/scripts/agentic/agy-devtools.sh#L22), the print timeout defaults to `5m` but can also be customized via `--print-timeout` when invoking DevTools).*
