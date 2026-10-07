import sys
import json
import re

RATES_TO_USD = {
    "USD": 1.0,
    "EUR": 1.08,
    "GBP": 1.28,
    "RUB": 0.0105,
    "JPY": 0.0067,
    "CNY": 0.138,
    "KZT": 0.0021,
    "TRY": 0.029,
    "CAD": 0.74,
    "CHF": 1.13,
    "AUD": 0.65,
    "BTC": 64500.0,
    "ETH": 3450.0
}

SYMBOLS = {
    "$": "USD",
    "€": "EUR",
    "£": "GBP",
    "₽": "RUB",
    "¥": "CNY",
    "₸": "KZT"
}

def parse_and_convert(query: str) -> dict:
    q = query.strip().upper()
    for sym, code in SYMBOLS.items():
        q = q.replace(sym, f" {code} ")
    pattern = r"([\d\.,]+)\s*([A-Z]{3,4})\s*(?:TO|IN|\=|->|В|К)\s*([A-Z]{3,4})"
    match = re.search(pattern, q, re.IGNORECASE)
    if not match:
        return {"success": False, "error": "Invalid currency query format"}
    raw_amount, from_curr, to_curr = match.groups()
    try:
        amount = float(raw_amount.replace(",", "."))
    except ValueError:
        return {"success": False, "error": "Invalid number format"}
    from_curr = from_curr.upper()
    to_curr = to_curr.upper()
    if from_curr not in RATES_TO_USD or to_curr not in RATES_TO_USD:
        return {"success": False, "error": f"Unsupported currency: {from_curr} or {to_curr}"}
    usd_val = amount * RATES_TO_USD[from_curr]
    converted = usd_val / RATES_TO_USD[to_curr]
    if converted >= 1:
        formatted_val = f"{converted:,.2f}"
    else:
        formatted_val = f"{converted:,.6f}"
    return {
        "success": True,
        "from_amount": amount,
        "from_currency": from_curr,
        "to_currency": to_curr,
        "result": converted,
        "formatted": f"{amount:g} {from_curr} = {formatted_val} {to_curr}"
    }

if __name__ == "__main__":
    if len(sys.argv) > 1:
        query_text = " ".join(sys.argv[1:])
        print(json.dumps(parse_and_convert(query_text), ensure_ascii=False))
    else:
        print(json.dumps({"success": False, "error": "No query provided"}))
