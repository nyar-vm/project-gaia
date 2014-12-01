#!/usr/bin/env python3
"""
用于测试生成的 .pyc 文件的 Python 脚本
"""

import sys
import os
import marshal
import importlib.util

def test_pyc_file(pyc_path):
    """测试 .pyc 文件是否可以正确执行"""
    if not os.path.exists(pyc_path):
        print(f"Error: File {pyc_path} does not exist")
        return False
    
    try:
        # 尝试直接执行 .pyc 文件
        print(f"Attempting to execute {pyc_path}...")
        
        # 读取 .pyc 文件
        with open(pyc_path, 'rb') as f:
            # 跳过头部（16字节）
            magic = f.read(4)
            flags = f.read(4)
            timestamp = f.read(4)
            size = f.read(4)
            
            print(f"Magic: {magic.hex()}")
            print(f"Flags: {flags.hex()}")
            print(f"Timestamp: {timestamp.hex()}")
            print(f"Size: {size.hex()}")
            
            # 尝试读取 marshal 数据
            try:
                code_object = marshal.load(f)
                print(f"Successfully loaded code object: {code_object}")
                
                # 尝试执行代码对象
                exec(code_object)
                return True
            except Exception as e:
                print(f"Failed to load marshal data: {e}")
                return False
                
    except Exception as e:
        print(f"Error reading .pyc file: {e}")
        return False

if __name__ == "__main__":
    if len(sys.argv) != 2:
        print("Usage: python run_pyc.py <pyc_file>")
        sys.exit(1)
    
    pyc_file = sys.argv[1]
    success = test_pyc_file(pyc_file)
    
    if success:
        print("✓ .pyc file executed successfully")
        sys.exit(0)
    else:
        print("✗ .pyc file execution failed")
        sys.exit(1)