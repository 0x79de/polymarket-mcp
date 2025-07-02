#!/usr/bin/env python3
import re
from pathlib import Path

def get_md_files():
    """Get all markdown files except README.md"""
    md_files = []
    for file in Path('.').glob('*.md'):
        if file.name != 'README.md':
            md_files.append(file)
    return sorted(md_files)

def parse_date_from_filename(filename):
    """Extract date from filename like '2025-05-may.md' or use friendly names"""
    # Try to match date pattern first
    match = re.match(r'(\d{4})-(\d{2})-(\w+)\.md', filename)
    if match:
        year, _, month_name = match.groups()
        return f"{month_name.title()} {year}"
    
    # Handle special filenames with friendly names
    friendly_names = {
        'CLAUDE.local.md': 'Claude.Local',
    }
    
    return friendly_names.get(filename, filename.replace('.md', '').replace('_', ' ').title())

def update_readme():
    """Update README.md with list of markdown files"""
    md_files = get_md_files()
    
    # Read current README
    with open('README.md', 'r') as f:
        content = f.read()
    
    # Generate file list
    file_list = "## Archive\n\n"
    for file in md_files:
        display_name = parse_date_from_filename(file.name)
        file_list += f"- [{display_name}]({file.name})\n"
    
    # Check if archive section exists
    if "## Archive" in content:
        # Replace existing archive section
        before_archive = content.split("## Archive")[0].rstrip()
        content = before_archive + "\n\n" + file_list
    else:
        # Add archive section at the end
        content = content.rstrip() + "\n\n" + file_list
    
    # Write updated README
    with open('README.md', 'w') as f:
        f.write(content)
    
    print(f"Updated README.md with {len(md_files)} files")

if __name__ == "__main__":
    update_readme()