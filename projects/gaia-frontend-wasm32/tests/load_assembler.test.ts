import {describe, expect, it} from 'vitest';
import {
    types,
    instructions,
    compiler,
    parser,
    backends,
    adapters,
    program,
    easyTest
} from '../dist/gaia_assembler.js';

describe('Gaia Assembler WebAssembly Module', () => {
    describe('Types Interface', () => {
        it('should have types interface defined', () => {
            expect(types).toBeDefined();
        });
    });

    describe('Instructions Interface', () => {
        it('should have instructions interface defined', () => {
            expect(instructions).toBeDefined();
        });
    });

    describe('Compiler Interface', () => {
        it('should have compiler methods', () => {
            expect(compiler).toBeDefined();
            expect(typeof compiler.compileProgram).toBe('function');
            expect(typeof compiler.getSupportedTargets).toBe('function');
            expect(typeof compiler.validateOptions).toBe('function');
        });
    });

    describe('Parser Interface', () => {
        it('should have parser methods', () => {
            expect(parser).toBeDefined();
            expect(typeof parser.parseSource).toBe('function');
            expect(typeof parser.parseExpression).toBe('function');
            expect(typeof parser.validateSyntax).toBe('function');
            expect(typeof parser.getParserInfo).toBe('function');
        });
    });

    describe('Backends Interface', () => {
        it('should have backends methods', () => {
            expect(backends).toBeDefined();
            expect(typeof backends.generateCode).toBe('function');
            expect(typeof backends.getDefaultOptions).toBe('function');
            expect(typeof backends.validateBackendOptions).toBe('function');
        });
    });

    describe('Adapters Interface', () => {
        it('should have adapters methods', () => {
            expect(adapters).toBeDefined();
            expect(typeof adapters.importModule).toBe('function');
            expect(typeof adapters.generateExports).toBe('function');
            expect(typeof adapters.getImportAdapters).toBe('function');
            expect(typeof adapters.getExportAdapters).toBe('function');
        });
    });

    describe('Program Interface', () => {
        it('should have program methods', () => {
            expect(program).toBeDefined();
            expect(typeof program.createProgram).toBe('function');
            expect(typeof program.addFunction).toBe('function');
            expect(typeof program.addGlobal).toBe('function');
            expect(typeof program.validateProgram).toBe('function');
        });
    });

    describe('Easy Test Interface', () => {
        it('should have easy test methods', () => {
            expect(easyTest).toBeDefined();
            expect(typeof easyTest.runTest).toBe('function');
            expect(typeof easyTest.runTestSuite).toBe('function');
            expect(typeof easyTest.createHelloWorldTest).toBe('function');
            expect(typeof easyTest.createArithmeticTest).toBe('function');
            expect(typeof easyTest.validateTestCase).toBe('function');
        });
    });
});