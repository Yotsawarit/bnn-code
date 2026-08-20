#!/usr/bin/env python3
"""
BNN Autoregressive ONNX Generator
=================================
Runs a small greedy / top-k autoregressive generation loop on top of the
exported CodeBERTa model.onnx using onnxruntime.

The base model is a *masked* LM, but we can still drive it autoregressively
by reading the logits at the last position and sampling the next token.

Usage:
    python scripts/generate.py "def add(a, b):"            # latest local model
    python scripts/generate.py "def add(a, b):" --model huggingface--CodeBERTa-small-v1
    python scripts/generate.py "def " --tokens 24 --top-k 10 --temperature 0.8
"""

import argparse
import sys
from pathlib import Path

from download_model import MODELS_DIR, latest_local_model, list_local_models

REQUIRED_PKGS = ["onnxruntime", "numpy", "transformers"]


def check_deps():
    missing = []
    for pkg in REQUIRED_PKGS:
        try:
            __import__(pkg)
        except ImportError:
            missing.append(pkg)
    if missing:
        import subprocess
        print(f"Installing missing packages: {', '.join(missing)}")
        subprocess.check_call(
            [sys.executable, "-m", "pip", "install", *missing, "-q", "--break-system-packages"]
        )


def load(model_dir: Path):
    import onnxruntime as ort
    from transformers import AutoTokenizer

    onnx_path = model_dir / "model.onnx"
    if not onnx_path.exists():
        raise FileNotFoundError(f"model.onnx not found in {model_dir}")

    tokenizer = AutoTokenizer.from_pretrained(model_dir)
    session = ort.InferenceSession(str(onnx_path))
    return tokenizer, session


def sample_next(logits, top_k: int, temperature: float):
    import numpy as np

    if temperature <= 0:
        return int(np.argmax(logits))

    logits = logits.astype("float32") / temperature
    if top_k > 0:
        idx = np.argpartition(logits, -top_k)[-top_k:]
        logits = logits[idx]
    else:
        idx = np.arange(len(logits))

    probs = np.exp(logits - logits.max())
    probs = probs / probs.sum()
    choice = int(np.random.choice(len(probs), p=probs))
    return int(idx[choice])


def generate(tokenizer, session, model_dir, prompt: str, max_new_tokens: int,
             top_k: int, temperature: float, eos_id: int):
    import json
    import numpy as np

    # Disable EOS if it falls outside the model's output vocab range
    # (e.g. codeparrot's eos_token_id=50256 but logits dim is 32768).
    cfg = json.loads((model_dir / "config.json").read_text())
    vocab = cfg.get("vocab_size", len(tokenizer))
    if eos_id is None or eos_id >= vocab:
        eos_id = -1

    input_ids = tokenizer(prompt, return_tensors="np")["input_ids"].astype("int64")
    generated = list(input_ids[0])
    repeat = 0

    for _ in range(max_new_tokens):
        ids = np.array([generated], dtype="int64")
        logits = session.run(["logits"], {"input_ids": ids})[0]
        next_logits = logits[0, -1, :]
        next_id = sample_next(next_logits, top_k, temperature)

        if next_id == eos_id:
            break

        # Degeneracy guard: stop if the same token repeats too many times.
        if generated and next_id == generated[-1]:
            repeat += 1
            if repeat >= 4:
                break
        else:
            repeat = 0

        generated.append(next_id)

    return tokenizer.decode(generated, skip_special_tokens=True)


def main():
    parser = argparse.ArgumentParser(description="Autoregressive ONNX generation")
    parser.add_argument("prompt", nargs="?", default="def ")
    parser.add_argument("--model", default=None,
                        help="Local model dir name under models/ (default: latest)")
    parser.add_argument("--tokens", type=int, default=24, help="Max new tokens")
    parser.add_argument("--top-k", type=int, default=0, help="0 = full vocab")
    parser.add_argument("--temperature", type=float, default=0.0,
                        help="0 = greedy, >0 = sampled")
    parser.add_argument("--seed", type=int, default=None)
    args = parser.parse_args()

    check_deps()

    if args.seed is not None:
        import numpy as np
        np.random.seed(args.seed)

    if args.model:
        model_dir = MODELS_DIR / args.model
        if not model_dir.exists():
            print(f"Model dir not found: {model_dir}")
            return
    else:
        model_dir = latest_local_model()
        if model_dir is None:
            print("No local models. Run: python scripts/download_model.py")
            return

    print(f"Model: {model_dir}")
    tokenizer, session = load(model_dir)
    eos_id = tokenizer.eos_token_id if tokenizer.eos_token_id is not None else -1

    print(f"Prompt: {args.prompt!r}")
    out = generate(tokenizer, session, model_dir, args.prompt, args.tokens,
                   args.top_k, args.temperature, eos_id)
    print("\n" + out)


if __name__ == "__main__":
    main()
