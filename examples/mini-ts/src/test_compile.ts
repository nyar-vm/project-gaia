import {MiniTSParser} from './lib';

// 测试源代码
const testSource = `
fn main() -> i32 {
  let x: i32 = 10;
  let y: i32 = 20;
  let z: i32 = x + y;
  return z;
}

fn add(a: i32, b: i32) -> i32 {
  return a + b;
}

fn factorial(n: i32) -> i32 {
  if (n <= 1) {
    return 1;
  } else {
    return n * factorial(n - 1);
  }
}

fn fibonacci(n: i32) -> i32 {
  if (n <= 1) {
    return n;
  } else {
    return fibonacci(n - 1) + fibonacci(n - 2);
  }
}
`;

function main() {
    console.log('Testing Mini-TS Compiler...');
    console.log('Source code:');
    console.log(testSource);
    console.log('--------------------------------');

    const parser = new MiniTSParser(testSource);
    const result = parser.parse();

    if (result.success) {
        console.log('Parse successful!');
        console.log(`Program name: ${result.program.functions[0]?.name || 'unnamed'}`);
        console.log(`Functions: ${result.program.functions.length}`);

        for (const func of result.program.functions) {
            console.log(`  Function ${func.name}: ${func.instructions.length} instructions`);
            console.log(`    Parameters: ${func.parameters.length}`);
            console.log(`    Return type: ${func.returnType}`);
        }
    } else {
        console.log('Parse failed:');
        console.log(result.error);
    }
}

main();