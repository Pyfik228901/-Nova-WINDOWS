import subprocess
import json

def test_lsp():
    proc = subprocess.Popen(
        ['target/release/ns.exe', 'lsp'],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE
    )

    req = json.dumps({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {}
    }).encode("utf-8")

    header = f"Content-Length: {len(req)}\r\n\r\n".encode("utf-8")
    proc.stdin.write(header + req)
    proc.stdin.flush()

    # Read response
    header_line = proc.stdout.readline().decode("utf-8")
    print("Received Header:", header_line.strip())
    empty_line = proc.stdout.readline()

    assert header_line.startswith("Content-Length:")
    content_length = int(header_line.split(":")[1].strip())
    body = proc.stdout.read(content_length).decode("utf-8")
    print("Received Body:", body)

    data = json.loads(body)
    assert data["id"] == 1
    assert data["result"]["serverInfo"]["name"] == "ns-lsp"
    assert "capabilities" in data["result"]
    print("[OK] LSP initialize response verified successfully")

    # Clean exit
    shutdown_req = json.dumps({"jsonrpc": "2.0", "id": 2, "method": "shutdown"}).encode("utf-8")
    proc.stdin.write(f"Content-Length: {len(shutdown_req)}\r\n\r\n".encode("utf-8") + shutdown_req)
    proc.stdin.flush()
    proc.stdin.close()
    proc.terminate()

if __name__ == "__main__":
    test_lsp()
