# generate most of the assets needed in the project
from pathlib import Path
import json

root = Path(__file__).parent.parent / "src"

reference = (root / "assets/reference").glob("*")
ref_vars = []
for i, path in enumerate(sorted(reference)):
    name = f"reference_{i:02}"
    print(f'import {name} from "./{path.relative_to(root)}";')
    ref_vars.append(name)

seed_tags = (root / "assets/seed_tags").glob("*")
seed_vars = []
for i, path in enumerate(sorted(seed_tags)):
    name = f"seed_{i:02}"
    print(f'import {name} from "./{path.relative_to(root)}";')
    seed_vars.append(name)

print()
print(f"const reference = [{', '.join(ref_vars)}];")
print(f"const seed_tags = [{', '.join(seed_vars)}];")
