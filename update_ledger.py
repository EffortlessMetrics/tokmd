import json

filepath = '.jules/compat/ledger.json'
with open(filepath, 'r') as f:
    data = json.load(f)

new_entry = {
    "run_id": "run-20240525-compat-004",
    "date": "2024-05-25",
    "target": "tokmd",
    "improvement": "fixed unused code warnings in cockpit command when git disabled"
}

data.append(new_entry)

with open(filepath, 'w') as f:
    json.dump(data, f, indent=2)

print("Ledger updated.")
