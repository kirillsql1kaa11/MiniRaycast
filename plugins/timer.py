import sys
import json
import re

def parse_timer(query: str) -> dict:
    q = query.strip()
    match = re.search(r"(?:timer|таймер)\s+(\d+)\s*([smhсмчминсек]+)?(?:\s+(.*))?", q, re.IGNORECASE)
    if not match:
        return {"success": False, "error": "Invalid timer query"}
    amount, unit, label = match.groups()
    amount = int(amount)
    unit = (unit or "m").lower()
    label = (label or "Timer").strip()
    seconds = amount
    if unit in ["m", "мин", "min", "минут"]:
        seconds = amount * 60
    elif unit in ["h", "ч", "час", "часов", "hour", "hours"]:
        seconds = amount * 3600
    elif unit in ["s", "сек", "sec", "секунд"]:
        seconds = amount
    minutes = seconds // 60
    remaining_secs = seconds % 60
    time_str = f"{minutes}m {remaining_secs}s" if minutes else f"{remaining_secs}s"
    return {
        "success": True,
        "seconds": seconds,
        "label": label,
        "formatted": f"{label}: {time_str}"
    }

if __name__ == "__main__":
    if len(sys.argv) > 1:
        query_text = " ".join(sys.argv[1:])
        print(json.dumps(parse_timer(query_text), ensure_ascii=False))
    else:
        print(json.dumps({"success": False, "error": "No query provided"}))
