import os

EXCLUDED_DIRS = ['node_modules', '.angular', '.git', '__pycache__', 'dist', 'build', 'coverage', 'venv', 'env']

def print_directory_structure(start_path, prefix=''):
    for root, dirs, files in os.walk(start_path):
        # Filter out excluded directories
        dirs[:] = [d for d in dirs if d not in EXCLUDED_DIRS]

        level = root.replace(start_path, '').count(os.sep)
        indent = ' ' * 4 * (level)
        print('{}{}/'.format(indent, os.path.basename(root)))
        sub_indent = ' ' * 4 * (level + 1)
        for f in files:
            print('{}{}'.format(sub_indent, f))

if __name__ == "__main__":
    print("Project Directory Structure:\n")
    print_directory_structure('.')
