#!/usr/bin/env python3
import os
import re

def fix_analysis_warning_in_file(file_path):
    """Добавляет недостающие поля category в AnalysisWarning"""
    try:
        with open(file_path, 'r', encoding='utf-8') as f:
            content = f.read()
        
        original_content = content
        
        # Паттерн для поиска AnalysisWarning без category
        pattern = r'(AnalysisWarning\s*\{[^}]*?)(\s*capsule_id:[^,]+,\s*suggestion:[^,}]+)(\s*\})'
        
        def replace_warning(match):
            prefix = match.group(1)
            middle = match.group(2)
            suffix = match.group(3)
            
            # Проверяем, есть ли уже category
            if 'category:' in prefix or 'category:' in middle:
                return match.group(0)
            
            # Добавляем category перед capsule_id
            return prefix + ',\n                category: "validation".to_string(),' + middle + suffix
        
        content = re.sub(pattern, replace_warning, content, flags=re.DOTALL)
        
        # Также исправляем случаи где есть только message и level
        pattern2 = r'(AnalysisWarning\s*\{\s*message:[^,]+,\s*level:[^,}]+)(\s*\})'
        
        def replace_simple_warning(match):
            prefix = match.group(1)
            suffix = match.group(2)
            
            return prefix + ',\n                category: "validation".to_string(),\n                capsule_id: None,\n                suggestion: None' + suffix
        
        content = re.sub(pattern2, replace_simple_warning, content, flags=re.DOTALL)
        
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
        if fix_analysis_warning_in_file(file_path):
            fixed_count += 1
    
    print(f"Исправлено файлов: {fixed_count}")

if __name__ == "__main__":
    main() 