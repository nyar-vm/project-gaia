// 导出所有模块
export * from './ast';
export * from './lexer';
export * from './parser';
export * from './codegen';

// 导出主要的解析器类
import {Parser} from './parser';
import {CodeGenerator} from './codegen';

export class MiniTSParser {
    private parser: Parser;
    private codegen: CodeGenerator;

    constructor(input: string) {
        this.parser = new Parser(input);
        this.codegen = new CodeGenerator();
    }

    public parse() {
        try {
            const ast = this.parser.parse();
            const gaiaProgram = this.codegen.generate(ast);

            return {
                success: true,
                program: gaiaProgram,
                ast: ast
            };
        } catch (error) {
            return {
                success: false,
                error: error instanceof Error ? error.message : String(error)
            };
        }
    }
}