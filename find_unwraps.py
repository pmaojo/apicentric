import os

for root, dirs, files in os.walk('src'):
    for file in files:
        if not file.endswith('.rs'): continue
        filepath = os.path.join(root, file)

        # skip test files
        if 'tests.rs' in file or 'tests' in filepath:
            continue

        with open(filepath, 'r') as f:
            lines = f.readlines()
            in_test_mod = False
            for i, line in enumerate(lines):
                if '#[cfg(test)]' in line:
                    in_test_mod = True

                if '#[test]' in line or '#[tokio::test]' in line:
                    continue

                if not in_test_mod and '.unwrap()' in line and not line.strip().startswith('//'):
                    print(f"{filepath}:{i+1}:{line.strip()}")
