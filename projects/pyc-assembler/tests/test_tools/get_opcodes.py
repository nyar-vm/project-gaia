import opcode
import json

with open('opcode_map.json', 'w') as f:
    json.dump({name: value for name, value in opcode.opmap.items()}, f, indent=4)