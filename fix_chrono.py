#!/usr/bin/env python3
import os
import re

def fix_chrono_some_in_file(file_path):
    """Исправляет все chrono::Some на просто Some"""
    try:
        with open(file_path, 'r', encoding='utf-8') as f:
            content = f.read()
        
        # Заменяем chrono::Some на Some
        original_content = content
        content = re.sub(r'chrono::Some\(', r'Some(', content)
        
        # Также исправляем вложенные Some(Some(...).to_rfc3339())
        content = re.sub(r'Some\(Some\(([^)]+)\)\.to_rfc3339\(\)\)\.to_rfc3339\(\)', r'Some(\1.to_rfc3339())', content)
        
        if content != original_content:
            with open(file_path, 'w', encoding='utf-8') as f:
                f.write(content)
            print(f"Исправлен файл: {file_path}")
            return True
        return False
        
    except Exception as e:
        print(f"Ошибка при обработке файла {file_path}: {e}")
        return False

def main():
    src_dir = "src"
    rust_files = [f for f in os.listdir(src_dir) if f.endswith('.rs')]
    
    fixed_count = 0
    for file_name in rust_files:
        file_path = os.path.join(src_dir, file_name)
        if fix_chrono_some_in_file(file_path):
            fixed_count += 1
    
    print(f"Исправлено файлов: {fixed_count}")

if __name__ == "__main__":
    main() 