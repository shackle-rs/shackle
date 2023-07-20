# Finds mermaid code blocks and renders them

import json
import sys
import re
from pathlib import Path
from tempfile import TemporaryDirectory
from subprocess import run

if __name__ == "__main__":
    if len(sys.argv) > 1:
        if sys.argv[1] == "supports":
            sys.exit(0)

    context, book = json.load(sys.stdin)
    cfg = context.get("config", {}).get("preprocessor", {}).get("mermaid", {})
    prerender = cfg.get("prerender", False)
    regex = r"^[^\S\n]*```(?:mermaid)(\r?\n([\s\S]*?))```[^\S\n]*$"
    todo = list(book["sections"])
    while len(todo) > 0:
        item = todo.pop()
        chapter = item.get("Chapter")
        if chapter is None:
            continue
        for sub_item in chapter["sub_items"]:
            todo.append(sub_item)
        pos = 0
        output = []
        content = chapter["content"]
        if prerender:
            # Run mermaid-cli to pre-render as SVG
            with TemporaryDirectory() as td:
                temp_dir = Path(td)
                for i, match in enumerate(re.finditer(regex, content, re.M)):
                    mermaid = match.group(2)
                    src = temp_dir / f"diagram-{i}.mmd"
                    src.write_text(mermaid)
                    dst = src.with_suffix(".svg")
                    run(
                        [
                            "npx",
                            "--yes",
                            "-p",
                            "@mermaid-js/mermaid-cli",
                            "mmdc",
                            "-q",
                            "-b",
                            "transparent",
                            "-i",
                            src,
                            "-o",
                            dst,
                        ],
                        shell=True,
                        check=True,
                    )
                    svg = dst.read_text()
                    start, end = match.span()
                    output.append(content[pos:start])
                    output.append(svg)
                    pos = end
            output.append(content[pos:])
        else:
            # Render in browser
            had_match = False
            for i, match in enumerate(re.finditer(regex, content, re.M)):
                mermaid = match.group(2)
                start, end = match.span()
                output.append(content[pos:start])
                output.append('<pre class="mermaid">')
                output.append(mermaid)
                output.append("</pre>")
                pos = end
                had_match = True
            output.append(content[pos:])
            if had_match:
                output.append(
                    '\n<script src="https://cdn.jsdelivr.net/npm/mermaid@latest/dist/mermaid.min.js"></script>'
                )
                output.append(
                    "\n<script>mermaid.initialize({ startOnLoad: true });</script>"
                )
        chapter["content"] = "".join(output)
    json.dump(book, sys.stdout, ensure_ascii=False)
