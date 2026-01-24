# lachat

lachat is a command-line client for interacting with [llama-server](https://github.com/ggml-org/llama.cpp). It can launch, reuse, or terminate a [llama-server](https://github.com/ggml-org/llama.cpp) instance automatically, send prompts to the model, stream responses, and persist chat history.

If no [llama-server](https://github.com/ggml-org/llama.cpp) is running for the current session, it is started automatically. Server state and chat history are stored in the session directory.

## Help message

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
        Chat identifier. Enables persistent chat history stored in the session.
        If no prompt is provided, the chat history is printed as JSON.
  -S, --session <PATH>
        Path to the session directory. Stores server state and chat history.
        Defaults to $LACHAT_SESSION or /tmp/lachat.
  -t, --temp --temperature <VALUE>
        Sampling temperature (float).
  -x, --max-tokens <VALUE>
        Maximum number of tokens to generate.
  -e, --code-only
        Output only code blocks from the model response.
  -E, --first-code
        Output only the first code block from the model response.
  -l, --last
        Print the last assistant message.
        Uses the specified --chat or the most recent session.
  -i, --interactive
        Start an interactive chat session.
  -L, --list --chat-list
        List stored chats
  -C, --clear
        Clear chats (all or specified with --chat)
  -o, --open
        Open the llama-server web interface in the default browser.
  -k, --kill
        Kill the currently running llama-server.
  -h, --help
        Print this help message and exit.
  -V, --version
        Print version information and exit.
PASSTHROUGH ARGUMENTS:
  -- <ARGS>...
        All arguments after '--' are passed directly to llama-server.
SERVER BEHAVIOR:
  If no server is running for the session, llama-server is started automatically.
  If server arguments change (host/port), the old server is terminated and restarted.
PROMPT SOURCES (in order):
  stdin (if not empty)
  --prompt arguments
EXAMPLES:
  lachat hello
  lachat -m qwen -c mychat -p "main.rs" -p "refactor this code"
  cat main.go | lachat -s system.txt
  lachat -- --host 127.0.0.1 --port 5050
```
