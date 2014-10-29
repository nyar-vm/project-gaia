import {readFileSync} from 'fs';
import {MiniTSParser} from './lib';

function main() {
    if (process.argv.length < 3) {
        console.log('Usage: node dist/index.js <input-file>');
        process.exit(1);
    }

    const inputFile = process.argv[2];
    const sourceCode = readFileSync(inputFile, 'utf8');

    console.log(`Compiling ${inputFile}...`);

    const parser = new MiniTSParser(sourceCode);
    const result = parser.parse();

    if (result.success) {
        console.log('Compilation successful!');
        console.log(`Program name: ${result.program.functions[0]?.name || 'unnamed'}`);
        console.log(`Functions: ${result.program.functions.length}`);

        for (const func of result.program.functions) {
            console.log(`  Function ${func.name}: ${func.instructions.length} instructions`);
        }
    } else {
        console.log('Compilation failed:');
        console.log(result.error);
        process.exit(1);
    }
}

main();