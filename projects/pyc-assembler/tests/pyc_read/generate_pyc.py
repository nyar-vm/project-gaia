import py_compile
import os

def create_and_compile(filename, content):
    with open(filename, 'w') as f:
        f.write(content)
    py_compile.compile(filename)
    # Delete the source file after compilation
    os.remove(filename)
    print(f"Generated {filename}")

os.chdir(os.path.dirname(__file__))

create_and_compile('simple.py', """
print("Hello Python!")
""")
create_and_compile('another.py', """
def greet(name):
    return f"Hello, {name}"

print(greet("World"))
""")

create_and_compile('loop_test.py', """
for i in range(3):
    print(i)
""")

create_and_compile('conditional_test.py', """
x = 10
if x > 5:
    print("x is greater than 5")
else:
    print("x is not greater than 5")
""")

create_and_compile('async_test.py', """
import asyncio

async def hello_async():
    await asyncio.sleep(0.01)
    return "Hello from async!"

async def main():
    result = await hello_async()
    print(result)

if __name__ == "__main__":
    asyncio.run(main())
""")

create_and_compile('generator_test.py', """
def simple_generator():
    yield 1
    yield 2
    yield 3

for value in simple_generator():
    print(value)
""")