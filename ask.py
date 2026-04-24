import requests
import json
import sys
import os
import subprocess
import argparse
import ollama

def call_remote_api(prompt):
    """Sends a prompt to the DashScope API and handles the streaming response."""
    headers = {
        "Authorization": f"Bearer {os.environ.get('DEEPSEEK_API_KEY')}",
        "Content-Type": "application/json"
    }
    json_data = {
        "stream": False,  # For scoring, we don't need to stream
        "model": "deepseek-v4-pro",
        "messages": [
            {"role": "system", "content": "You are a helpful assistant."},
            {"role": "user", "content": prompt}
        ]
    }
    try:
        response = requests.post(
            "https://api.deepseek.com/chat/completions",
            headers=headers,
            json=json_data
        )
        response.raise_for_status()
        content = response.json().get('choices', [{}])[0].get('message', {}).get('content', '')
        print(content, end='', flush=True)
    except requests.exceptions.RequestException as e:
        print(f"Error calling remote API: {e}", file=sys.stderr)
        sys.exit(1)

def call_ollama_local(prompt, model_name):
    """Sends a prompt to the local Ollama model."""
    try:
        response = ollama.chat(
            model=model_name,
            messages=[
                {'role': 'user', 'content': prompt}
            ]
        )
        print(response['message']['content'], end='', flush=True)
    except Exception as e:
        print(f"Error calling Ollama: {e}", file=sys.stderr)
        print("Please ensure Ollama is running and the model is available.", file=sys.stderr)
        sys.exit(1)

def main():
    """Main function to handle command-line arguments and call the appropriate API."""
    parser = argparse.ArgumentParser(description="Ask an AI model a question.")
    parser.add_argument("prompt", nargs='*', help="The prompt to send to the model.")
    parser.add_argument("--local", action="store_true", help="Use the local Ollama model.")
    parser.add_argument("--model", default="gemma3:1b", help="The name of the local model to use.")

    args = parser.parse_args()

    if not args.prompt:
        if sys.stdin.isatty():
            print("Enter your prompt:", file=sys.stderr)
            prompt_text = sys.stdin.read()
        else:
            prompt_text = sys.stdin.read()
    else:
        prompt_text = ' '.join(args.prompt)

    if not prompt_text.strip():
        print("Error: Prompt is empty.", file=sys.stderr)
        sys.exit(1)

#    if args.local:
#        call_ollama_local(prompt_text, args.model)
#    else:
    call_remote_api(prompt_text)

if __name__ == "__main__":
    try:
        main()
    except KeyboardInterrupt:
        pass
