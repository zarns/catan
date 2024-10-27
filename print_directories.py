import os
from pathlib import Path
from datetime import datetime
import sys

def print_directory_structure(startpath, output_file, exclude_dirs=None):
    """
    Print the directory structure starting from startpath and save to a file.
    
    Args:
        startpath (str): The root directory to start from
        output_file (file): File object to write to
        exclude_dirs (set): Set of directory names to exclude
    """
    if exclude_dirs is None:
        exclude_dirs = {'.git', '__pycache__', 'target', 'node_modules', '.next', 'dist'}
    
    def write_line(line):
        print(line)  # Print to console
        output_file.write(line + '\n')  # Write to file
    
    write_line(f"Catan Project Directory Structure")
    write_line(f"Generated at: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}")
    write_line(f"Root path: {os.path.abspath(startpath)}")
    write_line("\n")
    
    for root, dirs, files in os.walk(startpath):
        # Skip excluded directories
        dirs[:] = [d for d in dirs if d not in exclude_dirs]
        
        # Calculate relative path from startpath
        rel_path = os.path.relpath(root, startpath)
        level = len(Path(rel_path).parts)
        
        if rel_path == '.':
            write_line('📁 catan/')
            continue
            
        indent = '│   ' * (level - 1)
        folder_name = os.path.basename(root)
        
        # Print directory name
        write_line(f'{indent}📁 {folder_name}/')
        
        # Print files
        subindent = '│   ' * level
        for file in sorted(files):
            if file.endswith(('.py', '.rs', '.ts', 'Cargo.toml', '.md', '.json')):
                write_line(f'{subindent}📄 {file}')

def main():
    # Get the directory where the script is located
    script_dir = os.path.dirname(os.path.abspath(__file__))
    
    output_path = os.path.join(script_dir, 'directory_structure.txt')
    
    with open(output_path, 'w', encoding='utf-8') as f:
        print_directory_structure(script_dir, f)
    
    print(f"\nDirectory structure has been saved to: {output_path}")

if __name__ == "__main__":
    main()