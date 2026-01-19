# lachat

lacht - минималистичный CLI клиент для [llama-server](https://github.com/ggml-org/llama.cpp).

### Help message

```text
lachat — minimal CLI client for llama-server
USAGE:
  lachat [OPTIONS] [PROMPT...] -- [LLAMA_SERVER_ARGS...]
OPTIONS:
  -p, --prompt <TEXT|PATH>
          Prompt to send to the model. Can be specified multiple times.
          If a file path is provided, its contents are used.
  -m, --model <MODEL>
          Model name to use. If not specified, the first available model is used.
          Fuzzy matching is applied when resolving the model name.
  -s, --system <TEXT|PATH>
          System prompt (system message).
  -c, --chat <ID>
          Chat ID or chat name.
          Enables persistent chat history stored in the session.
  -S, --session <PATH>
          Path to the session directory.
          Defaults to $LACHAT_SESSION or the built-in default path /tmp/lachat
  -t, --temperature <VALUE>
          Sampling temperature (float).
  -x, --max-tokens <VALUE>
          Sets maximum tokens to generate.
  -i, --interactive
          Start an interactive chat session.
  -k, --kill
          Kill the currently running llama-server.
  -h, --help
          Print this help message and exit.
  -V, --version
          Print version information and exit.
PASSTHROUGH ARGUMENTS:
  -- <ARGS>...
          All arguments after '--' are passed directly to llama-server.
PROMPT SOURCES (in order):
  1. stdin (if not empty)
  2. --prompt arguments
EXAMPLES:
  lachat hello!
  cat main.rs | lachat -m qwen -p "code refactor" -c mychat -- --port 5050
```
