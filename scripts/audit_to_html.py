#!/usr/bin/env python3
import json
import sys
from pathlib import Path

def main():
    if len(sys.argv) != 3:
        print(f"Usage: {sys.argv[0]} <audit.json> <output.html>")
        sys.exit(1)

    audit_path = Path(sys.argv[1])
    output_path = Path(sys.argv[2])

    with audit_path.open() as f:
        data = json.load(f)

    vulnerabilities = data.get("vulnerabilities", {}).get("list", [])
    found = data.get("vulnerabilities", {}).get("found", False)
    count = len(vulnerabilities)

    html = [
        "<html>",
        "<head><title>Cargo Audit Report</title></head>",
        "<body>",
        f"<h1>Cargo Audit Report</h1>",
        f"<p><b>Vulnerabilities found:</b> {'Yes' if found else 'No'}</p>",
        f"<p><b>Total vulnerabilities:</b> {count}</p>",
    ]

    if count > 0:
        html.append("<ul>")
        for vuln in vulnerabilities:
            advisory = vuln.get("advisory", {})
            html.append("<li>")
            html.append(f"<b>{advisory.get('package', 'unknown')}</b> - {advisory.get('title', '')}<br>")
            html.append(f"<b>ID:</b> {advisory.get('id', '')}<br>")
            html.append(f"<b>URL:</b> <a href='{advisory.get('url', '')}'>{advisory.get('url', '')}</a><br>")
            html.append(f"<b>Severity:</b> {advisory.get('cvss', {}).get('severity', 'N/A')}<br>")
            html.append(f"<b>Description:</b> {advisory.get('description', '')}<br>")
            html.append("</li>")
        html.append("</ul>")
    else:
        html.append("<p>No vulnerabilities found. \u2705</p>")

    html.append("</body></html>")

    with output_path.open("w") as f:
        f.write("\n".join(html))

if __name__ == "__main__":
    main()
