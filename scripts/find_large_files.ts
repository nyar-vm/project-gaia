import * as fs from 'fs';
import * as path from 'path';

const MAX_LINES = 1000;
const IGNORE_DIRS = ['node_modules', 'target', '.git', 'dist', 'cache', 'temp', '.vitepress'];
const ROOT_DIR = path.resolve(__dirname, '..');

function countLines(filePath: string): number {
    const content = fs.readFileSync(filePath, 'utf-8');
    return content.split('\n').length;
}

function findLargeFiles(dir: string, fileList: string[] = []): string[] {
    const files = fs.readdirSync(dir);

    for (const file of files) {
        const filePath = path.join(dir, file);
        const stat = fs.statSync(filePath);

        if (stat.isDirectory()) {
            if (!IGNORE_DIRS.includes(file) && !file.startsWith('.')) {
                findLargeFiles(filePath, fileList);
            }
        } else {
            // Check only source files
            if (/\.(rs|ts|js|py|cs|il)$/.test(file)) {
                try {
                    const lines = countLines(filePath);
                    if (lines > MAX_LINES) {
                        fileList.push(`${filePath} (${lines} lines)`);
                    }
                } catch (e) {
                    // console.error(`Error reading file ${filePath}: ${e}`);
                }
            }
        }
    }

    return fileList;
}

console.log(`Searching for files with more than ${MAX_LINES} lines in ${ROOT_DIR}...`);
const largeFiles = findLargeFiles(ROOT_DIR);

if (largeFiles.length === 0) {
    console.log('No files found with more than 1000 lines.');
} else {
    console.log('\nFound large files:');
    largeFiles.forEach(file => console.log(file));
}
