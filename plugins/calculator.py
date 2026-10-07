import sys
import json
import math

SAFE_NAMES = {
    'sin': math.sin,
    'cos': math.cos,
    'tan': math.tan,
    'asin': math.asin,
    'acos': math.acos,
    'atan': math.atan,
    'sqrt': math.sqrt,
    'log': math.log,
    'log10': math.log10,
    'exp': math.exp,
    'pi': math.pi,
    'e': math.e,
    'pow': pow,
    'abs': abs,
    'round': round,
    'floor': math.floor,
    'ceil': math.ceil
}

def clean_expression(expr: str) -> str:
    expr = expr.strip()
    expr = expr.replace('^', '**')
    expr = expr.replace('×', '*').replace('÷', '/')
    return expr

def evaluate(expr: str) -> dict:
    try:
        cleaned = clean_expression(expr)
        code = compile(cleaned, "<string>", "eval")
        for name in code.co_names:
            if name not in SAFE_NAMES:
                return {"success": False, "error": f"Disallowed function: {name}"}
        result = eval(code, {"__builtins__": {}}, SAFE_NAMES)
        if isinstance(result, float) and result.is_integer():
            result = int(result)
        elif isinstance(result, float):
            result = round(result, 6)
        return {
            "success": True,
            "expression": expr,
            "result": str(result)
        }
    except Exception as e:
        return {"success": False, "error": str(e)}

if __name__ == "__main__":
    if len(sys.argv) > 1:
        query = " ".join(sys.argv[1:])
        print(json.dumps(evaluate(query), ensure_ascii=False))
    else:
        print(json.dumps({"success": False, "error": "No expression provided"}))
