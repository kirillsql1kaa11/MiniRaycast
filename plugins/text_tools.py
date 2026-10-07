import sys
import json
import base64

def process(query: str) -> dict:
    parts = query.strip().split(" ", 1)
    if len(parts) < 2:
        return {"success": False, "error": "Need command and text"}
    cmd = parts[0].lower()
    text = parts[1]
    
    if cmd == "base64" or cmd == "b64":
        encoded = base64.b64encode(text.encode("utf-8")).decode("utf-8")
        return {"success": True, "result": encoded, "label": "Base64"}
    elif cmd == "unbase64" or cmd == "deb64":
        try:
            decoded = base64.b64decode(text.encode("utf-8")).decode("utf-8")
            return {"success": True, "result": decoded, "label": "Decoded Base64"}
        except Exception:
            return {"success": False, "error": "Invalid base64"}
    elif cmd == "upper":
        return {"success": True, "result": text.upper(), "label": "Uppercase"}
    elif cmd == "lower":
        return {"success": True, "result": text.lower(), "label": "Lowercase"}
    elif cmd == "reverse" or cmd == "rev":
        return {"success": True, "result": text[::-1], "label": "Reversed"}
    elif cmd == "length" or cmd == "len":
        return {"success": True, "result": str(len(text)), "label": "Length"}
    return {"success": False, "error": "Unknown text command"}

if __name__ == "__main__":
    if len(sys.argv) > 1:
        query_text = " ".join(sys.argv[1:])
        print(json.dumps(process(query_text), ensure_ascii=False))
    else:
        print(json.dumps({"success": False, "error": "No query provided"}))
