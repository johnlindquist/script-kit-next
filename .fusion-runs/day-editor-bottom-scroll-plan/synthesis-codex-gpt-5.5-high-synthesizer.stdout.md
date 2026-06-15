The original task is missing. The only provided task content is:

```text
--input -
```

That is not enough to evaluate, implement, or review anything. Please resend the actual task, prompt, diff, file, or artifact that was meant to be passed in.

If this came from an automated fusion run, the most likely hypothesis is that the wrapper intended `--input -` to read from stdin but the stdin content did not reach the panel. Check the pipeline’s stdin forwarding or retry with the task content inline.

Conditional note: if `--input -` itself was the intended subject, say that explicitly. In that case the relevant question is CLI parsing/stdin semantics, especially whether `-` means stdin and whether `--input -` behaves the same as `--input=-`.
