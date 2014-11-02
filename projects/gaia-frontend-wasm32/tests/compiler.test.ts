import {describe, expect, it} from 'vitest';
import {assembler, easyTest} from '../dist/gaia_frontend.js';

describe('Gaia Assembler Functionality', () => {
    describe('Supported Targets', () => {
        it('should return supported target platforms', async () => {
            const targets = await assembler.getSupportedTargets();
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

    describe('Syntax Validation', () => {
        it('should validate syntax', async () => {
            const validSource = `
                fn main() {
                    let x = 42;
                    console.log(x);
                }
            `;

            const result = await assembler.validateSyntax(validSource);
            expect(result).toBeDefined();
        });
    });

    describe('Easy Test Integration', () => {
        it('should generate exit code', async () => {
            const bytecode = await easyTest.generateExitCode(0, 'jvm');
            expect(bytecode).toBeInstanceOf(Uint8Array);
            expect(bytecode.length).toBeGreaterThan(0);
        });

        it('should generate console log', async () => {
            const bytecode = await easyTest.generateConsoleLog('Hello, Gaia!', 'jvm');
            expect(bytecode).toBeInstanceOf(Uint8Array);
            expect(bytecode.length).toBeGreaterThan(0);
        });
    });
});