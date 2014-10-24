import {describe, expect, it} from 'vitest';
import {compiler, parser, easyTest} from '../dist/gaia_assembler.js';

describe('Gaia Compiler Functionality', () => {
    describe('Supported Targets', () => {
        it('should return supported target platforms', async () => {
            const targets = await compiler.getSupportedTargets();
            expect(Array.isArray(targets)).toBe(true);
            expect(targets.length).toBeGreaterThan(0);
            
            // 检查是否包含预期的目标平台
            const targetNames = targets.map(t => t.toString());
            expect(targetNames).toContain('clr');
            expect(targetNames).toContain('jvm');
            expect(targetNames).toContain('pe');
            expect(targetNames).toContain('wasi');
        });
    });

    describe('Compile Options Validation', () => {
        it('should validate compile options', async () => {
            const validOptions = {
                target: 'wasi',
                optimizationLevel: 0,
                debugInfo: true,
                outputPath: null
            };
            
            const result = await compiler.validateOptions(validOptions);
            expect(result).toBeDefined();
        });
    });

    describe('Parser Integration', () => {
        it('should get parser information', async () => {
            const parserInfo = await parser.getParserInfo();
            expect(parserInfo).toBeDefined();
            expect(parserInfo.version).toBeDefined();
            expect(parserInfo.supportedFeatures).toBeDefined();
        });

        it('should validate syntax', async () => {
            const validSource = `
                fn main() {
                    let x = 42;
                    console.log(x);
                }
            `;
            
            const result = await parser.validateSyntax(validSource, null);
            expect(result).toBeDefined();
        });
    });

    describe('Easy Test Integration', () => {
        it('should create hello world test', async () => {
            const test = await easyTest.createHelloWorldTest('wasi');
            expect(test.name).toBe('hello_world');
            expect(test.source).toContain('Hello, Gaia World!');
            expect(test.target).toBe('wasi');
        });

        it('should create arithmetic test', async () => {
            const test = await easyTest.createArithmeticTest('pe');
            expect(test.name).toBe('arithmetic');
            expect(test.source).toContain('10');
            expect(test.source).toContain('20');
            expect(test.expected).toBe('30');
            expect(test.target).toBe('pe');
        });

        it('should validate test case', async () => {
            const validTest = {
                name: 'test',
                source: 'fn main() {}',
                expected: 'output',
                target: 'clr'
            };
            
            const result = await easyTest.validateTestCase(validTest);
            expect(result).toBeDefined();
        });
    });
});