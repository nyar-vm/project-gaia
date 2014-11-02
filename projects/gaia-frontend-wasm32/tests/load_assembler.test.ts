import {describe, expect, it} from 'vitest';
import {
    utils,
    assembler,
    metadata,
    easyTest
} from '../dist/gaia_frontend.js';

describe('Gaia Assembler WebAssembly Module', () => {

    describe('Utils Interface', () => {
        it('should have utils interface defined', () => {
            expect(utils).toBeDefined();
        });
    });

    describe('Assembler Interface', () => {
        it('should have assembler methods', () => {
            expect(assembler).toBeDefined();
            expect(typeof assembler.assemble).toBe('function');
            expect(typeof assembler.getSupportedTargets).toBe('function');
            expect(typeof assembler.validateSyntax).toBe('function');
            expect(typeof assembler.getInstructionSet).toBe('function');
            expect(typeof assembler.disassemble).toBe('function');
        });
    });

    describe('Metadata Interface', () => {
        it('should have metadata interface defined', () => {
            expect(metadata).toBeDefined();
        });
    });

    describe('Easy Test Interface', () => {
        it('should have easy test methods', () => {
            expect(easyTest).toBeDefined();
            expect(typeof easyTest.generateExitCode).toBe('function');
            expect(typeof easyTest.generateConsoleLog).toBe('function');
        });
    });
});