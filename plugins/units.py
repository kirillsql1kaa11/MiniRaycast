import sys
import json
import re

LENGTH = {
    'm': 1.0, 'km': 1000.0, 'cm': 0.01, 'mm': 0.001,
    'mi': 1609.344, 'mile': 1609.344, 'miles': 1609.344,
    'ft': 0.3048, 'feet': 0.3048, 'foot': 0.3048,
    'in': 0.0254, 'inch': 0.0254, 'inches': 0.0254,
    'yd': 0.9144, 'yard': 0.9144
}

MASS = {
    'g': 1.0, 'kg': 1000.0, 'mg': 0.001,
    'lb': 453.592, 'lbs': 453.592, 'pound': 453.592, 'pounds': 453.592,
    'oz': 28.3495, 'ounce': 28.3495
}

DATA = {
    'b': 1.0, 'byte': 1.0, 'bytes': 1.0,
    'kb': 1024.0, 'mb': 1024.0**2, 'gb': 1024.0**3, 'tb': 1024.0**4
}

TIME = {
    's': 1.0, 'sec': 1.0, 'second': 1.0, 'seconds': 1.0,
    'm': 60.0, 'min': 60.0, 'minute': 60.0, 'minutes': 60.0,
    'h': 3600.0, 'hr': 3600.0, 'hour': 3600.0, 'hours': 3600.0,
    'd': 86400.0, 'day': 86400.0, 'days': 86400.0
}

def convert_temp(val: float, u1: str, u2: str) -> float:
    c = val
    if u1 in ['f', 'fahrenheit']:
        c = (val - 32) * 5 / 9
    elif u1 in ['k', 'kelvin']:
        c = val - 273.15
    if u2 in ['f', 'fahrenheit']:
        return (c * 9 / 5) + 32
    elif u2 in ['k', 'kelvin']:
        return c + 273.15
    return c

def convert_units(query: str) -> dict:
    match = re.search(r"([\d\.,]+)\s*([a-zA-Z]+)\s*(?:to|in|\=|->|в|к)\s*([a-zA-Z]+)", query.strip(), re.IGNORECASE)
    if not match:
        return {"success": False, "error": "Invalid unit conversion query"}
    raw_val, u1, u2 = match.groups()
    try:
        val = float(raw_val.replace(',', '.'))
    except ValueError:
        return {"success": False, "error": "Invalid numeric value"}
    u1 = u1.lower()
    u2 = u2.lower()
    if u1 in ['c', 'f', 'k', 'celsius', 'fahrenheit', 'kelvin'] and u2 in ['c', 'f', 'k', 'celsius', 'fahrenheit', 'kelvin']:
        res = convert_temp(val, u1, u2)
        return {
            "success": True,
            "result": round(res, 2),
            "formatted": f"{val:g} {u1} = {round(res, 2)} {u2}"
        }
    for category in [LENGTH, MASS, DATA, TIME]:
        if u1 in category and u2 in category:
            base = val * category[u1]
            res = base / category[u2]
            return {
                "success": True,
                "result": round(res, 4),
                "formatted": f"{val:g} {u1} = {round(res, 4)} {u2}"
            }
    return {"success": False, "error": f"Cannot convert {u1} to {u2}"}

if __name__ == "__main__":
    if len(sys.argv) > 1:
        query_text = " ".join(sys.argv[1:])
        print(json.dumps(convert_units(query_text), ensure_ascii=False))
    else:
        print(json.dumps({"success": False, "error": "No query provided"}))
