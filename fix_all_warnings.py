#!/usr/bin/env python3
import os
import re

def fix_analysis_warnings_comprehensive(file_path):
    """Всесторонне исправляет AnalysisWarning структуры"""
    try:
        with open(file_path, 'r', encoding='utf-8') as f:
            content = f.read()
        
        original_content = content
        
        # Паттерн 1: AnalysisWarning { level: X, message: Y, ... }
        # Добавляет category если его нет
        def add_category_if_missing(match):
            full_match = match.group(0)
            
            # Если category уже есть, не изменяем
            if 'category:' in full_match:
                return full_match
            
            # Находим позицию после message
            lines = full_match.split('\n')
            new_lines = []
            category_added = False
            
            for line in lines:
                new_lines.append(line)
                # Добавляем category после строки с message
                if 'message:' in line and not category_added:
                    # Определяем отступ
                    indent = ''
                    for char in line:
                        if char == ' ':
                            indent += char
                        else:
                            break
                    
                    # Определяем категорию по содержимому сообщения
                    if 'сложность' in line.lower() or 'complexity' in line.lower():
                        category = 'complexity'
                    elif 'связанность' in line.lower() or 'coupling' in line.lower():
                        category = 'coupling'
                    elif 'зависимост' in line.lower() or 'cycle' in line.lower():
                        category = 'dependencies'
                    elif 'слой' in line.lower() or 'layer' in line.lower():
                        category = 'architecture'
                    elif 'имен' in line.lower() or 'naming' in line.lower():
                        category = 'naming'
                    else:
                        category = 'validation'
                    
                    new_lines.append(f'{indent}category: "{category}".to_string(),')
                    category_added = True
            
            return '\n'.join(new_lines)
        
        # Применяем к блокам AnalysisWarning
        pattern = r'AnalysisWarning\s*\{[^}]*?\}'
        content = re.sub(pattern, add_category_if_missing, content, flags=re.DOTALL)
        
        # Паттерн 2: Исправляем Vec<String> warnings на Vec<AnalysisWarning>
        # warnings.push("строка") -> warnings.push(AnalysisWarning { ... })
        def convert_string_warning(match):
            indent = match.group(1)
            warning_text = match.group(2)
            
            return f'''{indent}warnings.push(AnalysisWarning {{
{indent}    message: {warning_text},
{indent}    level: Priority::Medium,
{indent}    category: "analysis".to_string(),
{indent}    capsule_id: None,
{indent}    suggestion: None,
{indent}}});'''
        
        pattern_string_warnings = r'(\s*)warnings\.push\(([^)]+\.to_string\(\))\);'
        content = re.sub(pattern_string_warnings, convert_string_warning, content)
        
        # Паттерн 3: Исправляем format! макросы в warnings.push
        def convert_format_warning(match):
            indent = match.group(1)
            format_content = match.group(2)
            
            return f'''{indent}warnings.push(AnalysisWarning {{
{indent}    message: format!({format_content}),
{indent}    level: Priority::Medium,
{indent}    category: "analysis".to_string(),
{indent}    capsule_id: None,
{indent}    suggestion: None,
{indent}}});'''
        
        pattern_format_warnings = r'(\s*)warnings\.push\(format!\(([^;]+)\)\);'
        content = re.sub(pattern_format_warnings, convert_format_warning, content)
        
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
        if fix_analysis_warnings_comprehensive(file_path):
            fixed_count += 1
    
    print(f"Исправлено файлов: {fixed_count}")
    
    # Также исправляем capsule.warnings.push(warning.message)
    print("Исправляю warnings.push(warning.message)...")
    for file_name in rust_files:
        file_path = os.path.join(src_dir, file_name)
        try:
            with open(file_path, 'r', encoding='utf-8') as f:
                content = f.read()
            
            # capsule.warnings.push(warning.message) -> capsule.warnings.push(warning)
            original_content = content
            content = re.sub(r'\.warnings\.push\(([^)]+)\.message\)', r'.warnings.push(\1)', content)
            
            if content != original_content:
                with open(file_path, 'w', encoding='utf-8') as f:
                    f.write(content)
                print(f"Исправлены warning.message в: {file_path}")
        except Exception as e:
            print(f"Ошибка: {e}")

if __name__ == "__main__":
    main() 